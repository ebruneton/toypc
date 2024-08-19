// This program is free software: you can redistribute it and/or modify it under
// the terms of the GNU General Public License as published by the Free Software
// Foundation, either version 3 of the License, or (at your option) any later
// version.
//
// This program is distributed in the hope that it will be useful, but WITHOUT
// ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
// FOR A PARTICULAR PURPOSE. See the GNU General Public License for more
// details.
//
// You should have received a copy of the GNU General Public License along with
// this program. If not, see <https://www.gnu.org/licenses/>.

use std::cell::RefCell;
use std::rc::Rc;

use crate::arm::{decode_insn, is32bit_insn, Instruction};
use crate::boot::BootMonitor;
use crate::bus::BusMatrix;
use crate::pio::{Controller, PioDevice};
use crate::spi::SpiDevice;
use crate::system::VECTOR_TABLE_OFFSET_REGISTER;
use crate::time::WaitFunction;

const STACK_POINTER: usize = 13;
const LINK_REGISTER: usize = 14;

const MAIN_STACK_POINTER_SPECIAL_REGISTER: u8 = 8;
const PROCESS_STACK_POINTER_SPECIAL_REGISTER: u8 = 9;
const CONTROL_SPECIAL_REGISTER: u8 = 20;

// See section A7.3, Conditional execution.
const CONDITION_EQ: u8 = 0b0000;
const CONDITION_NE: u8 = 0b0001;
const CONDITION_GE: u8 = 0b0010;
const CONDITION_LT: u8 = 0b0011;
const CONDITION_GT: u8 = 0b1000;
const CONDITION_LE: u8 = 0b1001;

#[derive(Clone)]
pub struct MicroController {
    on: bool,
    boot_monitor: BootMonitor,
    bus_matrix: BusMatrix,
    registers: [u32; 15],
    program_counter: u32,
    last_comparison: std::cmp::Ordering,
    if_then_state: u8,
    interrupt_status: u8,
    main_stack: bool,
    shadow_stack_pointer: u32,
    run_from_flash: bool,
    max_boot_program_go_cycles: u32,
}

impl Default for MicroController {
    fn default() -> Self {
        Self::new(1, None)
    }
}

fn is_cmp_insn(insn: u32) -> bool {
    (insn & 0b1111100000000000 == 0b0010100000000000)
        || (insn & 0b1111111111000000 == 0b0100001010000000)
}

impl MicroController {
    // Special program counter value used to identify when a program run with the SAM-BA "G" command
    // returns in the SAM-BA monitor. This must be an even value of the form of an EXC_RETURN.
    const BOOT_MONITOR_PROGRAM_COUNTER: u32 = 0xF0123456;
    // Initial value of the link register, after reset (Section 10.4.3.3 of SAM3X datasheet).
    const INITIAL_LINK_REGISTER_VALUE: u32 = 0xFFFFFFFF;

    pub fn new(seed: u32, wait_function: Option<WaitFunction>) -> Self {
        let mut result = Self {
            on: true,
            boot_monitor: BootMonitor::new(),
            bus_matrix: BusMatrix::uninitialized(seed, wait_function),
            registers: [0; 15],
            program_counter: 0,
            last_comparison: std::cmp::Ordering::Equal,
            if_then_state: 0,
            interrupt_status: 0,
            main_stack: true,
            shadow_stack_pointer: 0,
            run_from_flash: false,
            max_boot_program_go_cycles: 10000000,
        };
        result.erase();
        result.reset();
        result
    }

    pub fn set_max_boot_program_go_cycles(&mut self, max_boot_program_go_cycles: u32) {
        self.max_boot_program_go_cycles = max_boot_program_go_cycles;
    }

    pub fn get_serial_output(&mut self) -> String {
        assert!(self.on);
        if self.run_from_flash {
            String::new()
        } else {
            self.boot_monitor.get_output()
        }
    }

    pub fn set_serial_input(&mut self, data: &str) -> bool {
        assert!(self.on);
        if self.run_from_flash {
            return false;
        }
        let go_address = self.boot_monitor.parse_input(data, &mut self.bus_matrix);
        if let Some(address) = go_address {
            assert!(
                self.bus_matrix.get32(address + 4) & 1 == 1,
                "Bad interworking address at {:#010X}",
                address + 4
            );
            const INITIAL_STACK_POINTER: u32 = 0x20001000; /* First word or ROM. */
            self.registers[STACK_POINTER] = INITIAL_STACK_POINTER;
            self.registers[LINK_REGISTER] = Self::BOOT_MONITOR_PROGRAM_COUNTER | 1;
            self.program_counter = self.bus_matrix.get32(address + 4) & !1;
            let mut counter = self.max_boot_program_go_cycles;
            while self.program_counter != Self::BOOT_MONITOR_PROGRAM_COUNTER && counter > 0 {
                self.emulate_one_insn();
                counter -= 1;
            }
            if counter == 0 {
                return false;
            }
            assert_eq!(
                self.registers[STACK_POINTER], INITIAL_STACK_POINTER,
                "Stack Pointer register not properly restored"
            );
            self.boot_monitor.write_prompt();
        }
        if self.reset_requested() {
            self.reset();
        }
        true
    }

    pub fn get_spi_device(&self) -> Rc<RefCell<dyn SpiDevice>> {
        self.bus_matrix.get_spi_controller().get_spi_device()
    }

    pub fn set_spi_device(&mut self, spi_device: Rc<RefCell<dyn SpiDevice>>) {
        self.bus_matrix
            .get_spi_controller_mut()
            .set_spi_device(spi_device);
    }

    // PS/2 requires USART configured in normal mode (0), external clock (3), 8 bit characters (3),
    // synchronous mode (1), ODD parity (1), 1 stop bit (0), normal channel (0), least significant
    // bit first (0), non inverted data (0) and no Manchester encoder (0).
    // required_mask 0b00100000_10000001_11111111_11111111
    // required mode 0b00000000_00000000_00000011_11110000
    pub fn set_usart_input(&mut self, character: u32, required_mode_mask: u32, required_mode: u32) {
        assert!(self.on);
        let clock_enabled = self
            .bus_matrix
            .get_power_management_controller()
            .usart_clock_enabled();
        let input_pins_enabled = self
            .bus_matrix
            .get_pio_controller()
            .usart_input_pins_enabled();
        if clock_enabled && input_pins_enabled {
            self.bus_matrix.get_usart_controller_mut().data_received(
                character,
                required_mode_mask,
                required_mode,
            );
        }
    }

    pub fn get_pio_device(&self) -> Rc<RefCell<dyn PioDevice>> {
        self.bus_matrix.get_pio_controller().get_pio_device()
    }

    pub fn set_pio_device(&mut self, pio_device: Rc<RefCell<dyn PioDevice>>) {
        self.bus_matrix
            .get_pio_controller_mut()
            .set_pio_device(pio_device)
    }

    pub fn get_pin_output(&self, controller: Controller, pin: u32) -> bool {
        assert!(self.on);
        self.bus_matrix
            .get_pio_controller()
            .get_pin_output(controller, pin)
    }

    pub fn debug_get_flash_content(&mut self) -> &Vec<u32> {
        self.bus_matrix.get_flash_content()
    }

    pub fn debug_get32(&mut self, address: u32) -> u32 {
        let mpu_enable = self.bus_matrix.get_memory_protection_unit().enable;
        self.bus_matrix.get_memory_protection_unit().enable = false;
        let result = self.bus_matrix.get32(address);
        self.bus_matrix.get_memory_protection_unit().enable = mpu_enable;
        result
    }

    pub fn debug_set32(&mut self, address: u32, value: u32) {
        self.bus_matrix.set32(address, value);
    }

    pub fn run_from_flash(&self) -> bool {
        self.run_from_flash
    }

    pub fn run(&mut self, instruction_count: u32) {
        assert!(self.on && self.run_from_flash);
        for _ in 0..instruction_count {
            self.emulate_one_insn();
        }
    }

    pub fn run_until<F>(&mut self, mut is_done: F) -> u32
    where
        F: FnMut(Instruction, u32, u32) -> bool,
    {
        assert!(self.on && self.run_from_flash);
        let mut instruction_count = 0;
        loop {
            let r0 = self.registers[0];
            let r1 = self.registers[1];
            let instruction = self.emulate_one_insn();
            instruction_count += 1;
            if is_done(instruction, r0, r1) {
                return instruction_count;
            }
        }
    }

    pub fn run_until_reset_or<F>(&mut self, mut is_done: F) -> u32
    where
        F: FnMut(Instruction, u32, u32) -> bool,
    {
        assert!(self.on && self.run_from_flash);
        let mut instruction_count = 0;
        loop {
            let r0 = self.registers[0];
            let r1 = self.registers[1];
            let instruction = self.emulate_one_insn();
            instruction_count += 1;
            if self.reset_requested() || is_done(instruction, r0, r1) {
                if self.reset_requested() {
                    self.reset();
                }
                return instruction_count;
            }
        }
    }

