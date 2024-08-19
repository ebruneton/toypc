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

use crate::{
    arm::Instruction,
    backup::{BackupRegistersController, BACKUP_REGISTERS_BEGIN, BACKUP_REGISTERS_LAST},
    flash::{FlashController, FLASH_CONTROLLER_BEGIN, FLASH_CONTROLLER_LAST},
    interrupt::{
        NestedVectorInterruptController, NESTED_VECTOR_INTERRUPT_CONTROLLER_BEGIN,
        NESTED_VECTOR_INTERRUPT_CONTROLLER_LAST,
    },
    memory::MemoryBank,
    mpu::{MemoryProtectionUnit, MEMORY_PROTECTION_UNIT_BEGIN, MEMORY_PROTECTION_UNIT_LAST},
    pio::{ParallelIoController, PIO_CONTROLLER_BEGIN, PIO_CONTROLLER_LAST},
    power::{PowerManagementController, PM_CONTROLLER_BEGIN, PM_CONTROLLER_LAST},
    reset::{ResetController, RESET_CONTROLLER_BEGIN, RESET_CONTROLLER_LAST},
    spi::{SerialPeripheralInterfaceController, SPI_CONTROLLER_BEGIN, SPI_CONTROLLER_LAST},
    system::{SystemControlBlock, SYSTEM_CONTROL_BLOCK_BEGIN, SYSTEM_CONTROL_BLOCK_LAST},
    time::{SystemTimer, WaitFunction, SYSTEM_TIMER_BEGIN, SYSTEM_TIMER_LAST},
    usart::{
        UniversalReceiverTransmitter, UNIVERSAL_RECEIVER_TRANSMITTER_BEGIN,
        UNIVERSAL_RECEIVER_TRANSMITTER_LAST,
    },
    watchdog::{WatchdogTimer, WATCHDOG_TIMER_BEGIN, WATCHDOG_TIMER_LAST},
};

// boot memory (maps flash memory bank0, bank1 or internal ROM):
// - [0, 80000[            : 512K
//
// flash memory:
// - [80000, C0000[        : 256KB, bank0, not repeated
// - [C0000, 100000[       : 256KB, bank1, not repeated
//
// internal ROM:
// - [100000, 200000[      : 1MB,   8KB used, repeated 128 times
//
// RAM:
// - [20000000, 20080000[  : 512KB, SRAM0 of 64KB, repeated 8 times
// - [20080000, 20100000[  : 512KB, SRAM1 of 32KB, repeated 16 times
//
// -> contiguous 96KB SRAM in [20070000, 20088000[

const BOOT_BEGIN: u32 = 0;
const BOOT_END: u32 = 0x80000;
const BOOT_LAST: u32 = BOOT_END - 1;

pub const FLASH_BEGIN: u32 = BOOT_END;
pub const FLASH_END: u32 = 0x100000;
pub const FLASH_LAST: u32 = FLASH_END - 1;
pub const FLASH_BYTES: u32 = FLASH_END - FLASH_BEGIN;

const ROM_BEGIN: u32 = FLASH_END;
const ROM_END: u32 = 0x200000;
const ROM_LAST: u32 = ROM_END - 1;
const ROM_BYTES: u32 = 8 * 1024;

pub const RAM0_BEGIN: u32 = 0x20000000;
pub const RAM0_END: u32 = 0x20080000;
pub const RAM0_LAST: u32 = RAM0_END - 1;
pub const RAM0_BYTES: u32 = 64 * 1024;

pub const RAM1_BEGIN: u32 = 0x20080000;
pub const RAM1_END: u32 = 0x20100000;
pub const RAM1_LAST: u32 = RAM1_END - 1;
pub const RAM1_BYTES: u32 = 32 * 1024;

#[derive(Clone)]
pub struct BusMatrix {
    seed: u32,
    flash: MemoryBank,
    rom: MemoryBank,
    ram0: MemoryBank,
    ram1: MemoryBank,
    spi_controller: SerialPeripheralInterfaceController,
    usart_controller: UniversalReceiverTransmitter,
    power_management_controller: PowerManagementController,
    flash_controller: FlashController,
    pio_controller: ParallelIoController,
    reset_controller: ResetController,
    watchdog_timer: WatchdogTimer,
    backup_registers: BackupRegistersController,
    system_timer: SystemTimer,
    interrupt_controller: NestedVectorInterruptController,
    system_controller: SystemControlBlock,
    memory_protection_unit: MemoryProtectionUnit,
    pub handler_mode: bool,
    pub privilege_mode: bool,
}

macro_rules! static_assert {
    ( $expr:expr ) => {
        const _: () = assert!($expr);
    };
}

macro_rules! static_assert_fast_modulo {
    ( $base:expr, $modulo:expr ) => {
        static_assert!($base % $modulo == 0 && $modulo.count_ones() == 1);
    };
}

