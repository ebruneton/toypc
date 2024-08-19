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

use std::{cell::RefCell, rc::Rc};

pub const PIO_CONTROLLER_BEGIN: u32 = 0x400E0E00;
pub const PIO_CONTROLLER_END: u32 = 0x400E1600;
pub const PIO_CONTROLLER_LAST: u32 = PIO_CONTROLLER_END - 1;

pub enum Controller {
    PA,
    PB,
    PC,
    PD,
}

/// An external device which can be connected to the output pins of the micro-controller.
pub trait PioDevice {
    /// Method called by the micro-controller emulator when at least one pin changed state. Pins are
    /// identified by a controller in [0..3], and a bit number in this controller [0..31].
    fn pio_state_changed(&mut self, pins: &[u32; 4]);
}

/// An empty PIO device, which does nothing.
#[derive(Default)]
pub struct EmptyPioDevice {}

impl PioDevice for EmptyPioDevice {
    fn pio_state_changed(&mut self, _pins: &[u32; 4]) {}
}

#[derive(Clone)]
enum Kind {
    /// Write-only register, writing to it sets bits in the 'value' ReadOnly register.
    Set,
    /// Write-only register, writing to it clears bits in the 'value' ReadOnly register.
    Clear,
    /// Read-only register, holds a value which can be changed with Set and Clear registers.
    ReadOnly,
    /// Read-write register, holds a value which can be changed by directly.
    ReadWrite,
    Reserved,
    Unsupported,
}

#[derive(Clone)]
struct Register {
    kind: Kind,
    value: u32,
}

impl Register {
    fn new(kind: Kind, value: u32) -> Self {
        Self { kind, value }
    }
}

/// The Parallel Input/Output Controller. See section 31, p618 of the Atmel SAM3X Datasheet.
/// This implementation does not support interrupts.
#[derive(Clone)]
pub struct ParallelIoController {
    registers: Vec<Register>,
    pio_device: Rc<RefCell<dyn PioDevice>>,
}

impl ParallelIoController {
    const PIO_STATUS_REGISTER_INDEX: usize = 0x08 / 4;
    const OUTPUT_STATUS_REGISTER_INDEX: usize = 0x18 / 4;
    const OUTPUT_DATA_STATUS_REGISTER_INDEX: usize = 0x38 / 4;
    const PIN_DATA_STATUS_REGISTER_INDEX: usize = 0x3C / 4;
    const PULL_UP_STATUS_REGISTER_INDEX: usize = 0x68 / 4;
    const AB_SELECT_REGISTER_INDEX: usize = 0x70 / 4;

    const REGISTER_COUNT_BY_CONTROLLER: usize = 0x200 / 4;

    pub fn uninitialized() -> Self {
        let mut registers = Vec::new();
        Self::add_controller(&mut registers);
        Self::add_controller(&mut registers);
        Self::add_controller(&mut registers);
        Self::add_controller(&mut registers);
        debug_assert!(registers.len() == 4 * Self::REGISTER_COUNT_BY_CONTROLLER);
        Self {
            registers,
            pio_device: Rc::new(RefCell::new(EmptyPioDevice::default())),
        }
    }

    #[cfg(test)]
    pub fn new() -> Self {
        let mut result = Self::uninitialized();
        result.reset();
        result
    }

    pub fn get_pio_device(&self) -> Rc<RefCell<dyn PioDevice>> {
        self.pio_device.clone()
    }

    pub fn set_pio_device(&mut self, pio_device: Rc<RefCell<dyn PioDevice>>) {
        self.pio_device = pio_device;
    }