    fn emulate_one_insn(&mut self) -> Instruction {
        use self::Instruction::*;
        if self.reset_requested() {
            self.reset();
        }
        if let Option::Some(interrupt) = self.bus_matrix.update() {
            self.enter_interrupt(interrupt + 16);
        }

        if self.in_if_then_block() {
            // See section A7.3, Conditional execution.
            let condition = self.if_then_state >> 4;
            let condition_passed = match condition {
                CONDITION_EQ => self.last_comparison.is_eq(),
                CONDITION_NE => self.last_comparison.is_ne(),
                CONDITION_GE => self.last_comparison.is_ge(),
                CONDITION_LT => self.last_comparison.is_lt(),
                CONDITION_GT => self.last_comparison.is_gt(),
                CONDITION_LE => self.last_comparison.is_le(),
                _ => panic!("Unsupported IT condition {condition:#b}"),
            };
            // See section A7.3.3, ITSTATE, including table A7-2 and ITAdvance() pseudocode.
            self.if_then_state = (self.if_then_state & 0xE0) | (self.if_then_state & 0xF) << 1;
            if !condition_passed {
                let insn = self.bus_matrix.get16(self.program_counter);
                self.program_counter += if is32bit_insn(insn) { 4 } else { 2 };
                return Instruction::Unknown;
            }
        }
        let mut instruction = self.bus_matrix.get_insn(self.program_counter);
        loop {
            match instruction {
                AddRdnImm8 { rdn, imm8 } => {
                    self.registers[rdn as usize] =
                        self.registers[rdn as usize].wrapping_add(imm8 as u32);
                    self.program_counter += 2;
                }
                AddRdRnRm { rd, rn, rm } => {
                    self.registers[rd as usize] =
                        self.registers[rn as usize].wrapping_add(self.registers[rm as usize]);
                    self.program_counter += 2;
                }
                AddRdSpImm8 { rd, imm } => {
                    self.registers[rd as usize] = self.registers[STACK_POINTER] + imm as u32;
                    self.program_counter += 2;
                }
                AddSpSpImm7 { imm } => {
                    self.registers[STACK_POINTER] += imm as u32;
                    self.program_counter += 2;
                }
                AdrRdMinusImm12 { rd, imm12 } => {
                    self.registers[rd as usize] =
                        ((self.program_counter + 4) & 0xFFFFFFFC) - imm12 as u32;
                    self.program_counter += 4;
                }
                AndRdnRm { rdn, rm } => {
                    self.registers[rdn as usize] &= self.registers[rm as usize];
                    self.program_counter += 2;
                }
                BImm11 { imm } => {
                    assert!(
                        !self.in_if_then_block(),
                        "B instruction inside an IT block at {:#010X}",
                        self.program_counter
                    );
                    self.program_counter = ((self.program_counter + 4) as i32 + imm as i32) as u32;
                }
                BlPlusImm24 { imm16, imm8 } => {
                    assert!(
                        !self.in_if_then_block(),
                        "BL instruction inside an IT block at {:#010X}",
                        self.program_counter
                    );
                    let imm = ((imm8 as u32) << 16) | imm16 as u32;
                    self.registers[LINK_REGISTER] = (self.program_counter + 4) | 1;
                    self.program_counter = self.program_counter + 4 + imm;
                }
                BlMinusImm24 { imm16, imm8 } => {
                    assert!(
                        !self.in_if_then_block(),
                        "BL instruction inside an IT block at {:#010X}",
                        self.program_counter
                    );
                    let imm = ((imm8 as u32) << 16) | imm16 as u32;
                    self.registers[LINK_REGISTER] = (self.program_counter + 4) | 1;
                    self.program_counter = self.program_counter + 3 - imm;
                }
                BlxRm { rm } => {
                    assert!(
                        !self.in_if_then_block(),
                        "BLX instruction inside an IT block at {:#010X}",
                        self.program_counter
                    );
                    assert!(
                        self.registers[rm as usize] & 1 == 1,
                        "Bad BLX interworking address at {:#010X}",
                        self.program_counter
                    );
                    self.registers[LINK_REGISTER] = (self.program_counter + 2) | 1;
                    self.program_counter = self.registers[rm as usize] & !1;
                }
                BxRm { rm } => {
                    assert!(
                        !self.in_if_then_block(),
                        "BX instruction inside an IT block at {:#010X}",
                        self.program_counter
                    );
                    assert!(
                        self.registers[rm as usize] & 1 == 1,
                        "Bad BX interworking address at {:#010X}",
                        self.program_counter
                    );
                    self.program_counter = self.registers[rm as usize] & !1;
                    if Self::is_exc_return(self.program_counter)
                        && self.program_counter != Self::BOOT_MONITOR_PROGRAM_COUNTER
                    {
                        self.exit_interrupt(self.registers[rm as usize]);
                    }
                }
                CbzRnImm6 { rn, imm } => {
                    // TODO: error even if last in IT!
                    assert!(
                        !self.in_if_then_block(),
                        "CBZ instruction inside an IT block at {:#010X}",
                        self.program_counter
                    );
                    if self.registers[rn as usize] == 0 {
                        self.program_counter += imm as u32 + 4;
                    } else {
                        self.program_counter += 2;
                    }
                }
                CmpRnImm8 { rn, imm8 } => {
                    self.last_comparison = self.registers[rn as usize].cmp(&(imm8 as u32));
                    self.program_counter += 2;
                }
                CmpRnRm { rn, rm } => {
                    self.last_comparison =
                        self.registers[rn as usize].cmp(&self.registers[rm as usize]);
                    self.program_counter += 2;
                }
                It {
                    first_cond_and_mask,
                } => {
                    assert!(
                        is_cmp_insn(self.bus_matrix.get16(self.program_counter - 2)),
                        "IT instructions not preceded by CMP instruction at {:#010X}",
                        self.program_counter
                    );
                    // TODO: error even if last in IT!
                    assert!(
                        !self.in_if_then_block(),
                        "IT instruction inside an IT block at {:#010X}",
                        self.program_counter
                    );
                    self.if_then_state = first_cond_and_mask;
                    self.program_counter += 2;
                }
                LdrbRtRnImm5 { rt, rn, imm5 } => {
                    self.registers[rt as usize] = self
                        .bus_matrix
                        .get8(self.registers[rn as usize] + imm5 as u32);
                    self.program_counter += 2;
                }
                LdrhRtRnImm5 { rt, rn, imm } => {
                    self.registers[rt as usize] = self
                        .bus_matrix
                        .get16(self.registers[rn as usize] + imm as u32);
                    self.program_counter += 2;
                }
                LdrRtPcImm8 { rt, imm } => {
                    self.registers[rt as usize] = self
                        .bus_matrix
                        .get32(((self.program_counter + 4) & 0xFFFFFFFC) + imm as u32);
                    self.program_counter += 2;
                }
                LdrRtRnImm5 { rt, rn, imm } => {
                    self.registers[rt as usize] = self
                        .bus_matrix
                        .get32(self.registers[rn as usize] + imm as u32);
                    self.program_counter += 2;
                }
                LdrRtSpImm8 { rt, imm } => {
                    self.registers[rt as usize] = self
                        .bus_matrix
                        .get32(self.registers[STACK_POINTER] + imm as u32);
                    self.program_counter += 2;
                }
                LslRdnRm { rdn, rm } => {
                    self.registers[rdn as usize] <<= self.registers[rm as usize] & 0xFF;
                    self.program_counter += 2;
                }
                LslRdRmImm5 { rd, rm, imm5 } => {
                    self.registers[rd as usize] = self.registers[rm as usize] << imm5;
                    self.program_counter += 2;
                }
                LsrRdnRm { rdn, rm } => {
                    self.registers[rdn as usize] >>= self.registers[rm as usize] & 0xFF;
                    self.program_counter += 2;
                }
                LsrRdRmImm5 { rd, rm, imm } => {
                    if imm == 32 {
                        self.registers[rd as usize] = 0;
                    } else {
                        self.registers[rd as usize] = self.registers[rm as usize] >> imm;
                    }
                    self.program_counter += 2;
                }
                MovRdImm8 { rd, imm8 } => {
                    self.registers[rd as usize] = imm8 as u32;
                    self.program_counter += 2;
                }
                MovRdRm { rd, rm } => {
                    if rd == 15 {
                        assert!(
                            !self.in_if_then_block(),
                            "MOV PC instruction inside an IT block at {:#010X}",
                            self.program_counter
                        );
                        self.program_counter = self.registers[rm as usize] & !1;
                    } else {
                        self.registers[rd as usize] = self.registers[rm as usize];
                        self.program_counter += 2;
                    }
                }
                MovtRdImm16 { rd, imm16 } => {
                    self.registers[rd as usize] =
                        ((imm16 as u32) << 16) | (self.registers[rd as usize] & 0xFFFF);
                    self.program_counter += 4;
                }
                MovwRdImm16 { rd, imm16 } => {
                    self.registers[rd as usize] = imm16 as u32;
                    self.program_counter += 4;
                }
                MrsRdReg { rd, reg } => {
                    self.registers[rd as usize] = 0;
                    match reg {
                        MAIN_STACK_POINTER_SPECIAL_REGISTER => {
                            if self.bus_matrix.is_privileged() {
                                self.registers[rd as usize] = self.get_stack_pointer(true);
                            }
                        }
                        PROCESS_STACK_POINTER_SPECIAL_REGISTER => {
                            if self.bus_matrix.is_privileged() {
                                self.registers[rd as usize] = self.get_stack_pointer(false);
                            }
                        }
                        CONTROL_SPECIAL_REGISTER => {
                            self.registers[rd as usize] = (!self.bus_matrix.privilege_mode) as u32;
                        }
                        _ => {
                            if self.bus_matrix.is_privileged() {
                                panic!("Unsupported special register {reg:X}");
                            }
                        }
                    }
                    self.program_counter += 4;
                }
                MsrRegRn { rn, reg } => {
                    match reg {
                        MAIN_STACK_POINTER_SPECIAL_REGISTER => {
                            if self.bus_matrix.is_privileged() {
                                self.set_stack_pointer(self.registers[rn as usize], true);
                            }
                        }
                        PROCESS_STACK_POINTER_SPECIAL_REGISTER => {
                            if self.bus_matrix.is_privileged() {
                                self.set_stack_pointer(self.registers[rn as usize], false);
                            }
                        }
                        CONTROL_SPECIAL_REGISTER => {
                            if self.bus_matrix.is_privileged() {
                                if (self.registers[rn as usize] & 2) != 0 {
                                    panic!("Unsupported CONTROL register value {rn:X}");
                                }
                                self.bus_matrix.privilege_mode =
                                    (self.registers[rn as usize] & 1) == 0;
                            }
                        }
                        _ => {
                            if self.bus_matrix.is_privileged() {
                                panic!("Unsupported special register {reg:X}")
                            }
                        }
                    }
                    self.program_counter += 4;
                }
                MulRdmRn { rdm, rn } => {
                    self.registers[rdm as usize] =
                        self.registers[rdm as usize].wrapping_mul(self.registers[rn as usize]);
                    self.program_counter += 2;
                }
                OrrRdnRm { rdn, rm } => {
                    self.registers[rdn as usize] |= self.registers[rm as usize];
                    self.program_counter += 2;
                }
                Pop { registers, pc } => {
                    for i in 0..8 {
                        if (registers & (1 << i)) != 0 {
                            self.registers[i] =
                                self.bus_matrix.get32(self.registers[STACK_POINTER]);
                            self.registers[STACK_POINTER] += 4;
                        }
                    }
                    if pc {
                        let new_program_counter =
                            self.bus_matrix.get32(self.registers[STACK_POINTER]);
                        self.registers[STACK_POINTER] += 4;
                        assert!(
                            !self.in_if_then_block(),
                            "POP PC instruction inside an IT block at {:#010X}",
                            self.program_counter
                        );
                        assert!(
                            new_program_counter & 1 == 1,
                            "Bad POP PC interworking address at {:#010X}",
                            self.program_counter
                        );
                        self.program_counter = new_program_counter & !1;
                        if Self::is_exc_return(self.program_counter)
                            && self.program_counter != Self::BOOT_MONITOR_PROGRAM_COUNTER
                        {
                            self.exit_interrupt(new_program_counter);
                        }
                    } else {
                        self.program_counter += 2;
                    }
                }
                Push { registers, lr } => {
                    if lr {
                        self.registers[STACK_POINTER] -= 4;
                        self.bus_matrix
                            .set32(self.registers[STACK_POINTER], self.registers[LINK_REGISTER]);
                    }
                    for i in (0..8).rev() {
                        if (registers & (1 << i)) != 0 {
                            self.registers[STACK_POINTER] -= 4;
                            self.bus_matrix
                                .set32(self.registers[STACK_POINTER], self.registers[i]);
                        }
                    }
                    self.program_counter += 2;
                }
                StrbRtRnImm5 { rt, rn, imm5 } => {
                    self.bus_matrix.set8(
                        self.registers[rn as usize] + imm5 as u32,
                        self.registers[rt as usize] as u8,
                    );
                    self.program_counter += 2;
                }
                StrhRtRnImm5 { rt, rn, imm } => {
                    self.bus_matrix.set16(
                        self.registers[rn as usize] + imm as u32,
                        self.registers[rt as usize] as u16,
                    );
                    self.program_counter += 2;
                }
                StrRtRnImm5 { rt, rn, imm } => {
                    self.bus_matrix.set32(
                        self.registers[rn as usize] + imm as u32,
                        self.registers[rt as usize],
                    );
                    self.program_counter += 2;
                }
                StrRtSpImm8 { rt, imm } => {
                    self.bus_matrix.set32(
                        self.registers[STACK_POINTER] + imm as u32,
                        self.registers[rt as usize],
                    );
                    self.program_counter += 2;
                }
                SubRdnImm8 { rdn, imm8 } => {
                    self.registers[rdn as usize] =
                        self.registers[rdn as usize].wrapping_sub(imm8 as u32);
                    self.program_counter += 2;
                }
                SubRdRnRm { rd, rn, rm } => {
                    self.registers[rd as usize] =
                        self.registers[rn as usize].wrapping_sub(self.registers[rm as usize]);
                    self.program_counter += 2;
                }
                SubSpSpImm7 { imm } => {
                    self.registers[STACK_POINTER] -= imm as u32;
                    self.program_counter += 2;
                }
                SvcImm8 { imm8: _ } => {
                    const SVC_EXCEPTION_NUMBER: u8 = 11;
                    self.program_counter += 2;
                    if self.interrupt_status != 0 {
                        panic!(
                            "SVC instruction executed while in exception handler at {:#010X}",
                            self.program_counter
                        );
                    }
                    self.enter_interrupt(SVC_EXCEPTION_NUMBER);
                }
                TbbRnRm { rn, rm } => {
                    assert!(
                        !self.in_if_then_block(),
                        "TBB instruction inside an IT block"
                    );
                    let base = if rn == 15 {
                        self.program_counter + 4
                    } else {
                        self.registers[rn as usize]
                    };
                    let index = self.registers[rm as usize];
                    self.program_counter += 2 * self.bus_matrix.get8(base + index) + 4;
                }
                UdivRdRnRm { rd, rn, rm } => {
                    self.registers[rd as usize] =
                        self.registers[rn as usize].wrapping_div(self.registers[rm as usize]);
                    self.program_counter += 4;
                }
                Unknown => {
                    const DECODE_INSTRUCTION_BATCH_SIZE: usize = 32;
                    self.bus_matrix
                        .decode_insns(self.program_counter, DECODE_INSTRUCTION_BATCH_SIZE);
                    instruction = self.bus_matrix.get_insn(self.program_counter);
                    if instruction == Unknown {
                        // This happens if the last 16 bits of a memory range are the start of a
                        // 32 bit instruction.
                        instruction = decode_insn(self.bus_matrix.get32(self.program_counter));
                    }
                    // Go back to the main "match" statement to emulate the now decoded instruction.
                    // Intentionally does not update any state (program_counter, etc).
                    continue;
                }
                Unsupported | Invalid => {
                    panic!(
                        "Unsupported or invalid instruction {insn:#b} at {pc:#X}",
                        insn = self.bus_matrix.get16(self.program_counter),
                        pc = self.program_counter
                    );
                }
            }
            return instruction;
        }
    }

    #[inline(always)]
    fn in_if_then_block(&self) -> bool {
        self.if_then_state & 0xF != 0
    }

    fn enter_interrupt(&mut self, exception_number: u8) {
        debug_assert!(exception_number > 0);
        let vector_table = self
            .bus_matrix
            .get_system_controller()
            .get32_aligned(VECTOR_TABLE_OFFSET_REGISTER);
        let handler_address = vector_table + 4 * (exception_number as u32);
        let mpu_enable = self.bus_matrix.get_memory_protection_unit().enable;
        // Accesses to the vector table for exception entry are always permitted. We ensure this by
        // temporarily disabling the MPU.
        self.bus_matrix.get_memory_protection_unit().enable = false;
        let handler = self.bus_matrix.get32(handler_address);
        self.bus_matrix.get_memory_protection_unit().enable = mpu_enable;
        assert!(
            handler & 1 == 1,
            "Bad handler interworking address at {handler_address:#010X}"
        );

        let mut stack_pointer = self.registers[STACK_POINTER] - 8 * 4;
        let padding = stack_pointer % 8 != 0;
        if padding {
            stack_pointer -= 4;
        }
        self.bus_matrix.set32(stack_pointer, self.registers[0]);
        self.bus_matrix.set32(stack_pointer + 4, self.registers[1]);
        self.bus_matrix.set32(stack_pointer + 8, self.registers[2]);
        self.bus_matrix.set32(stack_pointer + 12, self.registers[3]);
        self.bus_matrix
            .set32(stack_pointer + 16, self.registers[12]);
        self.bus_matrix
            .set32(stack_pointer + 20, self.registers[LINK_REGISTER]);
        self.bus_matrix
            .set32(stack_pointer + 24, self.program_counter);
        self.bus_matrix.set32(
            stack_pointer + 28,
            self.get_pseudo_program_status_register(padding),
        );

        self.registers[STACK_POINTER] = stack_pointer;
        let main_stack = self.main_stack;
        self.set_main_stack(true);
        if self.bus_matrix.handler_mode {
            debug_assert!(main_stack);
            self.registers[LINK_REGISTER] = 0xFFFFFFF1;
        } else if main_stack {
            self.registers[LINK_REGISTER] = 0xFFFFFFF9;
        } else {
            self.registers[LINK_REGISTER] = 0xFFFFFFFD;
        }
        self.program_counter = handler & !1;
        self.last_comparison = std::cmp::Ordering::Equal;
        self.if_then_state = 0;
        self.interrupt_status = exception_number;
        self.bus_matrix.handler_mode = true;
    }