impl BusMatrix {
    pub fn uninitialized(seed: u32, wait_function: Option<WaitFunction>) -> Self {
        Self {
            seed,
            flash: MemoryBank::uninitialized(FLASH_BYTES / 4),
            rom: MemoryBank::uninitialized(ROM_BYTES / 4),
            ram0: MemoryBank::uninitialized(RAM0_BYTES / 4),
            ram1: MemoryBank::uninitialized(RAM1_BYTES / 4),
            spi_controller: SerialPeripheralInterfaceController::new(),
            usart_controller: UniversalReceiverTransmitter::new(),
            power_management_controller: PowerManagementController::new(),
            flash_controller: FlashController::uninitialized(),
            pio_controller: ParallelIoController::uninitialized(),
            reset_controller: ResetController::new(),
            watchdog_timer: WatchdogTimer::uninitialized(),
            backup_registers: BackupRegistersController::new(),
            system_timer: SystemTimer::new(wait_function),
            interrupt_controller: NestedVectorInterruptController::new(),
            system_controller: SystemControlBlock::new(),
            memory_protection_unit: MemoryProtectionUnit::uninitialized(),
            handler_mode: false,
            privilege_mode: true,
        }
    }

    #[cfg(test)]
    pub fn default() -> Self {
        Self::new()
    }

    #[cfg(test)]
    pub fn new() -> Self {
        let mut result = Self::uninitialized(1, None);
        result.erase();
        result.reset();
        result
    }

    #[inline]
    pub fn boot_from_flash(&self) -> bool {
        self.flash_controller.boot_from_flash()
    }

    pub fn get_flash_content(&self) -> &Vec<u32> {
        self.flash.get_content()
    }

    #[inline]
    pub fn get_spi_controller(&self) -> &SerialPeripheralInterfaceController {
        &self.spi_controller
    }

    #[inline]
    pub fn get_spi_controller_mut(&mut self) -> &mut SerialPeripheralInterfaceController {
        &mut self.spi_controller
    }

    #[inline]
    pub fn get_usart_controller_mut(&mut self) -> &mut UniversalReceiverTransmitter {
        &mut self.usart_controller
    }

    #[inline]
    pub fn get_power_management_controller(&self) -> &PowerManagementController {
        &self.power_management_controller
    }

    #[inline]
    pub fn get_pio_controller(&self) -> &ParallelIoController {
        &self.pio_controller
    }

    #[inline]
    pub fn get_pio_controller_mut(&mut self) -> &mut ParallelIoController {
        &mut self.pio_controller
    }

    #[inline]
    pub fn get_reset_controler(&self) -> &ResetController {
        &self.reset_controller
    }

    #[inline]
    pub fn get_system_controller(&self) -> &SystemControlBlock {
        &self.system_controller
    }

    #[inline]
    pub fn get_memory_protection_unit(&mut self) -> &mut MemoryProtectionUnit {
        &mut self.memory_protection_unit
    }

    #[inline]
    pub fn is_privileged(&self) -> bool {
        self.privilege_mode | self.handler_mode
    }

    pub fn get8(&mut self, address: u32) -> u32 {
        let aligned_address = address & !3;
        let word = self.get32_aligned(aligned_address);
        let shift = (address & 3) << 3;
        (word >> shift) & 0xFF
    }

    pub fn set8(&mut self, address: u32, value: u8) {
        let aligned_address = address & !3;
        let word = self.get32_aligned(aligned_address);
        let shift = (address & 3) << 3;
        self.set32_aligned(
            aligned_address,
            (word & !(0xFF << shift)) | ((value as u32) << shift),
        );
    }

    pub fn get16(&mut self, address: u32) -> u32 {
        let aligned_address = address & !3;
        let word = self.get32_aligned(aligned_address);
        let shift = (address & 3) << 3;
        if shift != 24 {
            (word >> shift) & 0xFFFF
        } else {
            let next_word = self.get32_aligned(aligned_address + 4);
            ((next_word << 8) | (word >> 24)) & 0xFFFF
        }
    }

    pub fn set16(&mut self, address: u32, value: u16) {
        let aligned_address = address & !3;
        let word = self.get32_aligned(aligned_address);
        let shift = (address & 3) << 3;
        self.set32_aligned(
            aligned_address,
            (word & !(0xFFFF << shift)) | ((value as u32) << shift),
        );
        if shift == 24 {
            let next_word = self.get32_aligned(aligned_address + 4);
            self.set32_aligned(
                aligned_address + 4,
                (next_word & 0xFFFFFF00) | (value >> 8) as u32,
            );
        }
    }

    pub fn get32(&mut self, address: u32) -> u32 {
        let shift = (address & 3) << 3;
        if shift == 0 {
            self.get32_aligned(address)
        } else {
            let aligned_address = address & !3;
            let word = self.get32_aligned(aligned_address);
            let next_word = self.get32_aligned(aligned_address + 4);
            (next_word << (32 - shift)) | (word >> shift)
        }
    }