    fn add_controller(registers: &mut Vec<Register>) {
        // See section 31.7 Parallel Input/Output Controller (PIO) User Interface.

        // 0x0000 PIO Enable Register PIO_PER Write-only
        // 0x0004 PIO Disable Register PIO_PDR Write-only
        // 0x0008 PIO Status Register PIO_PSR Read-only
        // 0x000C Reserved
        Self::add_set_clear_registers(registers);

        // 0x0010 Output Enable Register PIO_OER Write-only
        // 0x0014 Output Disable Register PIO_ODR Write-only
        // 0x0018 Output Status Register PIO_OSR Read-only
        // 0x001C Reserved
        Self::add_set_clear_registers(registers);

        // 0x0020 Glitch Input Filter Enable Register PIO_IFER Write-only
        // 0x0024 Glitch Input Filter Disable Register PIO_IFDR Write-only
        // 0x0028 Glitch Input Filter Status Register PIO_IFSR Read-only
        // 0x002C Reserved
        Self::add_set_clear_registers(registers);

        // 0x0030 Set Output Data Register PIO_SODR Write-only
        // 0x0034 Clear Output Data Register PIO_CODR Write-only
        // 0x0038 Output Data Status Register PIO_ODSR Read-only or Read-write
        // 0x003C Pin Data Status Register PIO_PDSR Read-only
        Self::add_set_clear_registers(registers);
        registers.pop();
        registers.push(Register::new(Kind::ReadOnly, 0));

        // 0x0040 Interrupt Enable Register PIO_IER Write-only
        // 0x0044 Interrupt Disable Register PIO_IDR Write-only
        // 0x0048 Interrupt Mask Register PIO_IMR Read-only
        // 0x004C Interrupt Status Register PIO_ISR Read-only
        Self::add_set_clear_registers(registers);
        registers.pop();
        registers.push(Register::new(Kind::ReadOnly, 0));

        // 0x0050 Multi-driver Enable Register PIO_MDER Write-only
        // 0x0054 Multi-driver Disable Register PIO_MDDR Write-only
        // 0x0058 Multi-driver Status Register PIO_MDSR Read-only
        // 0x005C Reserved
        registers.push(Register::new(Kind::Unsupported, 0));
        registers.push(Register::new(Kind::Unsupported, 0));
        registers.push(Register::new(Kind::Unsupported, 0));
        registers.push(Register::new(Kind::Reserved, 0));

        // 0x0060 Pull-up Disable Register PIO_PUDR Write-only
        // 0x0064 Pull-up Enable Register PIO_PUER Write-only
        // 0x0068 Pad Pull-up Status Register PIO_PUSR Read-only
        // 0x006C Reserved
        Self::add_set_clear_registers(registers);

        // 0x0070 Peripheral AB Select Register Read-Write
        // 0x0074 to 0x007C Reserved
        registers.push(Register::new(Kind::ReadWrite, 0));
        registers.push(Register::new(Kind::Reserved, 0));
        registers.push(Register::new(Kind::Reserved, 0));
        registers.push(Register::new(Kind::Reserved, 0));

        // 0x0080 System Clock Glitch Input Filter Select Register PIO_SCIFSR Write-only
        // 0x0084 Debouncing Input Filter Select Register PIO_DIFSR Write-only
        // 0x0088 Input Filter Clock Selection Status Register PIO_IFDGSR Read-only
        // 0x008C Slow Clock Divider Debouncing Register PIO_SCDR Read-Write
        Self::add_clear_set_registers(registers);
        registers.pop();
        registers.push(Register::new(Kind::ReadWrite, 0));

        // 0x0090 to 0x009C Reserved
        registers.push(Register::new(Kind::Reserved, 0));
        registers.push(Register::new(Kind::Reserved, 0));
        registers.push(Register::new(Kind::Reserved, 0));
        registers.push(Register::new(Kind::Reserved, 0));

        // 0x00A0 Output Write Enable PIO_OWER Write-only
        // 0x00A4 Output Write Disable PIO_OWDR Write-only
        // 0x00A8 Output Write Status Register PIO_OWSR Read-only
        // 0x00AC Reserved
        Self::add_set_clear_registers(registers);

        // 0x00B0 Additional Interrupt Modes Enable Register PIO_AIMER Write-only
        // 0x00B4 Additional Interrupt Modes Disables Register PIO_AIMDR Write-only
        // 0x00B8 Additional Interrupt Modes Mask Register PIO_AIMMR Read-only
        // 0x00BC Reserved
        Self::add_set_clear_registers(registers);

        // 0x00C0 Edge Select Register PIO_ESR Write-only
        // 0x00C4 Level Select Register PIO_LSR Write-only
        // 0x00C8 Edge/Level Status Register PIO_ELSR Read-only
        // 0x00CC Reserved
        Self::add_clear_set_registers(registers);

        // 0x00D0 Falling Edge/Low Level Select Register PIO_FELLSR Write-only
        // 0x00D4 Rising Edge/ High Level Select Register PIO_REHLSR Write-only
        // 0x00D8 Fall/Rise - Low/High Status Register PIO_FRLHSR Read-only
        // 0x00DC Reserved
        Self::add_clear_set_registers(registers);

        // 0x00E0 Lock Status PIO_LOCKSR Read-only
        // 0x00E4 Write Protect Mode Register PIO_WPMR Read-write
        // 0x00E8 Write Protect Status Register PIO_WPSR Read-only
        registers.push(Register::new(Kind::Unsupported, 0));
        registers.push(Register::new(Kind::Unsupported, 0));
        registers.push(Register::new(Kind::Unsupported, 0));

        // 0x00EC to 0x00F8 Reserved
        // 0x0100 to 0x0144 Reserved
        // and start of next controller register at 0x0200.
        for _ in 0..69 {
            registers.push(Register::new(Kind::Reserved, 0));
        }
    }