    fn exit_interrupt(&mut self, exc_return: u32) {
        assert!(self.bus_matrix.handler_mode);
        assert!(self.main_stack);
        if exc_return == 0xFFFFFFF9 {
            self.bus_matrix.handler_mode = false;
        } else if exc_return == 0xFFFFFFFD {
            self.set_main_stack(false);
            self.bus_matrix.handler_mode = false;
        } else if exc_return != 0xFFFFFFF1 {
            panic!("Unsupported EXC_RETURN value {}", self.program_counter);
        }
        if self.interrupt_status >= 16 {
            self.bus_matrix.deactivate_interrupt();
        }
        let stack_pointer = self.registers[STACK_POINTER];
        self.registers[0] = self.bus_matrix.get32(stack_pointer);
        self.registers[1] = self.bus_matrix.get32(stack_pointer + 4);
        self.registers[2] = self.bus_matrix.get32(stack_pointer + 8);
        self.registers[3] = self.bus_matrix.get32(stack_pointer + 12);
        self.registers[12] = self.bus_matrix.get32(stack_pointer + 16);
        self.registers[STACK_POINTER] = stack_pointer + 8 * 4;
        self.registers[LINK_REGISTER] = self.bus_matrix.get32(stack_pointer + 20);
        self.program_counter = self.bus_matrix.get32(stack_pointer + 24);
        let pseudo_program_status = self.bus_matrix.get32(stack_pointer + 28);
        let padding = self.set_pseudo_program_status_register(pseudo_program_status);
        if padding {
            self.registers[STACK_POINTER] += 4;
        }
    }

    pub fn get_stack_pointer(&self, main_stack: bool) -> u32 {
        if main_stack == self.main_stack {
            self.registers[STACK_POINTER]
        } else {
            self.shadow_stack_pointer
        }
    }

    fn set_stack_pointer(&mut self, value: u32, main_stack: bool) {
        if main_stack == self.main_stack {
            self.registers[STACK_POINTER] = value;
        } else {
            self.shadow_stack_pointer = value;
        }
    }

    fn set_main_stack(&mut self, main_stack: bool) {
        if main_stack != self.main_stack {
            std::mem::swap(
                &mut self.registers[STACK_POINTER],
                &mut self.shadow_stack_pointer,
            );
        }
        self.main_stack = main_stack;
    }

    fn is_exc_return(value: u32) -> bool {
        value & 0xF0000000 == 0xF0000000
    }

    fn get_pseudo_program_status_register(&self, stack_padding: bool) -> u32 {
        let pseudo_condition_flags = match self.last_comparison {
            std::cmp::Ordering::Equal => 0,
            std::cmp::Ordering::Greater => 1,
            std::cmp::Ordering::Less => 2,
        };
        (1 << 24)
            | (self.if_then_state as u32) << 16
            | (pseudo_condition_flags as u32) << 10
            | (stack_padding as u32) << 9
            | (self.interrupt_status as u32)
    }

    fn set_pseudo_program_status_register(&mut self, pseudo_program_status: u32) -> bool {
        if pseudo_program_status & (1 << 24) == 0 {
            panic!(
                "Bit 24 of the Status Register must be 1 (Thumb mode) (pc={:#X})",
                self.program_counter
            );
        }
        self.if_then_state = (pseudo_program_status >> 16) as u8;
        self.last_comparison = match (pseudo_program_status >> 10) & 3 {
            0 => std::cmp::Ordering::Equal,
            1 => std::cmp::Ordering::Greater,
            _ => std::cmp::Ordering::Less,
        };
        self.interrupt_status = pseudo_program_status as u8;
        (pseudo_program_status >> 9) & 1 != 0
    }

    pub fn is_on(&self) -> bool {
        self.on
    }

    pub fn turn_on(&mut self) {
        assert!(!self.on);
        self.on = true;
        self.reset();
    }

    pub fn turn_off(&mut self) {
        assert!(self.on);
        self.reset();
        self.on = false;
    }

    pub fn erase(&mut self) {
        assert!(self.on);
        self.bus_matrix.erase();
    }

    pub fn reset_requested(&self) -> bool {
        self.bus_matrix.get_reset_controler().reset_requested()
    }

    pub fn reset(&mut self) {
        assert!(self.on);
        self.boot_monitor = BootMonitor::new();
        self.bus_matrix.reset();
        self.registers = [0; 15];
        self.registers[LINK_REGISTER] = Self::INITIAL_LINK_REGISTER_VALUE;
        self.registers[STACK_POINTER] = self.bus_matrix.get32(0);
        self.program_counter = self.bus_matrix.get32(4) & !1;
        self.last_comparison = std::cmp::Ordering::Equal;
        self.interrupt_status = 0;
        self.if_then_state = 0;
        self.main_stack = true;
        self.shadow_stack_pointer = 0;
        self.run_from_flash = self.bus_matrix.boot_from_flash();
    }

    pub fn serialize(&self) -> Vec<u32> {
        assert!(!self.on);
        let mut output = Vec::new();
        self.bus_matrix.serialize(&mut output);
        output
    }

    pub fn deserialize(input: &mut Vec<u32>) -> Self {
        let mut result = Self {
            on: false,
            ..Self::default()
        };
        result.bus_matrix.deserialize(input);
        result
    }
}