    pub fn set32(&mut self, address: u32, value: u32) {
        let shift = (address & 3) << 3;
        if shift == 0 {
            self.set32_aligned(address, value);
        } else {
            let aligned_address = address & !3;
            let word = self.get32_aligned(aligned_address);
            self.set32_aligned(
                aligned_address,
                (word & !(0xFFFFFFFF << shift)) | (value << shift),
            );
            let next_word = self.get32_aligned(aligned_address + 4);
            self.set32_aligned(
                aligned_address + 4,
                (next_word & (0xFFFFFFFF << shift)) | (value >> (32 - shift)),
            );
        }
    }

    fn get32_aligned(&mut self, address: u32) -> u32 {
        debug_assert!(address % 4 == 0);
        if !self
            .memory_protection_unit
            .validate_address(address, self.is_privileged())
        {
            panic!("Unauthorized memory access at {address:#010X}");
        }
        match address {
            BOOT_BEGIN..=BOOT_LAST => {
                if self.boot_from_flash() {
                    static_assert!(FLASH_BYTES == BOOT_END);
                    self.flash.get32_aligned(address)
                } else {
                    static_assert_fast_modulo!(ROM_BEGIN, ROM_BYTES);
                    self.rom.get32_aligned(address & (ROM_BYTES - 1))
                }
            }
            FLASH_BEGIN..=FLASH_LAST => self.flash.get32_aligned(address - FLASH_BEGIN),
            ROM_BEGIN..=ROM_LAST => {
                static_assert_fast_modulo!(ROM_BEGIN, ROM_BYTES);
                self.rom.get32_aligned(address & (ROM_BYTES - 1))
            }
            RAM0_BEGIN..=RAM0_LAST => {
                static_assert_fast_modulo!(RAM0_BEGIN, RAM0_BYTES);
                self.ram0.get32_aligned(address & (RAM0_BYTES - 1))
            }
            RAM1_BEGIN..=RAM1_LAST => {
                static_assert_fast_modulo!(RAM1_BEGIN, RAM1_BYTES);
                self.ram1.get32_aligned(address & (RAM1_BYTES - 1))
            }
            SPI_CONTROLLER_BEGIN..=SPI_CONTROLLER_LAST => {
                self.spi_controller.get32_aligned(address)
            }
            UNIVERSAL_RECEIVER_TRANSMITTER_BEGIN..=UNIVERSAL_RECEIVER_TRANSMITTER_LAST => {
                self.usart_controller.get32_aligned(address)
            }
            PM_CONTROLLER_BEGIN..=PM_CONTROLLER_LAST => {
                self.power_management_controller.get32_aligned(address)
            }
            FLASH_CONTROLLER_BEGIN..=FLASH_CONTROLLER_LAST => {
                self.flash_controller.get32_aligned(address)
            }
            PIO_CONTROLLER_BEGIN..=PIO_CONTROLLER_LAST => {
                self.pio_controller.get32_aligned(address)
            }
            RESET_CONTROLLER_BEGIN..=RESET_CONTROLLER_LAST => {
                self.reset_controller.get32_aligned(address)
            }
            WATCHDOG_TIMER_BEGIN..=WATCHDOG_TIMER_LAST => {
                self.watchdog_timer.get32_aligned(address)
            }
            BACKUP_REGISTERS_BEGIN..=BACKUP_REGISTERS_LAST => {
                self.backup_registers.get32_aligned(address)
            }
            SYSTEM_TIMER_BEGIN..=SYSTEM_TIMER_LAST => {
                if !self.is_privileged() {
                    panic!("Unauthorized access to {address:#010X}");
                }
                self.system_timer.get32_aligned(address)
            }
            NESTED_VECTOR_INTERRUPT_CONTROLLER_BEGIN..=NESTED_VECTOR_INTERRUPT_CONTROLLER_LAST => {
                if !self.is_privileged() {
                    panic!("Unauthorized access to {address:#010X}");
                }
                self.interrupt_controller.get32_aligned(address)
            }
            SYSTEM_CONTROL_BLOCK_BEGIN..=SYSTEM_CONTROL_BLOCK_LAST => {
                if !self.is_privileged() {
                    panic!("Unauthorized access to {address:#010X}");
                }
                self.system_controller.get32_aligned(address)
            }
            MEMORY_PROTECTION_UNIT_BEGIN..=MEMORY_PROTECTION_UNIT_LAST => {
                if !self.is_privileged() {
                    panic!("Unauthorized access to {address:#010X}");
                }
                self.memory_protection_unit.get32_aligned(address)
            }
            _ => panic!("Unsupported address {address:#010X}"),
        }
    }