    fn add_set_clear_registers(registers: &mut Vec<Register>) {
        let target = (registers.len() + 2) as u32;
        registers.push(Register::new(Kind::Set, target));
        registers.push(Register::new(Kind::Clear, target));
        registers.push(Register::new(Kind::ReadOnly, 0));
        registers.push(Register::new(Kind::Reserved, 0));
    }

    fn add_clear_set_registers(registers: &mut Vec<Register>) {
        let target = (registers.len() + 2) as u32;
        registers.push(Register::new(Kind::Clear, target));
        registers.push(Register::new(Kind::Set, target));
        registers.push(Register::new(Kind::ReadOnly, 0));
        registers.push(Register::new(Kind::Reserved, 0));
    }

    pub fn get32_aligned(&self, address: u32) -> u32 {
        debug_assert!(address % 4 == 0);
        let index = (address - PIO_CONTROLLER_BEGIN) >> 2;
        let register = &self.registers[index as usize];
        match register.kind {
            Kind::Set | Kind::Clear | Kind::Reserved | Kind::Unsupported => 0,
            Kind::ReadOnly | Kind::ReadWrite => register.value,
        }
    }

    pub fn set32_aligned(&mut self, address: u32, new_value: u32) {
        debug_assert!(address % 4 == 0);
        let index = (address - PIO_CONTROLLER_BEGIN) >> 2;
        let old_controller_outputs = [
            self.get_controller_outputs(0),
            self.get_controller_outputs(1),
            self.get_controller_outputs(2),
            self.get_controller_outputs(3),
        ];
        match self.registers[index as usize].kind {
            Kind::Set => {
                let target = self.registers[index as usize].value;
                self.registers[target as usize].value |= new_value;
            }
            Kind::Clear => {
                let target = self.registers[index as usize].value;
                self.registers[target as usize].value &= !new_value;
            }
            Kind::ReadOnly => (),
            Kind::ReadWrite => {
                self.registers[index as usize].value = new_value;
            }
            Kind::Reserved => (),
            Kind::Unsupported => panic!(
                "Unsupported PIO register {:#010X}",
                PIO_CONTROLLER_BEGIN + 4 * index
            ),
        }
        let new_controller_outputs = [
            self.get_controller_outputs(0),
            self.get_controller_outputs(1),
            self.get_controller_outputs(2),
            self.get_controller_outputs(3),
        ];
        if new_controller_outputs != old_controller_outputs {
            self.pio_device
                .borrow_mut()
                .pio_state_changed(&new_controller_outputs);
        }
    }