#[cfg(test)]
#[allow(clippy::unusual_byte_groupings)]
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use super::MicroController;
    use crate::arm::{decode_insn, Instruction::*};
    use crate::bus::{RAM0_BEGIN, RAM0_BYTES, RAM0_END};
    use crate::chip::{LINK_REGISTER, STACK_POINTER};
    use crate::pio::Controller;
    use crate::{GraphicsCard, PioDevice, SpiDevice};

    #[test]
    fn set_serial_input_write_word() {
        let mut micro_controller = MicroController::default();

        micro_controller.set_serial_input("W20080000,12345678#");

        assert_eq!(micro_controller.bus_matrix.get32(0x20080000), 0x12345678);
    }

    #[test]
    fn set_serial_input_go_return_with_pop() {
        let mut micro_controller = MicroController::default();

        micro_controller.set_serial_input("W20080000,0#"); // Unused (initial stack).
        micro_controller.set_serial_input("W20080004,20080009#"); // Real start address.
        micro_controller.set_serial_input("W20080008,317BB500#"); // PUSH LR, ADD Rdn=1 123.
        micro_controller.set_serial_input("W2008000C,0000BD00#"); // POP PC.
        micro_controller.set_serial_input("G20080000#");

        assert_eq!(micro_controller.registers[1], 123);
    }

    #[test]
    fn set_serial_input_go_return_with_bx_rm() {
        let mut micro_controller = MicroController::default();

        micro_controller.set_serial_input("W20080000,0#"); // Unused (initial stack).
        micro_controller.set_serial_input("W20080004,20080009#"); // Real start address.
        micro_controller.set_serial_input("W20080008,4770317B#"); // ADD Rdn=1 123, BX LR.
        micro_controller.set_serial_input("G20080000#");

        assert_eq!(micro_controller.registers[1], 123);
    }

    #[test]
    #[should_panic(expected = "Stack Pointer register not properly restored")]
    fn set_serial_input_go_sp_not_restored() {
        let mut micro_controller = MicroController::default();

        micro_controller.set_serial_input("W20080000,0#"); // Unused (initial stack).
        micro_controller.set_serial_input("W20080004,20080009#"); // Real start address.
        micro_controller.set_serial_input("W20080008,4770B001#"); // ADD SP=SP+4, BX LR.
        micro_controller.set_serial_input("G20080000#");
    }

    #[test]
    #[should_panic(expected = "Bad interworking address at 0x20080004")]
    fn set_serial_input_go_bad_interworking_address() {
        let mut micro_controller = MicroController::default();

        micro_controller.set_serial_input("W20080000,0#"); // Unused (initial stack).
        micro_controller.set_serial_input("W20080004,20080008#"); // Real start address.
        micro_controller.set_serial_input("W20080008,4770317B#"); // ADD Rdn=1 123, BX LR.
        micro_controller.set_serial_input("G20080000#");
    }

    #[test]
    fn set_serial_input_reset() {
        let mut micro_controller = MicroController::default();

        micro_controller.set_serial_input("W20080000,12345678#");
        micro_controller.set_serial_input("W400E1A00,A500000D#"); // Reset.

        assert_ne!(micro_controller.bus_matrix.get32(0x20080000), 0x12345678);
    }

    #[test]
    fn set_serial_input_run_from_flash() {
        let mut micro_controller = MicroController::default();
        micro_controller.reset();
        micro_controller.run_from_flash = true;

        micro_controller.set_serial_input("W20080000,12345678#");

        assert_ne!(micro_controller.bus_matrix.get32(0x20080000), 0x12345678);
    }

    #[test]
    fn get_serial_output() {
        let mut micro_controller = MicroController::default();

        micro_controller.set_serial_input("w10,#");
        let output = micro_controller.get_serial_output();

        assert_eq!(output, "0x001000C7\n>");
    }

    #[test]
    fn get_serial_output_run_from_flash() {
        let mut micro_controller = MicroController::default();

        micro_controller.set_serial_input("w0,#");
        micro_controller.run_from_flash = true;
        let output = micro_controller.get_serial_output();

        assert_eq!(output, "");
    }

    #[test]
    fn get_set_spi_device() {
        let mut micro_controller = MicroController::default();
        let gpu = Rc::new(RefCell::new(GraphicsCard::default()));

        micro_controller.set_spi_device(gpu.clone());

        assert!(Rc::ptr_eq(
            &micro_controller.get_spi_device(),
            &(gpu as Rc<RefCell<dyn SpiDevice>>)
        ));
    }

    #[test]
    fn get_set_pi_device() {
        let mut micro_controller = MicroController::default();
        let gpu = Rc::new(RefCell::new(GraphicsCard::default()));

        micro_controller.set_pio_device(gpu.clone());

        assert!(Rc::ptr_eq(
            &micro_controller.get_pio_device(),
            &(gpu as Rc<RefCell<dyn PioDevice>>)
        ));
    }

    #[test]
    fn add_rdn_imm8() {
        let mut micro_controller = MicroController::default();
        micro_controller
            .bus_matrix
            .set32(RAM0_BEGIN, 0b00110_010_00001111);
        micro_controller.registers[2] = u32::MAX - 4;
        micro_controller.program_counter = RAM0_BEGIN;

        micro_controller.emulate_one_insn();

        assert_eq!(
            micro_controller.bus_matrix.get_insn(RAM0_BEGIN),
            AddRdnImm8 { rdn: 2, imm8: 15 }
        );
        assert_eq!(micro_controller.registers[2], 15 - 4 - 1);
        assert_eq!(micro_controller.program_counter, RAM0_BEGIN + 2);
    }

    #[test]
    fn add_rd_rn_rm() {
        let mut micro_controller = MicroController::default();
        micro_controller
            .bus_matrix
            .set32(RAM0_BEGIN, 0b0001100_111_110_001);
        micro_controller.registers[1] = 1;
        micro_controller.registers[6] = u32::MAX - 4;
        micro_controller.registers[7] = 15;
        micro_controller.program_counter = RAM0_BEGIN;

        micro_controller.emulate_one_insn();

        assert_eq!(
            micro_controller.bus_matrix.get_insn(RAM0_BEGIN),
            AddRdRnRm {
                rd: 1,
                rn: 6,
                rm: 7
            }
        );
        assert_eq!(micro_controller.registers[1], 15 - 4 - 1);
        assert_eq!(micro_controller.registers[6], u32::MAX - 4);
        assert_eq!(micro_controller.registers[7], 15);
        assert_eq!(micro_controller.program_counter, RAM0_BEGIN + 2);
    }

    #[test]
    fn add_rd_sp_imm8() {
        let mut micro_controller = MicroController::default();
        micro_controller
            .bus_matrix
            .set32(RAM0_BEGIN, 0b10101_001_00000011);
        micro_controller.registers[STACK_POINTER] = RAM0_END;
        micro_controller.program_counter = RAM0_BEGIN;

        micro_controller.emulate_one_insn();

        assert_eq!(
            micro_controller.bus_matrix.get_insn(RAM0_BEGIN),
            AddRdSpImm8 { rd: 1, imm: 12 }
        );
        assert_eq!(micro_controller.registers[1], RAM0_END + 12);
        assert_eq!(micro_controller.program_counter, RAM0_BEGIN + 2);
    }

    #[test]
    fn add_sp_sp_imm7() {
        let mut micro_controller = MicroController::default();
        micro_controller
            .bus_matrix
            .set32(RAM0_BEGIN, 0b101100000_0000011);
        micro_controller.registers[STACK_POINTER] = RAM0_END;
        micro_controller.program_counter = RAM0_BEGIN;

        micro_controller.emulate_one_insn();

        assert_eq!(
            micro_controller.bus_matrix.get_insn(RAM0_BEGIN),
            AddSpSpImm7 { imm: 12 }
        );
        assert_eq!(micro_controller.registers[STACK_POINTER], RAM0_END + 12);
        assert_eq!(micro_controller.program_counter, RAM0_BEGIN + 2);
    }

    #[test]
    fn adr_rd_minus_imm12() {
        let mut micro_controller = MicroController::default();
        micro_controller
            .bus_matrix
            .set32(RAM0_BEGIN + 2, 0b0_000_0001_00001111_1111001010101111);
        micro_controller.program_counter = RAM0_BEGIN + 2;

        micro_controller.emulate_one_insn();

        assert_eq!(
            micro_controller.bus_matrix.get_insn(RAM0_BEGIN + 2),
            AdrRdMinusImm12 { rd: 1, imm12: 15 }
        );
        assert_eq!(micro_controller.registers[1], RAM0_BEGIN + 4 - 15);
        assert_eq!(micro_controller.program_counter, RAM0_BEGIN + 2 + 4);
    }

    #[test]
    fn and_rdn_rm() {
        let mut micro_controller = MicroController::default();
        micro_controller
            .bus_matrix
            .set32(RAM0_BEGIN, 0b0100000000_011_001);
        micro_controller.registers[1] = 0b1100;
        micro_controller.registers[3] = 0b0101;
        micro_controller.program_counter = RAM0_BEGIN;

        micro_controller.emulate_one_insn();

        assert_eq!(
            micro_controller.bus_matrix.get_insn(RAM0_BEGIN),
            AndRdnRm { rdn: 1, rm: 3 }
        );
        assert_eq!(micro_controller.registers[1], 0b0100);
        assert_eq!(micro_controller.registers[3], 0b0101);
        assert_eq!(micro_controller.program_counter, RAM0_BEGIN + 2);
    }

    #[test]
    fn b_imm11() {
        let mut micro_controller = MicroController::default();
        micro_controller
            .bus_matrix
            .set32(RAM0_BEGIN + 2, 0b11100_11111111010);
        micro_controller.program_counter = RAM0_BEGIN + 2;

        micro_controller.emulate_one_insn();

        assert_eq!(
            micro_controller.bus_matrix.get_insn(RAM0_BEGIN + 2),
            BImm11 { imm: -12 }
        );
        assert_eq!(micro_controller.program_counter, RAM0_BEGIN + 2 + 4 - 12);
    }

    #[test]
    #[should_panic(expected = "B instruction inside an IT block at 0x20000002")]
    fn b_imm11_in_it_block() {
        let mut micro_controller = MicroController::default();
        micro_controller
            .bus_matrix
            .set32(RAM0_BEGIN + 2, 0b11100_11111111010);
        micro_controller.program_counter = RAM0_BEGIN + 2;
        micro_controller.if_then_state = 0x01;

        micro_controller.emulate_one_insn();
    }

    #[test]
    fn bl_plus_imm24() {
        let mut micro_controller = MicroController::default();
        micro_controller
            .bus_matrix
            .set32(RAM0_BEGIN + 2, 0b11_1_1_1_00000000110_111100_0000110000);
        micro_controller.program_counter = RAM0_BEGIN + 2;

        micro_controller.emulate_one_insn();

        assert_eq!(
            micro_controller.bus_matrix.get_insn(RAM0_BEGIN + 2),
            BlPlusImm24 { imm16: 12, imm8: 3 }
        );
        assert_eq!(
            micro_controller.registers[LINK_REGISTER],
            RAM0_BEGIN + 2 + 4 + 1
        );
        assert_eq!(
            micro_controller.program_counter,
            RAM0_BEGIN + 2 + 4 + (3 << 16) + 12
        );
    }

    #[test]
    #[should_panic(expected = "BL instruction inside an IT block at 0x20000002")]
    fn bl_plus_imm24_in_it_block() {
        let mut micro_controller = MicroController::default();
        micro_controller
            .bus_matrix
            .set32(RAM0_BEGIN + 2, 0b11_1_1_1_00000000110_111100_0000110000);
        micro_controller.program_counter = RAM0_BEGIN + 2;
        micro_controller.if_then_state = 0x01;

        micro_controller.emulate_one_insn();
    }

    #[test]
    fn bl_minus_imm24() {
        let mut micro_controller = MicroController::default();
        micro_controller
            .bus_matrix
            .set32(RAM0_BEGIN + 2, 0b11_1_1_1_11111111010_111101_1111001111);
        micro_controller.program_counter = RAM0_BEGIN + 2;

        micro_controller.emulate_one_insn();

        assert_eq!(
            micro_controller.bus_matrix.get_insn(RAM0_BEGIN + 2),
            BlMinusImm24 {
                imm16: 12 - 1,
                imm8: 3
            }
        );
        assert_eq!(
            micro_controller.registers[LINK_REGISTER],
            RAM0_BEGIN + 2 + 4 + 1
        );
        assert_eq!(
            micro_controller.program_counter,
            RAM0_BEGIN + 2 + 4 - (3 << 16) - 12
        );
    }

    #[test]
    #[should_panic(expected = "BL instruction inside an IT block at 0x20000002")]
    fn bl_minus_imm24_in_it_block() {
        let mut micro_controller = MicroController::default();
        micro_controller
            .bus_matrix
            .set32(RAM0_BEGIN + 2, 0b11_1_1_1_11111111010_111101_1111001111);
        micro_controller.program_counter = RAM0_BEGIN + 2;
        micro_controller.if_then_state = 0x01;

        micro_controller.emulate_one_insn();
    }

    #[test]
    fn blx_rm() {
        let mut micro_controller = MicroController::default();
        micro_controller
            .bus_matrix
            .set32(RAM0_BEGIN + 2, 0b010001111_0011_000);
        micro_controller.registers[3] = 123456 | 1;
        micro_controller.program_counter = RAM0_BEGIN + 2;

        micro_controller.emulate_one_insn();

        assert_eq!(
            micro_controller.bus_matrix.get_insn(RAM0_BEGIN + 2),
            BlxRm { rm: 3 }
        );
        assert_eq!(
            micro_controller.registers[LINK_REGISTER],
            RAM0_BEGIN + 2 + 2 + 1
        );
        assert_eq!(micro_controller.program_counter, 123456);
    }

    #[test]
    #[should_panic(expected = "BLX instruction inside an IT block at 0x20000002")]
    fn blx_rm_in_it_block() {
        let mut micro_controller = MicroController::default();
        micro_controller
            .bus_matrix
            .set32(RAM0_BEGIN + 2, 0b010001111_0011_000);
        micro_controller.registers[3] = 123456 | 1;
        micro_controller.program_counter = RAM0_BEGIN + 2;
        micro_controller.if_then_state = 0x01;

        micro_controller.emulate_one_insn();
    }

    #[test]
    #[should_panic(expected = "Bad BLX interworking address at 0x20000002")]
    fn blx_rm_bad_interworking_address() {
        let mut micro_controller = MicroController::default();
        micro_controller
            .bus_matrix
            .set32(RAM0_BEGIN + 2, 0b010001111_0011_000);
        micro_controller.registers[3] = 123456;
        micro_controller.program_counter = RAM0_BEGIN + 2;

        micro_controller.emulate_one_insn();
    }

    #[test]
    fn bx_rm() {
        let mut micro_controller = MicroController::default();
        micro_controller
            .bus_matrix
            .set32(RAM0_BEGIN + 2, 0b010001110_0011_000);
        micro_controller.registers[3] = 123456 | 1;
        micro_controller.registers[LINK_REGISTER] = 1234 | 1;
        micro_controller.program_counter = RAM0_BEGIN + 2;

        micro_controller.emulate_one_insn();

        assert_eq!(
            micro_controller.bus_matrix.get_insn(RAM0_BEGIN + 2),
            BxRm { rm: 3 }
        );
        assert_eq!(micro_controller.registers[LINK_REGISTER], 1234 | 1);
        assert_eq!(micro_controller.program_counter, 123456);
    }

    #[test]
    #[should_panic(expected = "BX instruction inside an IT block at 0x20000002")]
    fn bx_rm_in_it_block() {
        let mut micro_controller = MicroController::default();
        micro_controller
            .bus_matrix
            .set32(RAM0_BEGIN + 2, 0b010001110_0011_000);
        micro_controller.registers[3] = 123456 | 1;
        micro_controller.registers[LINK_REGISTER] = 1234 | 1;
        micro_controller.program_counter = RAM0_BEGIN + 2;
        micro_controller.if_then_state = 0x01;

        micro_controller.emulate_one_insn();
    }

    #[test]
    #[should_panic(expected = "Bad BX interworking address at 0x20000002")]
    fn bx_rm_bad_interworking_address() {
        let mut micro_controller = MicroController::default();
        micro_controller
            .bus_matrix
            .set32(RAM0_BEGIN + 2, 0b010001110_0011_000);
        micro_controller.registers[3] = 123456;
        micro_controller.registers[LINK_REGISTER] = 1234 | 1;
        micro_controller.program_counter = RAM0_BEGIN + 2;

        micro_controller.emulate_one_insn();
    }

    #[test]
    fn cbz_rm_imm6_zero() {
        let mut micro_controller = MicroController::default();
        micro_controller
            .bus_matrix
            .set32(RAM0_BEGIN + 2, 0b101100_0_1_00110_011);
        micro_controller.registers[3] = 0;
        micro_controller.program_counter = RAM0_BEGIN + 2;

        micro_controller.emulate_one_insn();

        assert_eq!(
            micro_controller.bus_matrix.get_insn(RAM0_BEGIN + 2),
            CbzRnImm6 { rn: 3, imm: 12 }
        );
        assert_eq!(micro_controller.program_counter, RAM0_BEGIN + 2 + 4 + 12);
    }

    #[test]
    fn cbz_rm_imm6_nonzero() {
        let mut micro_controller = MicroController::default();
        micro_controller
            .bus_matrix
            .set32(RAM0_BEGIN + 2, 0b101100_0_1_00110_011);
        micro_controller.registers[3] = 1;
        micro_controller.program_counter = RAM0_BEGIN + 2;

        micro_controller.emulate_one_insn();

        assert_eq!(
            micro_controller.bus_matrix.get_insn(RAM0_BEGIN + 2),
            CbzRnImm6 { rn: 3, imm: 12 }
        );
        assert_eq!(micro_controller.program_counter, RAM0_BEGIN + 2 + 2);
    }

    #[test]
    #[should_panic(expected = "CBZ instruction inside an IT block at 0x20000002")]
    fn cbz_rm_imm6_in_it_block() {
        let mut micro_controller = MicroController::default();
        micro_controller
            .bus_matrix
            .set32(RAM0_BEGIN + 2, 0b101100_0_1_00110_011);
        micro_controller.program_counter = RAM0_BEGIN + 2;
        micro_controller.if_then_state = 0x01;

        micro_controller.emulate_one_insn();
    }

    #[test]
    fn cmp_rn_imm8() {
        let mut micro_controller = MicroController::default();
        micro_controller
            .bus_matrix
            .set32(RAM0_BEGIN, 0b00101_011_00001100);
        micro_controller.registers[3] = 11;
        micro_controller.program_counter = RAM0_BEGIN;

        micro_controller.emulate_one_insn();

        assert_eq!(
            micro_controller.bus_matrix.get_insn(RAM0_BEGIN),
            CmpRnImm8 { rn: 3, imm8: 12 }
        );
        assert_eq!(micro_controller.program_counter, RAM0_BEGIN + 2);
        assert_eq!(micro_controller.last_comparison, std::cmp::Ordering::Less);
    }

    #[test]
    fn cmp_rn_rm() {
        let mut micro_controller = MicroController::default();
        micro_controller
            .bus_matrix
            .set32(RAM0_BEGIN, 0b0100001010_111_011);
        micro_controller.registers[3] = 11;
        micro_controller.registers[7] = 12;
        micro_controller.program_counter = RAM0_BEGIN;

        micro_controller.emulate_one_insn();

        assert_eq!(
            micro_controller.bus_matrix.get_insn(RAM0_BEGIN),
            CmpRnRm { rn: 3, rm: 7 }
        );
        assert_eq!(micro_controller.program_counter, RAM0_BEGIN + 2);
        assert_eq!(micro_controller.last_comparison, std::cmp::Ordering::Less);
    }

    #[test]
    fn it() {
        let mut micro_controller = MicroController::default();
        micro_controller
            .bus_matrix
            .set16(RAM0_BEGIN, 0b00101_000_00000000);
        micro_controller
            .bus_matrix
            .set32(RAM0_BEGIN + 2, 0b10111111_1001_0110);
        micro_controller.registers[3] = 11;
        micro_controller.registers[7] = 12;
        micro_controller.program_counter = RAM0_BEGIN + 2;

        micro_controller.emulate_one_insn();

        assert_eq!(
            micro_controller.bus_matrix.get_insn(RAM0_BEGIN + 2),
            It {
                first_cond_and_mask: 0b1001_0110
            }
        );
        assert_eq!(micro_controller.program_counter, RAM0_BEGIN + 4);
        assert_eq!(micro_controller.if_then_state, 0b1001_0110);
    }

    #[test]
    #[should_panic(expected = "IT instructions not preceded by CMP instruction at 0x20000002")]
    fn it_not_after_cmp() {
        let mut micro_controller = MicroController::default();
        micro_controller
            .bus_matrix
            .set32(RAM0_BEGIN + 2, 0b10111111_1001_0110);
        micro_controller.program_counter = RAM0_BEGIN + 2;

        micro_controller.emulate_one_insn();
    }

    #[test]
    #[should_panic(expected = "IT instruction inside an IT block at 0x20000002")]
    fn it_in_it_block() {
        let mut micro_controller = MicroController::default();
        micro_controller
            .bus_matrix
            .set16(RAM0_BEGIN, 0b00101_000_00000000);
        micro_controller
            .bus_matrix
            .set32(RAM0_BEGIN + 2, 0b10111111_1001_0110);
        micro_controller.program_counter = RAM0_BEGIN + 2;
        micro_controller.if_then_state = 0x01;

        micro_controller.emulate_one_insn();
    }

    #[test]
    fn ldrb_rt_rn_imm5() {
        let mut micro_controller = MicroController::default();
        micro_controller
            .bus_matrix
            .set32(RAM0_BEGIN, 0b01111_10100_111_011);
        micro_controller.bus_matrix.set8(RAM0_BEGIN + 256 + 20, 137);
        micro_controller.registers[7] = RAM0_BEGIN + 256;
        micro_controller.program_counter = RAM0_BEGIN;

        micro_controller.emulate_one_insn();

        assert_eq!(
            micro_controller.bus_matrix.get_insn(RAM0_BEGIN),
            LdrbRtRnImm5 {
                rt: 3,
                rn: 7,
                imm5: 20
            }
        );
        assert_eq!(micro_controller.registers[3], 137);
        assert_eq!(micro_controller.program_counter, RAM0_BEGIN + 2);
    }

    #[test]
    fn ldrh_rt_rn_imm5() {
        let mut micro_controller = MicroController::default();
        micro_controller
            .bus_matrix
            .set32(RAM0_BEGIN, 0b10001_01010_111_011);
        micro_controller
            .bus_matrix
            .set16(RAM0_BEGIN + 256 + 20, 13579);
        micro_controller.registers[7] = RAM0_BEGIN + 256;
        micro_controller.program_counter = RAM0_BEGIN;

        micro_controller.emulate_one_insn();

        assert_eq!(
            micro_controller.bus_matrix.get_insn(RAM0_BEGIN),
            LdrhRtRnImm5 {
                rt: 3,
                rn: 7,
                imm: 20
            }
        );
        assert_eq!(micro_controller.registers[3], 13579);
        assert_eq!(micro_controller.program_counter, RAM0_BEGIN + 2);
    }

    #[test]
    fn ldr_rt_pc_imm8() {
        let mut micro_controller = MicroController::default();
        micro_controller
            .bus_matrix
            .set32(RAM0_BEGIN + 2, 0b01001_011_00000101);
        micro_controller
            .bus_matrix
            .set32(RAM0_BEGIN + 2 + 2 + 20, 123456);
        micro_controller.program_counter = RAM0_BEGIN + 2;

        micro_controller.emulate_one_insn();

        assert_eq!(
            micro_controller.bus_matrix.get_insn(RAM0_BEGIN + 2),
            LdrRtPcImm8 { rt: 3, imm: 20 }
        );
        assert_eq!(micro_controller.registers[3], 123456);
        assert_eq!(micro_controller.program_counter, RAM0_BEGIN + 2 + 2);
    }

    #[test]
    fn ldr_rt_rn_imm5() {
        let mut micro_controller = MicroController::default();
        micro_controller
            .bus_matrix
            .set32(RAM0_BEGIN, 0b01101_00101_111_011);
        micro_controller
            .bus_matrix
            .set32(RAM0_BEGIN + 256 + 20, 123456789);
        micro_controller.registers[7] = RAM0_BEGIN + 256;
        micro_controller.program_counter = RAM0_BEGIN;

        micro_controller.emulate_one_insn();

        assert_eq!(
            micro_controller.bus_matrix.get_insn(RAM0_BEGIN),
            LdrRtRnImm5 {
                rt: 3,
                rn: 7,
                imm: 20
            }
        );
        assert_eq!(micro_controller.registers[3], 123456789);
        assert_eq!(micro_controller.program_counter, RAM0_BEGIN + 2);
    }

    #[test]
    fn ldr_rt_sp_imm8() {
        let mut micro_controller = MicroController::default();
        micro_controller
            .bus_matrix
            .set32(RAM0_BEGIN, 0b10011_011_00000101);
        micro_controller
            .bus_matrix
            .set32(RAM0_BEGIN + 256 + 20, 123456789);
        micro_controller.registers[STACK_POINTER] = RAM0_BEGIN + 256;
        micro_controller.program_counter = RAM0_BEGIN;

        micro_controller.emulate_one_insn();

        assert_eq!(
            micro_controller.bus_matrix.get_insn(RAM0_BEGIN),
            LdrRtSpImm8 { rt: 3, imm: 20 }
        );
        assert_eq!(micro_controller.registers[3], 123456789);
        assert_eq!(micro_controller.program_counter, RAM0_BEGIN + 2);
    }

    #[test]
    fn lsl_rdn_rm() {
        let mut micro_controller = MicroController::default();
        micro_controller
            .bus_matrix
            .set32(RAM0_BEGIN, 0b0100000010_011_001);
        micro_controller.registers[1] = 123456789;
        micro_controller.registers[3] = 0x0FF07;
        micro_controller.program_counter = RAM0_BEGIN;

        micro_controller.emulate_one_insn();

        assert_eq!(
            micro_controller.bus_matrix.get_insn(RAM0_BEGIN),
            LslRdnRm { rdn: 1, rm: 3 }
        );
        assert_eq!(micro_controller.registers[1], 123456789 << 7);
        assert_eq!(micro_controller.registers[3], 0x0FF07);
        assert_eq!(micro_controller.program_counter, RAM0_BEGIN + 2);
    }

    #[test]
    fn lsl_rd_rm_imm5() {
        let mut micro_controller = MicroController::default();
        micro_controller
            .bus_matrix
            .set32(RAM0_BEGIN, 0b00000_01001_111_001);
        micro_controller.registers[7] = 123456789;
        micro_controller.program_counter = RAM0_BEGIN;

        micro_controller.emulate_one_insn();

        assert_eq!(
            micro_controller.bus_matrix.get_insn(RAM0_BEGIN),
            LslRdRmImm5 {
                rd: 1,
                rm: 7,
                imm5: 9
            }
        );
        assert_eq!(micro_controller.registers[1], 123456789 << 9);
        assert_eq!(micro_controller.registers[7], 123456789);
        assert_eq!(micro_controller.program_counter, RAM0_BEGIN + 2);
    }

    #[test]
    fn lsr_rdn_rm() {
        let mut micro_controller = MicroController::default();
        micro_controller
            .bus_matrix
            .set32(RAM0_BEGIN, 0b0100000011_011_001);
        micro_controller.registers[1] = 123456789;
        micro_controller.registers[3] = 0x0FF07;
        micro_controller.program_counter = RAM0_BEGIN;

        micro_controller.emulate_one_insn();

        assert_eq!(
            micro_controller.bus_matrix.get_insn(RAM0_BEGIN),
            LsrRdnRm { rdn: 1, rm: 3 }
        );
        assert_eq!(micro_controller.registers[1], 123456789 >> 7);
        assert_eq!(micro_controller.registers[3], 0x0FF07);
        assert_eq!(micro_controller.program_counter, RAM0_BEGIN + 2);
    }

    #[test]
    fn lsr_rd_rm_imm5() {
        let mut micro_controller = MicroController::default();
        micro_controller
            .bus_matrix
            .set32(RAM0_BEGIN, 0b00001_01001_111_001);
        micro_controller.registers[7] = 123456789;
        micro_controller.program_counter = RAM0_BEGIN;

        micro_controller.emulate_one_insn();

        assert_eq!(
            micro_controller.bus_matrix.get_insn(RAM0_BEGIN),
            LsrRdRmImm5 {
                rd: 1,
                rm: 7,
                imm: 9
            }
        );
        assert_eq!(micro_controller.registers[1], 123456789 >> 9);
        assert_eq!(micro_controller.registers[7], 123456789);
        assert_eq!(micro_controller.program_counter, RAM0_BEGIN + 2);
    }

    #[test]
    fn lsr_rd_rm_imm5_shift32() {
        let mut micro_controller = MicroController::default();
        micro_controller
            .bus_matrix
            .set32(RAM0_BEGIN, 0b00001_00000_111_001);
        micro_controller.registers[7] = 123456789;
        micro_controller.program_counter = RAM0_BEGIN;

        micro_controller.emulate_one_insn();

        assert_eq!(
            micro_controller.bus_matrix.get_insn(RAM0_BEGIN),
            LsrRdRmImm5 {
                rd: 1,
                rm: 7,
                imm: 32
            }
        );
        assert_eq!(micro_controller.registers[1], 0);
        assert_eq!(micro_controller.registers[7], 123456789);
        assert_eq!(micro_controller.program_counter, RAM0_BEGIN + 2);
    }

    #[test]
    fn mov_rd_imm8() {
        let mut micro_controller = MicroController::default();
        micro_controller
            .bus_matrix
            .set32(RAM0_BEGIN, 0b00100_001_11101010);
        micro_controller.registers[1] = 0xFFFFFFFF;
        micro_controller.program_counter = RAM0_BEGIN;

        micro_controller.emulate_one_insn();

        assert_eq!(
            micro_controller.bus_matrix.get_insn(RAM0_BEGIN),
            MovRdImm8 { rd: 1, imm8: 234 }
        );
        assert_eq!(micro_controller.registers[1], 234);
        assert_eq!(micro_controller.program_counter, RAM0_BEGIN + 2);
    }

    #[test]
    fn mov_rd_rm() {
        let mut micro_controller = MicroController::default();
        micro_controller
            .bus_matrix
            .set32(RAM0_BEGIN, 0b01000110_0_0011_001);
        micro_controller.registers[1] = 0xFFFFFFFF;
        micro_controller.registers[3] = 0xCAFEBABE;
        micro_controller.program_counter = RAM0_BEGIN;

        micro_controller.emulate_one_insn();

        assert_eq!(
            micro_controller.bus_matrix.get_insn(RAM0_BEGIN),
            MovRdRm { rd: 1, rm: 3 }
        );
        assert_eq!(micro_controller.registers[1], 0xCAFEBABE);
        assert_eq!(micro_controller.registers[3], 0xCAFEBABE);
        assert_eq!(micro_controller.program_counter, RAM0_BEGIN + 2);
    }

    #[test]
    fn mov_rd_rm_with_program_counter() {
        let mut micro_controller = MicroController::default();
        micro_controller
            .bus_matrix
            .set32(RAM0_BEGIN, 0b01000110_1_0011_111);
        micro_controller.registers[3] = 0xDEADBEEF;
        micro_controller.program_counter = RAM0_BEGIN;

        micro_controller.emulate_one_insn();

        assert_eq!(
            micro_controller.bus_matrix.get_insn(RAM0_BEGIN),
            MovRdRm { rd: 15, rm: 3 }
        );
        assert_eq!(micro_controller.program_counter, 0xDEADBEEE);
    }

    #[test]
    #[should_panic(expected = "MOV PC instruction inside an IT block at 0x20000000")]
    fn mov_rd_rm_with_program_counter_in_it_block() {
        let mut micro_controller = MicroController::default();
        micro_controller
            .bus_matrix
            .set32(RAM0_BEGIN, 0b01000110_1_0011_111);
        micro_controller.registers[3] = 0xDEADBEEF;
        micro_controller.program_counter = RAM0_BEGIN;
        micro_controller.if_then_state = 0x01;

        micro_controller.emulate_one_insn();
    }

    #[test]
    fn movt_rd_imm16() {
        let mut micro_controller = MicroController::default();
        micro_controller
            .bus_matrix
            .set32(RAM0_BEGIN, 0b0_101_0001_10110110_11110_1_101100_1100);
        micro_controller.registers[1] = 0xCAFEBABE;
        micro_controller.program_counter = RAM0_BEGIN;

        micro_controller.emulate_one_insn();

        assert_eq!(
            micro_controller.bus_matrix.get_insn(RAM0_BEGIN),
            MovtRdImm16 {
                rd: 1,
                imm16: 0b1100_1_101_10110110
            }
        );
        assert_eq!(
            micro_controller.registers[1],
            0b1100_1_101_10110110 << 16 | 0xBABE
        );
        assert_eq!(micro_controller.program_counter, RAM0_BEGIN + 4);
    }

    #[test]
    fn movw_rd_imm16() {
        let mut micro_controller = MicroController::default();
        micro_controller
            .bus_matrix
            .set32(RAM0_BEGIN, 0b0_101_0001_10110110_11110_1_100100_1100);
        micro_controller.registers[1] = 0xFFFFFFFF;
        micro_controller.program_counter = RAM0_BEGIN;

        micro_controller.emulate_one_insn();

        assert_eq!(
            micro_controller.bus_matrix.get_insn(RAM0_BEGIN),
            MovwRdImm16 {
                rd: 1,
                imm16: 0b1100_1_101_10110110
            }
        );
        assert_eq!(micro_controller.registers[1], 0b1100_1_101_10110110);
        assert_eq!(micro_controller.program_counter, RAM0_BEGIN + 4);
    }

    #[test]
    fn mul_rdm_rn() {
        let mut micro_controller = MicroController::default();
        micro_controller
            .bus_matrix
            .set32(RAM0_BEGIN, 0b0100001101_011_001);
        micro_controller.registers[1] = 123456789;
        micro_controller.registers[3] = 257;
        micro_controller.program_counter = RAM0_BEGIN;

        micro_controller.emulate_one_insn();

        assert_eq!(
            micro_controller.bus_matrix.get_insn(RAM0_BEGIN),
            MulRdmRn { rdm: 1, rn: 3 }
        );
        assert_eq!(
            micro_controller.registers[1],
            123456789u32.wrapping_mul(257)
        );
        assert_eq!(micro_controller.registers[3], 257);
        assert_eq!(micro_controller.program_counter, RAM0_BEGIN + 2);
    }

    #[test]
    fn orr_rdn_rm() {
        let mut micro_controller = MicroController::default();
        micro_controller
            .bus_matrix
            .set32(RAM0_BEGIN, 0b0100001100_011_001);
        micro_controller.registers[1] = 0b1100;
        micro_controller.registers[3] = 0b0101;
        micro_controller.program_counter = RAM0_BEGIN;

        micro_controller.emulate_one_insn();

        assert_eq!(
            micro_controller.bus_matrix.get_insn(RAM0_BEGIN),
            OrrRdnRm { rdn: 1, rm: 3 }
        );
        assert_eq!(micro_controller.registers[1], 0b1101);
        assert_eq!(micro_controller.registers[3], 0b0101);
        assert_eq!(micro_controller.program_counter, RAM0_BEGIN + 2);
    }

    #[test]
    fn pop_registers_only() {
        let mut micro_controller = MicroController::default();
        micro_controller
            .bus_matrix
            .set32(RAM0_BEGIN, 0b1011110_0_00001010);
        micro_controller.bus_matrix.set32(RAM0_END + 4, 5678);
        micro_controller.bus_matrix.set32(RAM0_END, 1234);
        micro_controller.registers[STACK_POINTER] = RAM0_END;
        micro_controller.program_counter = RAM0_BEGIN;

        micro_controller.emulate_one_insn();

        assert_eq!(
            micro_controller.bus_matrix.get_insn(RAM0_BEGIN),
            Pop {
                registers: 0b1010,
                pc: false
            }
        );
        assert_eq!(micro_controller.registers[1], 1234);
        assert_eq!(micro_controller.registers[3], 5678);
        assert_eq!(micro_controller.registers[STACK_POINTER], RAM0_END + 8);
        assert_eq!(micro_controller.program_counter, RAM0_BEGIN + 2);
    }

    #[test]
    fn pop_with_program_counter() {
        let mut micro_controller = MicroController::default();
        micro_controller
            .bus_matrix
            .set32(RAM0_BEGIN, 0b1011110_1_00001010);
        micro_controller
            .bus_matrix
            .set32(RAM0_END + 8, 12345678 | 1);
        micro_controller.bus_matrix.set32(RAM0_END + 4, 5678);
        micro_controller.bus_matrix.set32(RAM0_END, 1234);
        micro_controller.registers[STACK_POINTER] = RAM0_END;
        micro_controller.program_counter = RAM0_BEGIN;

        micro_controller.emulate_one_insn();

        assert_eq!(
            micro_controller.bus_matrix.get_insn(RAM0_BEGIN),
            Pop {
                registers: 0b1010,
                pc: true
            }
        );
        assert_eq!(micro_controller.registers[1], 1234);
        assert_eq!(micro_controller.registers[3], 5678);
        assert_eq!(micro_controller.registers[STACK_POINTER], RAM0_END + 12);
        assert_eq!(micro_controller.program_counter, 12345678);
    }

    #[test]
    #[should_panic(expected = "POP PC instruction inside an IT block at 0x20000000")]
    fn pop_with_program_counter_in_it_block() {
        let mut micro_controller = MicroController::default();
        micro_controller
            .bus_matrix
            .set32(RAM0_BEGIN, 0b1011110_1_00001010);
        micro_controller.registers[STACK_POINTER] = RAM0_END;
        micro_controller.program_counter = RAM0_BEGIN;
        micro_controller.if_then_state = 0x01;

        micro_controller.emulate_one_insn();
    }

    #[test]
    #[should_panic(expected = "Bad POP PC interworking address at 0x20000000")]
    fn pop_with_program_counter_bad_interworking_address() {
        let mut micro_controller = MicroController::default();
        micro_controller
            .bus_matrix
            .set32(RAM0_BEGIN, 0b1011110_1_00000000);
        micro_controller.bus_matrix.set32(RAM0_END + 8, 12345678);
        micro_controller.registers[STACK_POINTER] = RAM0_END;
        micro_controller.program_counter = RAM0_BEGIN;

        micro_controller.emulate_one_insn();
    }

    #[test]
    fn push() {
        let mut micro_controller = MicroController::default();
        micro_controller
            .bus_matrix
            .set32(RAM0_BEGIN, 0b1011010_1_00001010);
        micro_controller.registers[1] = 1234;
        micro_controller.registers[3] = 5678;
        micro_controller.registers[LINK_REGISTER] = 12345678;
        micro_controller.registers[STACK_POINTER] = RAM0_END + 12;
        micro_controller.program_counter = RAM0_BEGIN;

        micro_controller.emulate_one_insn();

        assert_eq!(
            micro_controller.bus_matrix.get_insn(RAM0_BEGIN),
            Push {
                registers: 0b1010,
                lr: true
            }
        );
        assert_eq!(micro_controller.bus_matrix.get32(RAM0_END + 8), 12345678);
        assert_eq!(micro_controller.bus_matrix.get32(RAM0_END + 4), 5678);
        assert_eq!(micro_controller.bus_matrix.get32(RAM0_END), 1234);
        assert_eq!(micro_controller.registers[STACK_POINTER], RAM0_END);
        assert_eq!(micro_controller.program_counter, RAM0_BEGIN + 2);
    }

    #[test]
    fn strb_rt_rn_imm5() {
        let mut micro_controller = MicroController::default();
        micro_controller
            .bus_matrix
            .set32(RAM0_BEGIN, 0b01110_10100_111_011);
        micro_controller.registers[3] = 137;
        micro_controller.registers[7] = RAM0_BEGIN + 256;
        micro_controller.program_counter = RAM0_BEGIN;

        micro_controller.emulate_one_insn();

        assert_eq!(
            micro_controller.bus_matrix.get_insn(RAM0_BEGIN),
            StrbRtRnImm5 {
                rt: 3,
                rn: 7,
                imm5: 20
            }
        );
        assert_eq!(micro_controller.bus_matrix.get8(RAM0_BEGIN + 256 + 20), 137);
        assert_eq!(micro_controller.program_counter, RAM0_BEGIN + 2);
    }

    #[test]
    fn strh_rt_rn_imm5() {
        let mut micro_controller = MicroController::default();
        micro_controller
            .bus_matrix
            .set32(RAM0_BEGIN, 0b10000_01010_111_011);
        micro_controller.registers[3] = 13579;
        micro_controller.registers[7] = RAM0_BEGIN + 256;
        micro_controller.program_counter = RAM0_BEGIN;

        micro_controller.emulate_one_insn();

        assert_eq!(
            micro_controller.bus_matrix.get_insn(RAM0_BEGIN),
            StrhRtRnImm5 {
                rt: 3,
                rn: 7,
                imm: 20
            }
        );
        assert_eq!(
            micro_controller.bus_matrix.get16(RAM0_BEGIN + 256 + 20),
            13579
        );
        assert_eq!(micro_controller.program_counter, RAM0_BEGIN + 2);
    }

    #[test]
    fn str_rt_rn_imm5() {
        let mut micro_controller = MicroController::default();
        micro_controller
            .bus_matrix
            .set32(RAM0_BEGIN, 0b01100_00101_111_011);
        micro_controller.registers[3] = 123456789;
        micro_controller.registers[7] = RAM0_BEGIN + 256;
        micro_controller.program_counter = RAM0_BEGIN;

        micro_controller.emulate_one_insn();

        assert_eq!(
            micro_controller.bus_matrix.get_insn(RAM0_BEGIN),
            StrRtRnImm5 {
                rt: 3,
                rn: 7,
                imm: 20
            }
        );
        assert_eq!(
            micro_controller.bus_matrix.get32(RAM0_BEGIN + 256 + 20),
            123456789
        );
        assert_eq!(micro_controller.program_counter, RAM0_BEGIN + 2);
    }

    #[test]
    fn str_rt_sp_imm8() {
        let mut micro_controller = MicroController::default();
        micro_controller
            .bus_matrix
            .set32(RAM0_BEGIN, 0b10010_011_00000101);
        micro_controller.registers[3] = 123456789;
        micro_controller.registers[STACK_POINTER] = RAM0_BEGIN + 256;
        micro_controller.program_counter = RAM0_BEGIN;

        micro_controller.emulate_one_insn();

        assert_eq!(
            micro_controller.bus_matrix.get_insn(RAM0_BEGIN),
            StrRtSpImm8 { rt: 3, imm: 20 }
        );
        assert_eq!(
            micro_controller.bus_matrix.get32(RAM0_BEGIN + 256 + 20),
            123456789
        );
        assert_eq!(micro_controller.program_counter, RAM0_BEGIN + 2);
    }

    #[test]
    fn sub_rdn_imm8() {
        let mut micro_controller = MicroController::default();
        micro_controller
            .bus_matrix
            .set32(RAM0_BEGIN, 0b00111_010_00001111);
        micro_controller.registers[2] = 4;
        micro_controller.program_counter = RAM0_BEGIN;

        micro_controller.emulate_one_insn();

        assert_eq!(
            micro_controller.bus_matrix.get_insn(RAM0_BEGIN),
            SubRdnImm8 { rdn: 2, imm8: 15 }
        );
        assert_eq!(micro_controller.registers[2], u32::MAX - 15 + 4 + 1);
        assert_eq!(micro_controller.program_counter, RAM0_BEGIN + 2);
    }

    #[test]
    fn sub_rd_rn_rm() {
        let mut micro_controller = MicroController::default();
        micro_controller
            .bus_matrix
            .set32(RAM0_BEGIN, 0b0001101_111_110_001);
        micro_controller.registers[1] = 1;
        micro_controller.registers[6] = 4;
        micro_controller.registers[7] = 15;
        micro_controller.program_counter = RAM0_BEGIN;

        micro_controller.emulate_one_insn();

        assert_eq!(
            micro_controller.bus_matrix.get_insn(RAM0_BEGIN),
            SubRdRnRm {
                rd: 1,
                rn: 6,
                rm: 7
            }
        );
        assert_eq!(micro_controller.registers[1], u32::MAX - 15 + 4 + 1);
        assert_eq!(micro_controller.registers[6], 4);
        assert_eq!(micro_controller.registers[7], 15);
        assert_eq!(micro_controller.program_counter, RAM0_BEGIN + 2);
    }

    #[test]
    fn sub_sp_sp_imm7() {
        let mut micro_controller = MicroController::default();
        micro_controller
            .bus_matrix
            .set32(RAM0_BEGIN, 0b101100001_0000011);
        micro_controller.registers[STACK_POINTER] = RAM0_END;
        micro_controller.program_counter = RAM0_BEGIN;

        micro_controller.emulate_one_insn();

        assert_eq!(
            micro_controller.bus_matrix.get_insn(RAM0_BEGIN),
            SubSpSpImm7 { imm: 12 }
        );
        assert_eq!(micro_controller.registers[STACK_POINTER], RAM0_END - 12);
        assert_eq!(micro_controller.program_counter, RAM0_BEGIN + 2);
    }

    #[test]
    fn tbb_rn_rm() {
        let mut micro_controller = MicroController::default();
        micro_controller
            .bus_matrix
            .set32(RAM0_BEGIN, 0b111100000000_0111_111010001101_0001);
        micro_controller.bus_matrix.set8(RAM0_END + 15, 234);
        micro_controller.registers[1] = RAM0_END;
        micro_controller.registers[7] = 15;
        micro_controller.program_counter = RAM0_BEGIN;

        micro_controller.emulate_one_insn();

        assert_eq!(
            micro_controller.bus_matrix.get_insn(RAM0_BEGIN),
            TbbRnRm { rn: 1, rm: 7 }
        );
        assert_eq!(micro_controller.program_counter, RAM0_BEGIN + 4 + 234 * 2);
    }

    #[test]
    fn tbb_rn_rm_with_program_counter() {
        let mut micro_controller = MicroController::default();
        micro_controller
            .bus_matrix
            .set32(RAM0_BEGIN, 0b111100000000_0111_111010001101_1111);
        micro_controller.bus_matrix.set16(RAM0_BEGIN + 4 + 15, 234);
        micro_controller.registers[7] = 15;
        micro_controller.program_counter = RAM0_BEGIN;

        micro_controller.emulate_one_insn();

        assert_eq!(
            micro_controller.bus_matrix.get_insn(RAM0_BEGIN),
            TbbRnRm { rn: 15, rm: 7 }
        );
        assert_eq!(micro_controller.program_counter, RAM0_BEGIN + 4 + 234 * 2);
    }

    #[test]
    #[should_panic(expected = "TBB instruction inside an IT block")]
    fn tbb_rn_rm_in_it_block() {
        let mut micro_controller = MicroController::default();
        micro_controller
            .bus_matrix
            .set32(RAM0_BEGIN, 0b111100000000_0111_111010001101_0001);
        micro_controller.program_counter = RAM0_BEGIN;
        micro_controller.if_then_state = 0x01;

        micro_controller.emulate_one_insn();
    }

    #[test]
    fn udiv_rd_rn_rm() {
        let mut micro_controller = MicroController::default();
        micro_controller
            .bus_matrix
            .set32(RAM0_BEGIN, 0b1111_0001_1111_0111_111110111011_0110);
        micro_controller.registers[1] = 1;
        micro_controller.registers[6] = 31;
        micro_controller.registers[7] = 3;
        micro_controller.program_counter = RAM0_BEGIN;

        micro_controller.emulate_one_insn();

        assert_eq!(
            micro_controller.bus_matrix.get_insn(RAM0_BEGIN),
            UdivRdRnRm {
                rd: 1,
                rn: 6,
                rm: 7
            }
        );
        assert_eq!(micro_controller.registers[1], 10);
        assert_eq!(micro_controller.registers[6], 31);
        assert_eq!(micro_controller.registers[7], 3);
        assert_eq!(micro_controller.program_counter, RAM0_BEGIN + 4);
    }

    #[test]
    fn u32_insn_at_end_of_memory_range() {
        let mut micro_controller = MicroController::default();
        micro_controller
            .bus_matrix
            .set16(RAM0_BEGIN, 0b0_000_0100_00000111);
        micro_controller
            .bus_matrix
            .set16(RAM0_END - 2, 0b11110_0_100100_0000);
        micro_controller
            .bus_matrix
            .set16(RAM0_END, 0b0_001_0011_00000111);

        micro_controller.program_counter = RAM0_END - 2 - RAM0_BYTES;
        micro_controller.emulate_one_insn();
        micro_controller.program_counter = RAM0_END - 2;
        micro_controller.emulate_one_insn();

        assert_eq!(
            decode_insn(micro_controller.bus_matrix.get32(RAM0_END - 2 - RAM0_BYTES)),
            MovwRdImm16 { rd: 4, imm16: 7 }
        );
        assert_eq!(
            decode_insn(micro_controller.bus_matrix.get32(RAM0_END - 2)),
            MovwRdImm16 { rd: 3, imm16: 263 }
        );
        assert_eq!(micro_controller.registers[3], 263);
        assert_eq!(micro_controller.registers[4], 7);
    }

    #[test]
    fn if_eq_then_else_true() {
        let mut micro_controller: MicroController = Default::default();
        // CMP 3 5; ITE EQ; MOV 1 1; MOV 1 0;
        micro_controller
            .bus_matrix
            .set16(RAM0_BEGIN, 0b0100001010_101_011);
        micro_controller
            .bus_matrix
            .set16(RAM0_BEGIN + 2, 0b10111111_0000_1100);
        micro_controller
            .bus_matrix
            .set16(RAM0_BEGIN + 4, 0b00100_001_00000001);
        micro_controller
            .bus_matrix
            .set16(RAM0_BEGIN + 6, 0b00100_001_00000000);
        micro_controller.registers[3] = 4;
        micro_controller.registers[5] = 4;
        micro_controller.program_counter = RAM0_BEGIN;

        micro_controller.emulate_one_insn();
        micro_controller.emulate_one_insn();
        micro_controller.emulate_one_insn();
        micro_controller.emulate_one_insn();

        assert_eq!(micro_controller.registers[1], 1);
        assert_eq!(micro_controller.program_counter, RAM0_BEGIN + 8);
    }

    #[test]
    fn if_eq_then_else_false() {
        let mut micro_controller: MicroController = Default::default();
        // CMP 3 5; ITE EQ; MOV 1 1; MOV 1 0;
        micro_controller
            .bus_matrix
            .set16(RAM0_BEGIN, 0b0100001010_101_011);
        micro_controller
            .bus_matrix
            .set16(RAM0_BEGIN + 2, 0b10111111_0000_1100);
        micro_controller
            .bus_matrix
            .set16(RAM0_BEGIN + 4, 0b00100_001_00000001);
        micro_controller
            .bus_matrix
            .set16(RAM0_BEGIN + 6, 0b00100_001_00000000);
        micro_controller.registers[3] = 4;
        micro_controller.registers[5] = 5;
        micro_controller.program_counter = RAM0_BEGIN;

        micro_controller.emulate_one_insn();
        micro_controller.emulate_one_insn();
        micro_controller.emulate_one_insn();
        micro_controller.emulate_one_insn();

        assert_eq!(micro_controller.registers[1], 0);
        assert_eq!(micro_controller.program_counter, RAM0_BEGIN + 8);
    }

    #[test]
    fn if_ne_then_else_true() {
        let mut micro_controller: MicroController = Default::default();
        // CMP 3 5; ITE NE; MOV 1 1; MOV 1 0;
        micro_controller
            .bus_matrix
            .set16(RAM0_BEGIN, 0b0100001010_101_011);
        micro_controller
            .bus_matrix
            .set16(RAM0_BEGIN + 2, 0b10111111_0001_0100);
        micro_controller
            .bus_matrix
            .set16(RAM0_BEGIN + 4, 0b00100_001_00000001);
        micro_controller
            .bus_matrix
            .set16(RAM0_BEGIN + 6, 0b00100_001_00000000);
        micro_controller.registers[3] = 4;
        micro_controller.registers[5] = 5;
        micro_controller.program_counter = RAM0_BEGIN;

        micro_controller.emulate_one_insn();
        micro_controller.emulate_one_insn();
        micro_controller.emulate_one_insn();
        micro_controller.emulate_one_insn();

        assert_eq!(micro_controller.registers[1], 1);
        assert_eq!(micro_controller.program_counter, RAM0_BEGIN + 8);
    }

    #[test]
    fn if_ne_then_else_false() {
        let mut micro_controller: MicroController = Default::default();
        // CMP 3 5; ITE NE; MOV 1 1; MOV 1 0;
        micro_controller
            .bus_matrix
            .set16(RAM0_BEGIN, 0b0100001010_101_011);
        micro_controller
            .bus_matrix
            .set16(RAM0_BEGIN + 2, 0b10111111_0001_0100);
        micro_controller
            .bus_matrix
            .set16(RAM0_BEGIN + 4, 0b00100_001_00000001);
        micro_controller
            .bus_matrix
            .set16(RAM0_BEGIN + 6, 0b00100_001_00000000);
        micro_controller.registers[3] = 4;
        micro_controller.registers[5] = 4;
        micro_controller.program_counter = RAM0_BEGIN;

        micro_controller.emulate_one_insn();
        micro_controller.emulate_one_insn();
        micro_controller.emulate_one_insn();
        micro_controller.emulate_one_insn();

        assert_eq!(micro_controller.registers[1], 0);
        assert_eq!(micro_controller.program_counter, RAM0_BEGIN + 8);
    }

    #[test]
    fn if_ge_then_else_true() {
        let mut micro_controller: MicroController = Default::default();
        // CMP 3 5; ITE GE; MOV 1 1; MOV 1 0;
        micro_controller
            .bus_matrix
            .set16(RAM0_BEGIN, 0b0100001010_101_011);
        micro_controller
            .bus_matrix
            .set16(RAM0_BEGIN + 2, 0b10111111_0010_1100);
        micro_controller
            .bus_matrix
            .set16(RAM0_BEGIN + 4, 0b00100_001_00000001);
        micro_controller
            .bus_matrix
            .set16(RAM0_BEGIN + 6, 0b00100_001_00000000);
        micro_controller.registers[3] = 5;
        micro_controller.registers[5] = 5;
        micro_controller.program_counter = RAM0_BEGIN;

        micro_controller.emulate_one_insn();
        micro_controller.emulate_one_insn();
        micro_controller.emulate_one_insn();
        micro_controller.emulate_one_insn();

        assert_eq!(micro_controller.registers[1], 1);
        assert_eq!(micro_controller.program_counter, RAM0_BEGIN + 8);
    }

    #[test]
    fn if_ge_then_else_false() {
        let mut micro_controller: MicroController = Default::default();
        // CMP 3 5; ITE GE; MOV 1 1; MOV 1 0;
        micro_controller
            .bus_matrix
            .set16(RAM0_BEGIN, 0b0100001010_101_011);
        micro_controller
            .bus_matrix
            .set16(RAM0_BEGIN + 2, 0b10111111_0010_1100);
        micro_controller
            .bus_matrix
            .set16(RAM0_BEGIN + 4, 0b00100_001_00000001);
        micro_controller
            .bus_matrix
            .set16(RAM0_BEGIN + 6, 0b00100_001_00000000);
        micro_controller.registers[3] = 4;
        micro_controller.registers[5] = 5;
        micro_controller.program_counter = RAM0_BEGIN;

        micro_controller.emulate_one_insn();
        micro_controller.emulate_one_insn();
        micro_controller.emulate_one_insn();
        micro_controller.emulate_one_insn();

        assert_eq!(micro_controller.registers[1], 0);
        assert_eq!(micro_controller.program_counter, RAM0_BEGIN + 8);
    }

    #[test]
    fn if_lt_then_else_true() {
        let mut micro_controller: MicroController = Default::default();
        // CMP 3 5; ITE LT; MOV 1 1; MOV 1 0;
        micro_controller
            .bus_matrix
            .set16(RAM0_BEGIN, 0b0100001010_101_011);
        micro_controller
            .bus_matrix
            .set16(RAM0_BEGIN + 2, 0b10111111_0011_0100);
        micro_controller
            .bus_matrix
            .set16(RAM0_BEGIN + 4, 0b00100_001_00000001);
        micro_controller
            .bus_matrix
            .set16(RAM0_BEGIN + 6, 0b00100_001_00000000);
        micro_controller.registers[3] = 4;
        micro_controller.registers[5] = 5;
        micro_controller.program_counter = RAM0_BEGIN;

        micro_controller.emulate_one_insn();
        micro_controller.emulate_one_insn();
        micro_controller.emulate_one_insn();
        micro_controller.emulate_one_insn();

        assert_eq!(micro_controller.registers[1], 1);
        assert_eq!(micro_controller.program_counter, RAM0_BEGIN + 8);
    }

    #[test]
    fn if_lt_then_else_false() {
        let mut micro_controller: MicroController = Default::default();
        // CMP 3 5; ITE LT; MOV 1 1; MOV 1 0;
        micro_controller
            .bus_matrix
            .set16(RAM0_BEGIN, 0b0100001010_101_011);
        micro_controller
            .bus_matrix
            .set16(RAM0_BEGIN + 2, 0b10111111_0011_0100);
        micro_controller
            .bus_matrix
            .set16(RAM0_BEGIN + 4, 0b00100_001_00000001);
        micro_controller
            .bus_matrix
            .set16(RAM0_BEGIN + 6, 0b00100_001_00000000);
        micro_controller.registers[3] = 4;
        micro_controller.registers[5] = 4;
        micro_controller.program_counter = RAM0_BEGIN;

        micro_controller.emulate_one_insn();
        micro_controller.emulate_one_insn();
        micro_controller.emulate_one_insn();
        micro_controller.emulate_one_insn();

        assert_eq!(micro_controller.registers[1], 0);
        assert_eq!(micro_controller.program_counter, RAM0_BEGIN + 8);
    }

    #[test]
    fn if_gt_then_else_true() {
        let mut micro_controller: MicroController = Default::default();
        // CMP 3 5; ITE GT; MOV 1 1; MOV 1 0;
        micro_controller
            .bus_matrix
            .set16(RAM0_BEGIN, 0b0100001010_101_011);
        micro_controller
            .bus_matrix
            .set16(RAM0_BEGIN + 2, 0b10111111_1000_1100);
        micro_controller
            .bus_matrix
            .set16(RAM0_BEGIN + 4, 0b00100_001_00000001);
        micro_controller
            .bus_matrix
            .set16(RAM0_BEGIN + 6, 0b00100_001_00000000);
        micro_controller.registers[3] = 5;
        micro_controller.registers[5] = 4;
        micro_controller.program_counter = RAM0_BEGIN;

        micro_controller.emulate_one_insn();
        micro_controller.emulate_one_insn();
        micro_controller.emulate_one_insn();
        micro_controller.emulate_one_insn();

        assert_eq!(micro_controller.registers[1], 1);
        assert_eq!(micro_controller.program_counter, RAM0_BEGIN + 8);
    }

    #[test]
    fn if_gt_then_else_false() {
        let mut micro_controller: MicroController = Default::default();
        // CMP 3 5; ITE GT; MOV 1 1; MOV 1 0;
        micro_controller
            .bus_matrix
            .set16(RAM0_BEGIN, 0b0100001010_101_011);
        micro_controller
            .bus_matrix
            .set16(RAM0_BEGIN + 2, 0b10111111_1000_1100);
        micro_controller
            .bus_matrix
            .set16(RAM0_BEGIN + 4, 0b00100_001_00000001);
        micro_controller
            .bus_matrix
            .set16(RAM0_BEGIN + 6, 0b00100_001_00000000);
        micro_controller.registers[3] = 4;
        micro_controller.registers[5] = 4;
        micro_controller.program_counter = RAM0_BEGIN;

        micro_controller.emulate_one_insn();
        micro_controller.emulate_one_insn();
        micro_controller.emulate_one_insn();
        micro_controller.emulate_one_insn();

        assert_eq!(micro_controller.registers[1], 0);
        assert_eq!(micro_controller.program_counter, RAM0_BEGIN + 8);
    }

    #[test]
    fn if_le_then_else_true() {
        let mut micro_controller: MicroController = Default::default();
        // CMP 3 5; ITE LE; MOVW 1 1; MOV 1 0;
        micro_controller
            .bus_matrix
            .set16(RAM0_BEGIN, 0b0100001010_101_011);
        micro_controller
            .bus_matrix
            .set16(RAM0_BEGIN + 2, 0b10111111_1001_0100);
        micro_controller
            .bus_matrix
            .set32(RAM0_BEGIN + 4, 0b0_000_0001_00000001_11110_0_100100_0000);
        micro_controller
            .bus_matrix
            .set16(RAM0_BEGIN + 8, 0b00100_001_00000000);
        micro_controller.registers[3] = 4;
        micro_controller.registers[5] = 4;
        micro_controller.program_counter = RAM0_BEGIN;

        micro_controller.emulate_one_insn();
        micro_controller.emulate_one_insn();
        micro_controller.emulate_one_insn();
        micro_controller.emulate_one_insn();

        assert_eq!(micro_controller.registers[1], 1);
        assert_eq!(micro_controller.program_counter, RAM0_BEGIN + 10);
    }

    #[test]
    fn if_le_then_else_false() {
        let mut micro_controller: MicroController = Default::default();
        // CMP 3 5; ITE LE; MOVW 1 1; MOV 1 0;
        micro_controller
            .bus_matrix
            .set16(RAM0_BEGIN, 0b0100001010_101_011);
        micro_controller
            .bus_matrix
            .set16(RAM0_BEGIN + 2, 0b10111111_1001_0100);
        micro_controller
            .bus_matrix
            .set32(RAM0_BEGIN + 4, 0b0_000_0001_00000001_11110_0_100100_0000);
        micro_controller
            .bus_matrix
            .set16(RAM0_BEGIN + 8, 0b00100_001_00000000);
        micro_controller.registers[3] = 5;
        micro_controller.registers[5] = 4;
        micro_controller.program_counter = RAM0_BEGIN;

        micro_controller.emulate_one_insn();
        micro_controller.emulate_one_insn();
        micro_controller.emulate_one_insn();
        micro_controller.emulate_one_insn();

        assert_eq!(micro_controller.registers[1], 0);
        assert_eq!(micro_controller.program_counter, RAM0_BEGIN + 10);
    }

    #[test]
    fn if_eq_then_then_else_true() {
        let mut micro_controller: MicroController = Default::default();
        // CMP 3 5; ITTE EQ; MOV 1 1; ADD 1 #1 ; MOV 1 0;
        micro_controller
            .bus_matrix
            .set16(RAM0_BEGIN, 0b0100001010_101_011);
        micro_controller
            .bus_matrix
            .set16(RAM0_BEGIN + 2, 0b10111111_0000_0110);
        micro_controller
            .bus_matrix
            .set16(RAM0_BEGIN + 4, 0b00100_001_00000001);
        micro_controller
            .bus_matrix
            .set16(RAM0_BEGIN + 6, 0b00110_001_00000001);
        micro_controller
            .bus_matrix
            .set16(RAM0_BEGIN + 8, 0b00100_001_00000000);
        micro_controller.registers[3] = 4;
        micro_controller.registers[5] = 4;
        micro_controller.program_counter = RAM0_BEGIN;

        micro_controller.emulate_one_insn();
        micro_controller.emulate_one_insn();
        micro_controller.emulate_one_insn();
        micro_controller.emulate_one_insn();
        micro_controller.emulate_one_insn();

        assert_eq!(micro_controller.registers[1], 2);
        assert_eq!(micro_controller.program_counter, RAM0_BEGIN + 10);
    }

    #[test]
    fn if_eq_then_then_else_false() {
        let mut micro_controller: MicroController = Default::default();
        // CMP 3 5; ITTE EQ; MOV 1 1; ADD 1 #1 ; MOV 1 0;
        micro_controller
            .bus_matrix
            .set16(RAM0_BEGIN, 0b0100001010_101_011);
        micro_controller
            .bus_matrix
            .set16(RAM0_BEGIN + 2, 0b10111111_0000_0110);
        micro_controller
            .bus_matrix
            .set16(RAM0_BEGIN + 4, 0b00100_001_00000001);
        micro_controller
            .bus_matrix
            .set16(RAM0_BEGIN + 6, 0b00110_001_00000001);
        micro_controller
            .bus_matrix
            .set16(RAM0_BEGIN + 8, 0b00100_001_00000000);
        micro_controller.registers[3] = 4;
        micro_controller.registers[5] = 5;
        micro_controller.program_counter = RAM0_BEGIN;

        micro_controller.emulate_one_insn();
        micro_controller.emulate_one_insn();
        micro_controller.emulate_one_insn();
        micro_controller.emulate_one_insn();
        micro_controller.emulate_one_insn();

        assert_eq!(micro_controller.registers[1], 0);
        assert_eq!(micro_controller.program_counter, RAM0_BEGIN + 10);
    }

    #[test]
    #[should_panic(expected = "Unsupported IT condition 0b100")]
    fn if_unsupported_condition_then() {
        let mut micro_controller: MicroController = Default::default();
        // CMP 3 5; ITE CS; MOV 1 1
        micro_controller
            .bus_matrix
            .set16(RAM0_BEGIN, 0b0100001010_101_011);
        micro_controller
            .bus_matrix
            .set16(RAM0_BEGIN + 2, 0b10111111_0100_1000);
        micro_controller
            .bus_matrix
            .set16(RAM0_BEGIN + 4, 0b00100_001_00000001);
        micro_controller.program_counter = RAM0_BEGIN;

        micro_controller.emulate_one_insn();
        micro_controller.emulate_one_insn();
        micro_controller.emulate_one_insn();
    }

    #[test]
    #[should_panic(expected = "Unsupported or invalid instruction 0b1110000000000 at 0x20000000")]
    fn unsupported_insn() {
        let mut micro_controller: MicroController = Default::default();
        micro_controller
            .bus_matrix
            .set16(RAM0_BEGIN, 0b00011_10_000000000);
        micro_controller.program_counter = RAM0_BEGIN;

        micro_controller.emulate_one_insn();
    }

    #[test]
    fn set_usart_input() {
        let mut micro_controller: MicroController = Default::default();
        micro_controller.set_serial_input("W400E0E00,FFFFFFFF#"); // PIO Enable Register.
        micro_controller.set_serial_input("W400E0610,00020000#"); // Clock Enable Register.
        micro_controller.set_serial_input("W40098000,00000010#"); // USART Control Register.

        micro_controller.set_usart_input(0xAB, 0, 0);

        assert_eq!(
            micro_controller
                .bus_matrix
                .get32(crate::usart::RECEIVE_HOLDING_REGISTER),
            0xAB
        );
    }

    #[test]
    fn set_usart_input_clock_disabled() {
        let mut micro_controller: MicroController = Default::default();
        micro_controller.set_serial_input("W400E0E00,FFFFFFFF#"); // PIO Enable Register.
        micro_controller.set_serial_input("W40098000,00000010#"); // USART Control Register.

        micro_controller.set_usart_input(0xAB, 0, 0);

        assert_eq!(
            micro_controller
                .bus_matrix
                .get32(crate::usart::RECEIVE_HOLDING_REGISTER),
            0
        );
    }

    #[test]
    fn set_usart_input_usart_input_pins_not_enabled() {
        let mut micro_controller: MicroController = Default::default();
        micro_controller.set_serial_input("W400E0E00,FFFFFFFF#"); // PIO Enable Register.
        micro_controller.set_serial_input("W400E0E10,00020000#"); // Output Enable Register.
        micro_controller.set_serial_input("W400E0610,00020000#"); // Clock Enable Register.
        micro_controller.set_serial_input("W40098000,00000010#"); // USART Control Register.

        micro_controller.set_usart_input(0xAB, 0, 0);

        assert_eq!(
            micro_controller
                .bus_matrix
                .get32(crate::usart::RECEIVE_HOLDING_REGISTER),
            0
        );
    }

    #[test]
    fn get_pin_output() {
        let mut micro_controller: MicroController = Default::default();

        let out0 = micro_controller.get_pin_output(Controller::PB, 3);
        micro_controller.set_serial_input("W400E1010,00000008#"); // Output Enable Register (OER)
        let out1 = micro_controller.get_pin_output(Controller::PB, 3);

        assert!(out0);
        assert!(!out1);
    }

    fn fill_page(micro_controller: &mut MicroController, address: u32) {
        for i in 0..64 {
            micro_controller.set_serial_input(&format!("W{:010X},#", address + 4 * i));
        }
    }

    #[test]
    fn run() {
        let mut micro_controller: MicroController = Default::default();
        fill_page(&mut micro_controller, 0x80000);
        micro_controller.set_serial_input("W00080000,20088000#"); // Initial stack.
        micro_controller.set_serial_input("W00080004,00080009#"); // Reset handler.
        micro_controller.set_serial_input("W00080008,E7FE317B#"); // ADD Rdn=1 123, B self.
        micro_controller.set_serial_input("W400E0A04,5A000003#"); // Flash page 0.
        micro_controller.set_serial_input("W400E0A04,5A00010B#"); // Set boot from flash.
        micro_controller.set_serial_input("W400E1A00,A500000D#"); // Reset.

        micro_controller.run(10);

        assert_eq!(micro_controller.registers[1], 123);
    }

    #[test]
    fn run_software_reset() {
        let mut micro_controller: MicroController = Default::default();
        fill_page(&mut micro_controller, 0x80000);
        micro_controller.set_serial_input("W00080000,20088000#"); // Initial stack.
        micro_controller.set_serial_input("W00080004,00080009#"); // Reset handler.
        micro_controller.set_serial_input("W00080008,4B024A01#"); // LDR Rd=2 addr, LDR Rd=3 value.
        micro_controller.set_serial_input("W0008000C,E7FE6013#"); // STR value in addr, B forever.
        micro_controller.set_serial_input("W00080010,400E1A00#"); // addr.
        micro_controller.set_serial_input("W00080014,A500000D#"); // value.
        micro_controller.set_serial_input("W400E0A04,5A000003#"); // Flash page 0.
        micro_controller.set_serial_input("W400E0A04,5A00010B#"); // Set boot from flash.
        micro_controller.set_serial_input("W400E1A00,A500000D#"); // Reset.
        micro_controller.bus_matrix.set32(0x20080000, 123);

        micro_controller.run(5);

        assert_ne!(micro_controller.bus_matrix.get32(0x20080000), 123);
    }

    #[test]
    fn run_interrupt_return_with_pop() {
        let mut micro_controller: MicroController = Default::default();
        fill_page(&mut micro_controller, 0x80000);
        micro_controller.set_serial_input("W00080000,20088000#"); // Initial stack.
        micro_controller.set_serial_input("W00080004,00080009#"); // Reset handler.
        micro_controller.set_serial_input("W00080008,E7FE317B#"); // ADD Rdn=1 123, B self.
        micro_controller.set_serial_input("W00080040,00080045#"); // IRQ0 handler.
        micro_controller.set_serial_input("W00080044,342AB500#"); // PUSH LR, ADD Rdn=4 42.
        micro_controller.set_serial_input("W00080048,0000BD00#"); // POP PC.
        micro_controller.set_serial_input("W400E0A04,5A000003#"); // Flash page 0.
        micro_controller.set_serial_input("W400E0A04,5A00010B#"); // Set boot from flash.
        micro_controller.set_serial_input("W400E1A00,A500000D#"); // Reset.

        micro_controller.run(10);
        micro_controller.bus_matrix.set32(0xE000E100, 1); // Enable interrupt 0.
        micro_controller.bus_matrix.set32(0xE000E200, 1); // Set interrupt 0 pending.
        micro_controller.run(10);

        assert_eq!(micro_controller.registers[1], 123);
        assert_eq!(micro_controller.registers[4], 42);
    }

    #[test]
    fn run_interrupt_return_with_bx() {
        let mut micro_controller: MicroController = Default::default();
        fill_page(&mut micro_controller, 0x80000);
        micro_controller.set_serial_input("W00080000,20088000#"); // Initial stack.
        micro_controller.set_serial_input("W00080004,00080009#"); // Reset handler.
        micro_controller.set_serial_input("W00080008,E7FE317B#"); // ADD Rdn=1 123, B self.
        micro_controller.set_serial_input("W00080040,00080045#"); // IRQ0 handler.
        micro_controller.set_serial_input("W00080044,4770342A#"); // BX LR, ADD Rdn=4 42.
        micro_controller.set_serial_input("W400E0A04,5A000003#"); // Flash page 0.
        micro_controller.set_serial_input("W400E0A04,5A00010B#"); // Set boot from flash.
        micro_controller.set_serial_input("W400E1A00,A500000D#"); // Reset.

        micro_controller.run(10);
        micro_controller.bus_matrix.set32(0xE000E100, 1); // Enable interrupt 0.
        micro_controller.bus_matrix.set32(0xE000E200, 1); // Set interrupt 0 pending.
        micro_controller.run(10);

        assert_eq!(micro_controller.registers[1], 123);
        assert_eq!(micro_controller.registers[4], 42);
    }

    #[test]
    #[should_panic(expected = "Bad handler interworking address at 0x00000040")]
    fn run_interrupt_bad_interworking_address() {
        let mut micro_controller: MicroController = Default::default();
        fill_page(&mut micro_controller, 0x80000);
        micro_controller.set_serial_input("W00080000,20088000#"); // Initial stack.
        micro_controller.set_serial_input("W00080004,00080009#"); // Reset handler.
        micro_controller.set_serial_input("W00080008,E7FE317B#"); // ADD Rdn=1 123, B self.
        micro_controller.set_serial_input("W00080040,00080044#"); // IRQ0 handler (BAD).
        micro_controller.set_serial_input("W00080044,342AB500#"); // PUSH LR, ADD Rdn=4 42.
        micro_controller.set_serial_input("W00080048,0000BD00#"); // POP PC.
        micro_controller.set_serial_input("W400E0A04,5A000003#"); // Flash page 0.
        micro_controller.set_serial_input("W400E0A04,5A00010B#"); // Set boot from flash.
        micro_controller.set_serial_input("W400E1A00,A500000D#"); // Reset.

        micro_controller.run(10);
        micro_controller.bus_matrix.set32(0xE000E100, 1); // Enable interrupt 0.
        micro_controller.bus_matrix.set32(0xE000E200, 1); // Set interrupt 0 pending.
        micro_controller.run(10);
    }

    #[test]
    fn get_set_pseudo_program_status() {
        use std::cmp::Ordering::*;
        let mut micro_controller = MicroController::default();
        for padding in [false, true] {
            for order in [Less, Equal, Greater] {
                micro_controller.if_then_state = 0xAB;
                micro_controller.last_comparison = order;

                let p = micro_controller.set_pseudo_program_status_register(
                    micro_controller.get_pseudo_program_status_register(padding),
                );

                assert_eq!(micro_controller.last_comparison, order);
                assert_eq!(micro_controller.if_then_state, 0xAB);
                assert_eq!(p, padding);
            }
        }
    }
}