    fn set32_aligned(&mut self, address: u32, value: u32) {
        debug_assert!(address % 4 == 0);
        if !self
            .memory_protection_unit
            .validate_address(address, self.is_privileged())
        {
            panic!("Unauthorized memory access at {address:#010X}");
        }
        match address {
            BOOT_BEGIN..=BOOT_LAST => (),
            FLASH_BEGIN..=FLASH_LAST => {
                self.flash_controller.set_latch_buffer(address, value);
            }
            RAM0_BEGIN..=RAM0_LAST => {
                static_assert_fast_modulo!(RAM0_BEGIN, RAM0_BYTES);
                self.ram0.set32_aligned(address & (RAM0_BYTES - 1), value);
            }
            RAM1_BEGIN..=RAM1_LAST => {
                static_assert_fast_modulo!(RAM1_BEGIN, RAM1_BYTES);
                self.ram1.set32_aligned(address & (RAM1_BYTES - 1), value);
            }
            SPI_CONTROLLER_BEGIN..=SPI_CONTROLLER_LAST => {
                let clock_enabled = self.power_management_controller.spi_clock_enabled();
                let output_enabled = self.pio_controller.spi_output_pins_enabled();
                self.spi_controller
                    .set32_aligned(address, value, clock_enabled, output_enabled);
            }
            UNIVERSAL_RECEIVER_TRANSMITTER_BEGIN..=UNIVERSAL_RECEIVER_TRANSMITTER_LAST => {
                self.usart_controller.set32_aligned(address, value);
            }
            PM_CONTROLLER_BEGIN..=PM_CONTROLLER_LAST => {
                self.power_management_controller
                    .set32_aligned(address, value);
            }
            FLASH_CONTROLLER_BEGIN..=FLASH_CONTROLLER_LAST => {
                self.flash_controller
                    .set32_aligned(address, value, &mut self.flash);
            }
            PIO_CONTROLLER_BEGIN..=PIO_CONTROLLER_LAST => {
                self.pio_controller.set32_aligned(address, value);
            }
            RESET_CONTROLLER_BEGIN..=RESET_CONTROLLER_LAST => {
                self.reset_controller.set32_aligned(address, value);
            }
            WATCHDOG_TIMER_BEGIN..=WATCHDOG_TIMER_LAST => {
                self.watchdog_timer.set32_aligned(address, value);
            }
            BACKUP_REGISTERS_BEGIN..=BACKUP_REGISTERS_LAST => {
                self.backup_registers.set32_aligned(address, value);
            }
            SYSTEM_TIMER_BEGIN..=SYSTEM_TIMER_LAST => {
                if !self.is_privileged() {
                    panic!("Unauthorized access to {address:#010X}");
                }
                self.system_timer.set32_aligned(address, value);
            }
            NESTED_VECTOR_INTERRUPT_CONTROLLER_BEGIN..=NESTED_VECTOR_INTERRUPT_CONTROLLER_LAST => {
                if !self.is_privileged() {
                    panic!("Unauthorized access to {address:#010X}");
                }
                self.interrupt_controller.set32_aligned(address, value);
            }
            SYSTEM_CONTROL_BLOCK_BEGIN..=SYSTEM_CONTROL_BLOCK_LAST => {
                if !self.is_privileged() {
                    panic!("Unauthorized access to {address:#010X}");
                }
                self.system_controller.set32_aligned(address, value);
            }
            MEMORY_PROTECTION_UNIT_BEGIN..=MEMORY_PROTECTION_UNIT_LAST => {
                if !self.is_privileged() {
                    panic!("Unauthorized access to {address:#010X}");
                }
                self.memory_protection_unit.set32_aligned(address, value);
            }
            _ => panic!("Unsupported address {address:#X}"),
        }
    }

    #[inline]
    pub fn update(&mut self) -> Option<u8> {
        self.system_timer.update();
        let level_interrupts = self.usart_controller.level_interrupts();
        self.interrupt_controller
            .maybe_activate_interrupt(level_interrupts)
    }

    #[inline]
    pub fn deactivate_interrupt(&mut self) {
        let level_interrupts = self.usart_controller.level_interrupts();
        self.interrupt_controller
            .deactivate_interrupt(level_interrupts)
    }

    pub fn get_insn(&mut self, address: u32) -> Instruction {
        debug_assert!(address % 2 == 0);
        if !self
            .memory_protection_unit
            .validate_address(address & !3, self.is_privileged())
        {
            panic!("Unauthorized memory access at {address:#010X}");
        }
        match address {
            BOOT_BEGIN..=BOOT_LAST => {
                if self.boot_from_flash() {
                    static_assert!(FLASH_BYTES == BOOT_END);
                    self.flash.get_insn(address)
                } else {
                    Instruction::Unsupported
                }
            }
            FLASH_BEGIN..=FLASH_LAST => self.flash.get_insn(address - FLASH_BEGIN),
            RAM0_BEGIN..=RAM0_LAST => {
                static_assert_fast_modulo!(RAM0_BEGIN, RAM0_BYTES);
                self.ram0.get_insn(address & (RAM0_BYTES - 1))
            }
            RAM1_BEGIN..=RAM1_LAST => {
                static_assert_fast_modulo!(RAM1_BEGIN, RAM1_BYTES);
                self.ram1.get_insn(address & (RAM1_BYTES - 1))
            }
            _ => Instruction::Unsupported,
        }
    }