    pub fn get_pin_output(&self, controller: Controller, pin: u32) -> bool {
        self.get_controller_outputs(controller as usize) & (1 << pin) != 0
    }

    pub fn spi_output_pins_enabled(&self) -> bool {
        // SPI pins are controlled with bits 25, 26, 27 and 28 of PIOA controller, peripheral A
        // (See Section 9.3.1 PIO Controller A Multiplexing, Table 9-2). For all these pins,
        // peripheral A must be selected and enabled, and the pull-up must be enabled.
        const SPI_BITS: u32 = 0xF << 25;
        (self.registers[Self::AB_SELECT_REGISTER_INDEX].value & SPI_BITS == 0)
            && (self.registers[Self::PIO_STATUS_REGISTER_INDEX].value & SPI_BITS == 0)
            && (self.registers[Self::PULL_UP_STATUS_REGISTER_INDEX].value & SPI_BITS == 0)
    }

    pub fn usart_input_pins_enabled(&self) -> bool {
        // USART0 input pins are controlled with bits 10 (RXD line) and 17 (CLOCK line) of the PIOA
        // controller (See Section 9.3.1 PIO Controller A Multiplexing, Table 9-2). These two pins
        // must be configured as input, with a pull-up enabled.
        const USART0_BITS: u32 = 1 << 17 | 1 << 10;
        (self.registers[Self::PIO_STATUS_REGISTER_INDEX].value
            & self.registers[Self::OUTPUT_STATUS_REGISTER_INDEX].value
            & USART0_BITS
            == 0)
            && (self.registers[Self::PULL_UP_STATUS_REGISTER_INDEX].value & USART0_BITS == 0)
    }

    fn get_controller_outputs(&self, controller: usize) -> u32 {
        let output_status = self.get_register_value(controller, Self::OUTPUT_STATUS_REGISTER_INDEX);
        let pio_status = self.get_register_value(controller, Self::PIO_STATUS_REGISTER_INDEX);
        let data = self.get_register_value(controller, Self::OUTPUT_DATA_STATUS_REGISTER_INDEX);
        let pullup = self.get_register_value(controller, Self::PULL_UP_STATUS_REGISTER_INDEX);
        // If output is enabled on a line, and if the PIO controler controls the output (i.e. not
        // an embedded peripheral), then the output value is the one stored in the output data
        // status register.
        // If output is enabled on a line, but the output is controlled by an embedded peripheral,
        // we return 0 (we don't support this case).
        // If output is not enabled on a line, the output is 1 if the pull-up is enabled (i.e. if
        // the PULL_UP_STATUS bit is 0), otherwise it is undefined (we assume 0).
        (output_status & pio_status & data) | (!output_status & !pullup)
    }

    fn get_register_value(&self, controller: usize, base_register: usize) -> u32 {
        let register = controller * Self::REGISTER_COUNT_BY_CONTROLLER + base_register;
        self.registers[register].value
    }

    fn get_register_value_mut(&mut self, controller: usize, base_register: usize) -> &mut u32 {
        let register = controller * Self::REGISTER_COUNT_BY_CONTROLLER + base_register;
        &mut self.registers[register].value
    }