    pub fn decode_insns(&mut self, address: u32, max_count: usize) {
        debug_assert!(address % 2 == 0);
        match address {
            BOOT_BEGIN..=BOOT_LAST => {
                if self.boot_from_flash() {
                    static_assert!(FLASH_BYTES == BOOT_END);
                    self.flash.decode_insns(address, max_count);
                }
            }
            FLASH_BEGIN..=FLASH_LAST => self.flash.decode_insns(address - FLASH_BEGIN, max_count),
            RAM0_BEGIN..=RAM0_LAST => {
                static_assert_fast_modulo!(RAM0_BEGIN, RAM0_BYTES);
                self.ram0
                    .decode_insns(address & (RAM0_BYTES - 1), max_count);
            }
            RAM1_BEGIN..=RAM1_LAST => {
                static_assert_fast_modulo!(RAM1_BEGIN, RAM1_BYTES);
                self.ram1
                    .decode_insns(address & (RAM1_BYTES - 1), max_count);
            }
            _ => (),
        }
    }

    pub fn erase(&mut self) {
        self.flash.reset(0xFFFFFFFF);
        self.flash_controller.erase();
    }

    pub fn reset(&mut self) {
        // See https://en.wikipedia.org/wiki/Linear_congruential_generator.
        const A: u32 = 1664525;
        const B: u32 = 1013904223;
        for i in 0..RAM0_BYTES / 4 {
            self.seed = self.seed.wrapping_mul(A).wrapping_add(B);
            self.ram0.set32_aligned(4 * i, self.seed);
        }
        for i in 0..RAM1_BYTES / 4 {
            self.seed = self.seed.wrapping_mul(A).wrapping_add(B);
            self.ram1.set32_aligned(4 * i, self.seed);
        }
        let mut seed = 1u32;
        for i in 0..ROM_BYTES / 4 {
            seed = seed.wrapping_mul(A).wrapping_add(B);
            self.rom.set32_aligned(4 * i, seed);
        }
        self.rom.set32_aligned(16, 0x001000C7); /* Real content of ROM at this addrees. */
        self.rom.set32_aligned(1032, 0xB004F8CA); /* Real content of ROM at this addrees. */
        self.spi_controller.reset();
        self.usart_controller.reset();
        self.power_management_controller.reset();
        self.flash_controller.reset();
        self.pio_controller.reset();
        self.reset_controller.reset();
        self.watchdog_timer.reset();
        self.backup_registers.reset();
        self.system_timer.reset();
        self.interrupt_controller.reset();
        self.system_controller.reset();
        self.memory_protection_unit.reset();
        self.handler_mode = false;
        self.privilege_mode = true;
    }

    pub fn serialize(&self, output: &mut Vec<u32>) {
        self.flash.serialize(output);
        self.flash_controller.serialize(output);
    }