    pub fn reset(&mut self) {
        use Controller::*;
        for register in &mut self.registers {
            match register.kind {
                Kind::ReadOnly => register.value = 0,
                Kind::ReadWrite => register.value = 0,
                _ => (),
            }
        }
        *self.get_register_value_mut(PA as usize, Self::PIO_STATUS_REGISTER_INDEX) = 0xFFFFFFFF;
        *self.get_register_value_mut(PB as usize, Self::PIO_STATUS_REGISTER_INDEX) = 0x0FFFFFFF;
        *self.get_register_value_mut(PC as usize, Self::PIO_STATUS_REGISTER_INDEX) = 0x7FFFFFFE;
        *self.get_register_value_mut(PD as usize, Self::PIO_STATUS_REGISTER_INDEX) = 0x7FFFFFFF;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pio::PIO_CONTROLLER_BEGIN;

    #[derive(Default)]
    pub struct LogPioDevice {
        states: Vec<bool>,
    }

    impl PioDevice for LogPioDevice {
        fn pio_state_changed(&mut self, pins: &[u32; 4]) {
            self.states
                .push(pins[Controller::PB as usize] & (1 << 27) != 0);
        }
    }

    #[test]
    fn get_set_pio_device() {
        let mut pio = ParallelIoController::new();
        let log_pio_device = Rc::new(RefCell::new(LogPioDevice::default()));

        pio.set_pio_device(log_pio_device.clone());
        let pio_device = pio.get_pio_device();

        assert!(Rc::ptr_eq(
            &pio_device,
            &(log_pio_device as Rc<RefCell<dyn PioDevice>>)
        ));
    }

    #[test]
    fn get32_aligned_set_register() {
        const PIOB_ENABLE_REGISTER: u32 = 0x400E1000;
        let pio = ParallelIoController::new();

        assert_eq!(pio.get32_aligned(PIOB_ENABLE_REGISTER), 0);
    }

    #[test]
    fn get32_aligned_clear_register() {
        const PIOB_DISABLE_REGISTER: u32 = 0x400E1004;
        let pio = ParallelIoController::new();

        assert_eq!(pio.get32_aligned(PIOB_DISABLE_REGISTER), 0);
    }

    #[test]
    fn get32_aligned_read_only_register() {
        const PIOB_STATUS_REGISTER: u32 = 0x400E1008;
        let mut pio = ParallelIoController::new();
        pio.registers[130].value = 123;

        assert_eq!(pio.get32_aligned(PIOB_STATUS_REGISTER), 123);
    }

    #[test]
    fn get32_aligned_read_write_register() {
        const PIOB_AB_SELECT_REGISTER: u32 = 0x400E1070;
        let mut pio = ParallelIoController::new();
        pio.registers[156].value = 123;

        assert_eq!(pio.get32_aligned(PIOB_AB_SELECT_REGISTER), 123);
    }

    #[test]
    fn get32_aligned_reserved_register() {
        const PIOB_RESERVED_REGISTER: u32 = 0x400E100C;
        let pio = ParallelIoController::new();

        assert_eq!(pio.get32_aligned(PIOB_RESERVED_REGISTER), 0);
    }

    #[test]
    fn get32_aligned_unsupported_register() {
        const PIOB_RESERVED_REGISTER: u32 = 0x400E1050;
        let pio = ParallelIoController::new();

        assert_eq!(pio.get32_aligned(PIOB_RESERVED_REGISTER), 0);
    }

    #[test]
    fn set32_aligned_set_register() {
        const PIOB_ENABLE_REGISTER: u32 = 0x400E1000;
        const PIOB_STATUS_REGISTER: u32 = 0x400E1008;
        let mut pio = ParallelIoController::new();
        pio.registers[130].value = 0x12;

        pio.set32_aligned(PIOB_ENABLE_REGISTER, 0xFF00);

        assert_eq!(pio.get32_aligned(PIOB_STATUS_REGISTER), 0xFF12);
    }

    #[test]
    fn set32_aligned_clear_register() {
        const PIOB_DISABLE_REGISTER: u32 = 0x400E1004;
        const PIOB_STATUS_REGISTER: u32 = 0x400E1008;
        let mut pio = ParallelIoController::new();
        pio.registers[130].value = 0xFF;

        pio.set32_aligned(PIOB_DISABLE_REGISTER, 0x12);

        assert_eq!(pio.get32_aligned(PIOB_STATUS_REGISTER), 0xED);
    }

    #[test]
    fn set32_aligned_read_only_register() {
        const PIOB_STATUS_REGISTER: u32 = 0x400E1008;
        let mut pio = ParallelIoController::new();

        pio.set32_aligned(PIOB_STATUS_REGISTER, 123);

        assert_eq!(pio.registers[130].value, 0x0FFFFFFF);
    }

    #[test]
    fn set32_aligned_read_write_register() {
        const PIOB_AB_SELECT_REGISTER: u32 = 0x400E1070;
        let mut pio = ParallelIoController::new();
        pio.registers[156].value = 456;

        pio.set32_aligned(PIOB_AB_SELECT_REGISTER, 123);

        assert_eq!(pio.registers[156].value, 123);
    }

    #[test]
    fn set32_aligned_reserved_register() {
        const PIOB_RESERVED_REGISTER: u32 = 0x400E100C;
        let mut pio = ParallelIoController::new();

        pio.set32_aligned(PIOB_RESERVED_REGISTER, 123);

        assert_eq!(pio.registers[131].value, 0);
    }

    #[test]
    #[should_panic(expected = "Unsupported PIO register 0x400E1050")]
    fn set32_aligned_unsupported_register() {
        const PIOB_RESERVED_REGISTER: u32 = 0x400E1050;
        let mut pio = ParallelIoController::new();

        pio.set32_aligned(PIOB_RESERVED_REGISTER, 123);
    }

    #[test]
    fn get_pin_output() {
        const PB27: u32 = 1 << 27;
        let mut pio = ParallelIoController::new();
        pio.set32_aligned(0x400E1004, PB27); // PIO Disable Register (PDR)
        let mut states = Vec::new();

        states.push(pio.get_pin_output(Controller::PB, 27));
        pio.set32_aligned(0x400E1010, PB27); // Output Enable Register (OER)
        states.push(pio.get_pin_output(Controller::PB, 27));
        pio.set32_aligned(0x400E1060, PB27); // Pull-up Disable Register (PUDR)
        states.push(pio.get_pin_output(Controller::PB, 27));
        pio.set32_aligned(0x400E1030, PB27); // Set Output Data Register (SODR)
        states.push(pio.get_pin_output(Controller::PB, 27));
        pio.set32_aligned(0x400E1000, PB27); // PIO Enable Register (PER)
        states.push(pio.get_pin_output(Controller::PB, 27));
        pio.set32_aligned(0x400E1034, PB27); // Clear Output Data Register (CODR)
        states.push(pio.get_pin_output(Controller::PB, 27));

        assert_eq!(states, vec![true, false, false, false, true, false]);
    }

    #[test]
    fn get_pin_output_depends_on_pull_up_if_output_disabled() {
        const PB27: u32 = 1 << 27;
        let mut pio = ParallelIoController::new();
        let mut states = Vec::new();

        states.push(pio.get_pin_output(Controller::PB, 27));
        pio.set32_aligned(0x400E1060, PB27); // Pull-up Disable Register (PUDR)
        states.push(pio.get_pin_output(Controller::PB, 27));

        assert_eq!(states, vec![true, false]);
    }

    #[test]
    fn spi_output_pins_enabled_ok() {
        const PIOA_DISABLE_REGISTER: u32 = 0x400E0E04;
        let mut pio = ParallelIoController::new();
        pio.set32_aligned(PIOA_DISABLE_REGISTER, 0xFFFFFFFF);

        assert!(pio.spi_output_pins_enabled());
    }

    #[test]
    fn spi_output_pins_enabled_pio_enabled() {
        const PIOA_DISABLE_REGISTER: u32 = 0x400E0E04;
        const ENABLE_REGISTER: u32 = 0x400E0E00;
        let mut pio = ParallelIoController::new();
        pio.set32_aligned(PIOA_DISABLE_REGISTER, 0xFFFFFFFF);

        pio.set32_aligned(ENABLE_REGISTER, 1 << 25);

        assert!(!pio.spi_output_pins_enabled());
    }

    #[test]
    fn spi_output_pins_enabled_pullup_disabled() {
        const PIOA_DISABLE_REGISTER: u32 = 0x400E0E04;
        const PULL_UP_DISABLE_REGISTER: u32 = 0x400E0E60;
        let mut pio = ParallelIoController::new();
        pio.set32_aligned(PIOA_DISABLE_REGISTER, 0xFFFFFFFF);

        pio.set32_aligned(PULL_UP_DISABLE_REGISTER, 1 << 26);

        assert!(!pio.spi_output_pins_enabled());
    }

    #[test]
    fn spi_output_pins_enabled_peripheral_b_selected() {
        const PIOA_DISABLE_REGISTER: u32 = 0x400E0E04;
        const AB_SELECT_REGISTER: u32 = 0x400E0E70;
        let mut pio = ParallelIoController::new();
        pio.set32_aligned(PIOA_DISABLE_REGISTER, 0xFFFFFFFF);

        pio.set32_aligned(AB_SELECT_REGISTER, 1 << 27);

        assert!(!pio.spi_output_pins_enabled());
    }

    #[test]
    fn usart_input_pins_enabled_ok() {
        const PIOA_ENABLE_REGISTER: u32 = 0x400E0E00;
        let mut pio = ParallelIoController::new();
        pio.set32_aligned(PIOA_ENABLE_REGISTER, 0xFFFFFFFF);

        assert!(pio.usart_input_pins_enabled());
    }

    #[test]
    fn usart_input_pins_enabled_pio_enabled() {
        const PIOA_ENABLE_REGISTER: u32 = 0x400E0E00;
        const OUTPUT_ENABLE_REGISTER: u32 = 0x400E0E10;
        let mut pio = ParallelIoController::new();
        pio.set32_aligned(PIOA_ENABLE_REGISTER, 0xFFFFFFFF);
        pio.set32_aligned(OUTPUT_ENABLE_REGISTER, 1 << 17);

        assert!(!pio.usart_input_pins_enabled());
    }

    #[test]
    fn usart_input_pins_enabled_pullup_disabled() {
        const PIOA_DISABLE_REGISTER: u32 = 0x400E0E04;
        const AB_SELECT_REGISTER: u32 = 0x400E0E70;
        const PULL_UP_DISABLE_REGISTER: u32 = 0x400E0E60;
        let mut pio = ParallelIoController::new();
        pio.set32_aligned(PIOA_DISABLE_REGISTER, 0xFFFFFFFF);
        pio.set32_aligned(AB_SELECT_REGISTER, 1 << 17);

        pio.set32_aligned(PULL_UP_DISABLE_REGISTER, 1 << 17);

        assert!(!pio.usart_input_pins_enabled());
    }

    #[test]
    fn pio_device_pin_state_changed() {
        const PB27: u32 = 1 << 27;
        let mut pio = ParallelIoController::new();
        let log_pio_device = Rc::new(RefCell::new(LogPioDevice::default()));
        pio.set_pio_device(log_pio_device.clone());

        pio.set32_aligned(0x400E1010, PB27); // Output Enable Register (OER)
                                             // State changed to LOW.
        pio.set32_aligned(0x400E1060, PB27); // Pull-up Disable Register (PUDR)
        pio.set32_aligned(0x400E1030, PB27); // Set Output Data Register (SODR)
        pio.set32_aligned(0x400E1000, PB27); // PIO Enable Register (PER)
                                             // State changed to HIGH.
        pio.set32_aligned(0x400E1034, PB27); // Clear Output Data Register (CODR)
                                             // State changed to LOW.

        assert_eq!(log_pio_device.borrow().states, vec![false, true, false]);
    }

    #[test]
    fn reset() {
        const PIOB_STATUS_REGISTER: u32 = 0x400E1008;
        const PIOB_AB_SELECT_REGISTER: u32 = 0x400E1070;
        let mut pio = ParallelIoController::new();
        pio.registers[(PIOB_STATUS_REGISTER - PIO_CONTROLLER_BEGIN) as usize / 4].value = 123;
        pio.registers[(PIOB_AB_SELECT_REGISTER - PIO_CONTROLLER_BEGIN) as usize / 4].value = 456;

        pio.reset();

        assert_eq!(pio.get32_aligned(PIOB_STATUS_REGISTER), 0x0FFFFFFF);
        assert_eq!(pio.get32_aligned(PIOB_AB_SELECT_REGISTER), 0);
    }
}