    pub fn deserialize(&mut self, input: &mut Vec<u32>) {
        self.flash_controller.deserialize(input);
        self.flash.deserialize(input);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn set_boot_from_flash(bus_matrix: &mut BusMatrix) {
        bus_matrix.set32_aligned(0x400E0A04, 0x5A00010B);
    }

    #[test]
    fn get8() {
        let mut bus_matrix = BusMatrix::default();
        bus_matrix.set32_aligned(RAM0_END - 4, 0x44332211);
        bus_matrix.set32_aligned(RAM0_END, 0x88776655);

        assert_eq!(bus_matrix.get8(RAM0_END - 4), 0x11);
        assert_eq!(bus_matrix.get8(RAM0_END - 3), 0x22);
        assert_eq!(bus_matrix.get8(RAM0_END - 2), 0x33);
        assert_eq!(bus_matrix.get8(RAM0_END - 1), 0x44);
        assert_eq!(bus_matrix.get8(RAM0_END), 0x55);
        assert_eq!(bus_matrix.get8(RAM0_END + 1), 0x66);
        assert_eq!(bus_matrix.get8(RAM0_END + 2), 0x77);
        assert_eq!(bus_matrix.get8(RAM0_END + 3), 0x88);
    }

    #[test]
    fn set8() {
        let mut bus_matrix = BusMatrix::default();
        bus_matrix.set32_aligned(RAM0_END - 8, 0x33221100);
        bus_matrix.set32_aligned(RAM0_END - 4, 0x77665544);
        bus_matrix.set32_aligned(RAM0_END, 0xBBAA9988);
        bus_matrix.set32_aligned(RAM0_END + 4, 0xFFEEDDCC);

        bus_matrix.set8(RAM0_END - 5, 0x19);
        bus_matrix.set8(RAM0_END - 2, 0x17);
        bus_matrix.set8(RAM0_END + 1, 0x15);
        bus_matrix.set8(RAM0_END + 4, 0x13);

        assert_eq!(bus_matrix.get32_aligned(RAM0_END - 8), 0x19221100);
        assert_eq!(bus_matrix.get32_aligned(RAM0_END - 4), 0x77175544);
        assert_eq!(bus_matrix.get32_aligned(RAM0_END), 0xBBAA1588);
        assert_eq!(bus_matrix.get32_aligned(RAM0_END + 4), 0xFFEEDD13);
    }

    #[test]
    fn get16() {
        let mut bus_matrix = BusMatrix::default();
        bus_matrix.set32_aligned(RAM0_END - 4, 0x44332211);
        bus_matrix.set32_aligned(RAM0_END, 0x88776655);

        assert_eq!(bus_matrix.get16(RAM0_END - 2), 0x4433);
        assert_eq!(bus_matrix.get16(RAM0_END - 1), 0x5544);
        assert_eq!(bus_matrix.get16(RAM0_END), 0x6655);
        assert_eq!(bus_matrix.get16(RAM0_END + 1), 0x7766);
    }

    #[test]
    fn set16() {
        let mut bus_matrix = BusMatrix::default();
        bus_matrix.set32_aligned(RAM0_END - 8, 0x33221100);
        bus_matrix.set32_aligned(RAM0_END - 4, 0x77665544);
        bus_matrix.set32_aligned(RAM0_END, 0xBBAA9988);
        bus_matrix.set32_aligned(RAM0_END + 4, 0xFFEEDDCC);

        bus_matrix.set16(RAM0_END - 5, 0x1234);
        bus_matrix.set16(RAM0_END - 2, 0x1357);
        bus_matrix.set16(RAM0_END + 1, 0x1468);
        bus_matrix.set16(RAM0_END + 4, 0x168A);

        assert_eq!(bus_matrix.get32_aligned(RAM0_END - 8), 0x34221100);
        assert_eq!(bus_matrix.get32_aligned(RAM0_END - 4), 0x13575512);
        assert_eq!(bus_matrix.get32_aligned(RAM0_END), 0xBB146888);
        assert_eq!(bus_matrix.get32_aligned(RAM0_END + 4), 0xFFEE168A);
    }

    #[test]
    fn get32() {
        let mut bus_matrix = BusMatrix::default();
        bus_matrix.set32_aligned(RAM0_END - 4, 0x44332211);
        bus_matrix.set32_aligned(RAM0_END, 0x88776655);

        assert_eq!(bus_matrix.get32(RAM0_END - 4), 0x44332211);
        assert_eq!(bus_matrix.get32(RAM0_END - 3), 0x55443322);
        assert_eq!(bus_matrix.get32(RAM0_END - 2), 0x66554433);
        assert_eq!(bus_matrix.get32(RAM0_END - 1), 0x77665544);
    }

    #[test]
    fn set32() {
        let mut bus_matrix = BusMatrix::default();
        bus_matrix.set32_aligned(RAM0_END - 16, 0x33221100);
        bus_matrix.set32_aligned(RAM0_END - 12, 0x77665544);
        bus_matrix.set32_aligned(RAM0_END - 8, 0xBBAA9988);
        bus_matrix.set32_aligned(RAM0_END - 4, 0xFFEEDDCC);
        bus_matrix.set32_aligned(RAM0_END, 0x76543210);
        bus_matrix.set32_aligned(RAM0_END + 4, 0xFEDCBA98);
        bus_matrix.set32_aligned(RAM0_END + 8, 0x01234567);

        bus_matrix.set32(RAM0_END - 16, 0xCAFEBABE);
        bus_matrix.set32(RAM0_END - 11, 0xDECACAFE);
        bus_matrix.set32(RAM0_END - 2, 0xDEADBEEF);
        bus_matrix.set32(RAM0_END + 7, 0x31415926);

        assert_eq!(bus_matrix.get32_aligned(RAM0_END - 16), 0xCAFEBABE);
        assert_eq!(bus_matrix.get32_aligned(RAM0_END - 12), 0xCACAFE44);
        assert_eq!(bus_matrix.get32_aligned(RAM0_END - 8), 0xBBAA99DE);
        assert_eq!(bus_matrix.get32_aligned(RAM0_END - 4), 0xBEEFDDCC);
        assert_eq!(bus_matrix.get32_aligned(RAM0_END), 0x7654DEAD);
        assert_eq!(bus_matrix.get32_aligned(RAM0_END + 4), 0x26DCBA98);
        assert_eq!(bus_matrix.get32_aligned(RAM0_END + 8), 0x01314159);
    }

    #[test]
    fn get32_aligned() {
        let mut bus_matrix = BusMatrix::default();
        set_boot_from_flash(&mut bus_matrix);
        bus_matrix.flash.set32_aligned(40, 123);
        bus_matrix.ram0.set32_aligned(20, 456);
        bus_matrix.ram1.set32_aligned(32, 789);

        assert_eq!(bus_matrix.get32_aligned(40), 123);
        assert_eq!(bus_matrix.get32_aligned(FLASH_BEGIN + 40), 123);
        assert_eq!(bus_matrix.get32_aligned(ROM_BEGIN + 16), 0x001000C7);
        assert_eq!(bus_matrix.get32_aligned(RAM0_BEGIN + 20), 456);
        assert_eq!(bus_matrix.get32_aligned(RAM0_BEGIN + RAM0_BYTES + 20), 456);
        assert_eq!(bus_matrix.get32_aligned(RAM1_BEGIN + 32), 789);
        assert_eq!(bus_matrix.get32_aligned(RAM1_BEGIN + RAM1_BYTES + 32), 789);
    }

    #[test]
    #[should_panic(expected = "Unsupported address 0x20100000")]
    fn get32_aligned_unsupported_address() {
        let mut bus_matrix = BusMatrix::default();
        bus_matrix.get32_aligned(RAM1_END);
    }

    #[test]
    fn set32_aligned() {
        let mut bus_matrix = BusMatrix::default();
        set_boot_from_flash(&mut bus_matrix);

        bus_matrix.set32_aligned(RAM0_BEGIN + RAM0_BYTES * 3 + 80, 456);
        bus_matrix.set32_aligned(RAM1_BEGIN + RAM1_BYTES * 7 + 120, 789);

        assert_eq!(bus_matrix.get32_aligned(FLASH_BEGIN + 40), 0xFFFFFFFF);
        assert_eq!(bus_matrix.get32_aligned(RAM0_BEGIN + 80), 456);
        assert_eq!(bus_matrix.get32_aligned(RAM1_BEGIN + 120), 789);
    }

    #[test]
    fn set32_aligned_peripherals() {
        let mut bus_matrix = BusMatrix::default();

        bus_matrix.set32_aligned(0x80100, 0xCAFEBABE);
        for i in 1..64 {
            bus_matrix.set32_aligned(0x80100 + 4 * i, 0x12345678);
        }
        bus_matrix.set32_aligned(crate::spi::MODE_REGISTER, 1);
        bus_matrix.set32_aligned(crate::usart::MODE_REGISTER, 2);
        bus_matrix.set32_aligned(crate::power::MAIN_OSCILLATOR_REGISTER, 0x00370008);
        bus_matrix.set32_aligned(crate::flash::MODE_REGISTER0, 0x600);
        bus_matrix.set32_aligned(crate::flash::COMMAND_REGISTER0, 0x5A000103);
        bus_matrix.set32_aligned(crate::pio::PIO_CONTROLLER_BEGIN + 0x70, 0x80);
        bus_matrix.set32_aligned(crate::reset::CONTROL_REGISTER, 3);
        bus_matrix.set32_aligned(crate::watchdog::MODE_REGISTER, 123);
        bus_matrix.set32_aligned(crate::time::RELOAD_VALUE_REGISTER, 456);
        bus_matrix.set32_aligned(crate::interrupt::SET_ENABLE_REGISTER0, 789);
        bus_matrix.set32_aligned(crate::system::VECTOR_TABLE_OFFSET_REGISTER, 0x20000000);

        assert_eq!(bus_matrix.get32_aligned(0x80100), 0xCAFEBABE);
        assert_eq!(bus_matrix.get32_aligned(crate::spi::MODE_REGISTER), 1);
        assert_eq!(bus_matrix.get32_aligned(crate::usart::MODE_REGISTER), 2);
        assert_eq!(
            bus_matrix.get32_aligned(crate::power::MAIN_OSCILLATOR_REGISTER),
            8
        );
        assert_eq!(
            bus_matrix.get32_aligned(crate::flash::MODE_REGISTER0),
            0x600
        );
        assert_eq!(
            bus_matrix.get32_aligned(crate::pio::PIO_CONTROLLER_BEGIN + 0x70),
            0x80
        );
        assert_eq!(bus_matrix.get32_aligned(crate::reset::CONTROL_REGISTER), 0);
        assert_eq!(
            bus_matrix.get32_aligned(crate::watchdog::MODE_REGISTER),
            123
        );
        assert_eq!(
            bus_matrix.get32_aligned(crate::time::RELOAD_VALUE_REGISTER),
            456
        );
        assert_eq!(
            bus_matrix.get32_aligned(crate::interrupt::SET_ENABLE_REGISTER0),
            789
        );
        assert_eq!(
            bus_matrix.get32_aligned(crate::system::VECTOR_TABLE_OFFSET_REGISTER),
            0x20000000
        );
        assert_eq!(
            bus_matrix
                .get_spi_controller_mut()
                .get32_aligned(crate::spi::MODE_REGISTER),
            1
        );
        assert_eq!(
            bus_matrix
                .get_usart_controller_mut()
                .get32_aligned(crate::usart::MODE_REGISTER),
            2
        );
        assert_eq!(
            bus_matrix
                .get_power_management_controller()
                .get32_aligned(crate::power::MAIN_OSCILLATOR_REGISTER),
            8
        );
        assert_eq!(
            bus_matrix
                .get_pio_controller()
                .get32_aligned(crate::pio::PIO_CONTROLLER_BEGIN + 0x70),
            0x80
        );
        assert_eq!(
            bus_matrix
                .get_pio_controller_mut()
                .get32_aligned(crate::pio::PIO_CONTROLLER_BEGIN + 0x70),
            0x80
        );
        assert_eq!(
            bus_matrix
                .get_reset_controler()
                .get32_aligned(crate::reset::CONTROL_REGISTER),
            0
        );
        assert_eq!(
            bus_matrix
                .get_system_controller()
                .get32_aligned(crate::system::VECTOR_TABLE_OFFSET_REGISTER),
            0x20000000
        );
    }

    #[test]
    #[should_panic(expected = "Unsupported address 0x20100000")]
    fn set32_aligned_unsupported_address() {
        let mut bus_matrix = BusMatrix::default();
        bus_matrix.set32_aligned(RAM1_END, 123);
    }

    #[test]
    fn decode_insns_boot_memory_boot_from_rom() {
        let mut bus_matrix = BusMatrix::default();

        bus_matrix.decode_insns(BOOT_BEGIN, 8);

        assert_eq!(bus_matrix.get_insn(BOOT_BEGIN), Instruction::Unsupported);
    }

    #[test]
    #[allow(clippy::unusual_byte_groupings)]
    fn decode_insns_flash_memory() {
        let mut bus_matrix = BusMatrix::default();
        set_boot_from_flash(&mut bus_matrix);
        bus_matrix.flash.set32_aligned(0, 0b0001_1000_00000011);
        bus_matrix
            .flash
            .set32_aligned(FLASH_BYTES - 4, 0b111110000100_0010 << 16);

        bus_matrix.decode_insns(0, 8);
        bus_matrix.decode_insns(FLASH_BEGIN - 8 * 2, 8);

        assert_eq!(
            bus_matrix.get_insn(0),
            Instruction::AddRdRnRm {
                rd: 0b011,
                rn: 0,
                rm: 0
            }
        );
        assert_eq!(bus_matrix.get_insn(FLASH_BEGIN - 2), Instruction::Unknown);
        assert_eq!(
            bus_matrix.get_insn(FLASH_BEGIN),
            Instruction::AddRdRnRm {
                rd: 0b011,
                rn: 0,
                rm: 0
            }
        );
        assert_eq!(bus_matrix.get_insn(FLASH_END - 2), Instruction::Unknown);
    }

    #[test]
    #[allow(clippy::unusual_byte_groupings)]
    fn decode_insns_ram0_memory() {
        let mut bus_matrix = BusMatrix::default();
        bus_matrix.set32(RAM0_BEGIN, 0b0001_1000_00000011);
        bus_matrix.set16(RAM0_END - 4, 0b010001110_0010_000);
        bus_matrix.set16(RAM0_END - 2, 0b111110000100_0010);

        bus_matrix.decode_insns(RAM0_BEGIN, 8);
        bus_matrix.decode_insns(RAM0_END - 8 * 2, 8);

        assert_eq!(
            bus_matrix.get_insn(RAM0_BEGIN),
            Instruction::AddRdRnRm {
                rd: 0b011,
                rn: 0,
                rm: 0
            }
        );
        assert_eq!(
            bus_matrix.get_insn(RAM0_BEGIN + 3 * RAM0_BYTES),
            Instruction::AddRdRnRm {
                rd: 0b011,
                rn: 0,
                rm: 0
            }
        );
        assert_eq!(
            bus_matrix.get_insn(RAM0_END - 4 - RAM0_BYTES),
            Instruction::BxRm { rm: 0b0010 }
        );
        assert_eq!(
            bus_matrix.get_insn(RAM0_END - 4),
            Instruction::BxRm { rm: 0b0010 }
        );
        assert_eq!(bus_matrix.get_insn(RAM0_END - 2), Instruction::Unknown);
    }

    #[test]
    #[allow(clippy::unusual_byte_groupings)]
    fn decode_insns_ram1_memory() {
        let mut bus_matrix = BusMatrix::default();
        bus_matrix.set32(RAM1_BEGIN, 0b0001_1000_00000011);
        bus_matrix.set16(RAM1_END - 4, 0b010001110_0010_000);
        bus_matrix.set16(RAM1_END - 2, 0b111110000100_0010);

        bus_matrix.decode_insns(RAM1_BEGIN, 8);
        bus_matrix.decode_insns(RAM1_END - 8 * 2, 8);

        assert_eq!(
            bus_matrix.get_insn(RAM1_BEGIN),
            Instruction::AddRdRnRm {
                rd: 0b011,
                rn: 0,
                rm: 0
            }
        );
        assert_eq!(
            bus_matrix.get_insn(RAM1_BEGIN + 3 * RAM1_BYTES),
            Instruction::AddRdRnRm {
                rd: 0b011,
                rn: 0,
                rm: 0
            }
        );
        assert_eq!(
            bus_matrix.get_insn(RAM1_END - 4 - RAM1_BYTES),
            Instruction::BxRm { rm: 0b0010 }
        );
        assert_eq!(
            bus_matrix.get_insn(RAM1_END - 4),
            Instruction::BxRm { rm: 0b0010 }
        );
        assert_eq!(bus_matrix.get_insn(RAM1_END - 2), Instruction::Unknown);
    }

    #[test]
    fn decode_insns_rom_memory() {
        let mut bus_matrix = BusMatrix::default();

        bus_matrix.decode_insns(ROM_BEGIN, 8);

        assert_eq!(bus_matrix.get_insn(ROM_BEGIN), Instruction::Unsupported);
    }

    #[test]
    fn reset() {
        let mut bus_matrix = BusMatrix::default();
        bus_matrix.flash.set32_aligned(4, 456);

        bus_matrix.reset();

        assert_eq!(bus_matrix.flash.get32_aligned(4), 456);
    }
}
