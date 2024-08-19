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

pub const SPI_CONTROLLER_BEGIN: u32 = 0x40008000;
pub const SPI_CONTROLLER_END: u32 = 0x40008100;
pub const SPI_CONTROLLER_LAST: u32 = SPI_CONTROLLER_END - 1;

pub const CONTROL_REGISTER: u32 = 0x40008000;
pub const MODE_REGISTER: u32 = 0x40008004;
pub const RECEIVE_DATA_REGISTER: u32 = 0x40008008;
pub const TRANSMIT_DATA_REGISTER: u32 = 0x4000800C;
pub const STATUS_REGISTER: u32 = 0x40008010;
pub const CHIP_SELECT0_REGISTER: u32 = 0x40008030;

// Control Register flags.
const SPI_ENABLE: u32 = 0x01;
const SPI_DISABLE: u32 = 0x02;
const SPI_SOFTWARE_RESET: u32 = 0x80;

// Mode Register flags.
const MASTER_SLAVE_MODE: u32 = 0x01;
const PERIPHERAL_SELECT: u32 = 0x02;
const CHIP_SELECT_DECODE: u32 = 0x04;
const PERIPHERAL_CHIP_SELECT: u32 = 0xF0000;
const MODE_BITS: u32 = 0xFF0F00B7;

// Status Register flags.
const RECEIVE_DATA_REGISTER_FULL: u32 = 0x01;
const TRANSMIT_DATA_REGISTER_EMPTY: u32 = 0x02;
const OVERRUN_ERROR_STATUS: u32 = 0x08;
const SPI_ENABLE_STATUS: u32 = 0x10000;

/// An external device which can be connected to the Serial Peripheral Interface of the
/// micro-controller. This implementation only supports master mode, with a fixed peripheral
/// select, without chip select decode, and for chip 0 only. It does not support interrupts.
pub trait SpiDevice {
    /// Receives data transmitted from the micro-controller and returns the data to send back.
    fn data_received(&mut self, data: u32, chip_select: u32) -> Option<u32>;
}

/// An empty SPI device, which does nothing and does not send responses to received requests.
#[derive(Default)]
pub struct EmptySpiDevice {}

impl SpiDevice for EmptySpiDevice {
    fn data_received(&mut self, _data: u32, _chip_select: u32) -> Option<u32> {
        Option::None
    }
}

/// The Serial Peripheral Interface (SPI0). See section 32, p676 of the Atmel SAM3X Datasheet.
#[derive(Clone)]
pub struct SerialPeripheralInterfaceController {
    mode: u32,
    received_data: u32,
    status: u32,
    chip_select0: u32,
    spi_device: Rc<RefCell<dyn SpiDevice>>,
}

impl SerialPeripheralInterfaceController {
    const CONTROL_REGISTER_INDEX: u32 = (CONTROL_REGISTER - SPI_CONTROLLER_BEGIN) / 4;
    const MODE_REGISTER_INDEX: u32 = (MODE_REGISTER - SPI_CONTROLLER_BEGIN) / 4;
    const RECEIVE_DATA_REGISTER_INDEX: u32 = (RECEIVE_DATA_REGISTER - SPI_CONTROLLER_BEGIN) / 4;
    const TRANSMIT_DATA_REGISTER_INDEX: u32 = (TRANSMIT_DATA_REGISTER - SPI_CONTROLLER_BEGIN) / 4;
    const STATUS_REGISTER_INDEX: u32 = (STATUS_REGISTER - SPI_CONTROLLER_BEGIN) / 4;
    const CHIP_SELECT0_REGISTER_INDEX: u32 = (CHIP_SELECT0_REGISTER - SPI_CONTROLLER_BEGIN) / 4;

    pub fn new() -> Self {
        Self {
            mode: 0,
            received_data: 0,
            status: 0,
            chip_select0: 0,
            spi_device: Rc::new(RefCell::new(EmptySpiDevice::default())),
        }
    }

    pub fn get_spi_device(&self) -> Rc<RefCell<dyn SpiDevice>> {
        self.spi_device.clone()
    }

    pub fn set_spi_device(&mut self, spi_device: Rc<RefCell<dyn SpiDevice>>) {
        self.spi_device = spi_device;
    }

    pub fn get32_aligned(&mut self, address: u32) -> u32 {
        debug_assert!(address % 4 == 0);
        let index = (address - SPI_CONTROLLER_BEGIN) >> 2;
        match index {
            Self::CONTROL_REGISTER_INDEX => 0,
            Self::MODE_REGISTER_INDEX => self.mode,
            Self::RECEIVE_DATA_REGISTER_INDEX => {
                self.status &= !RECEIVE_DATA_REGISTER_FULL;
                self.received_data
            }
            Self::TRANSMIT_DATA_REGISTER_INDEX => 0,
            Self::STATUS_REGISTER_INDEX => {
                let result = self.status;
                self.status &= !OVERRUN_ERROR_STATUS;
                result
            }
            Self::CHIP_SELECT0_REGISTER_INDEX => self.chip_select0,
            _ => panic!("Unsupported SPI register {address:#010X}"),
        }
    }

    pub fn set32_aligned(
        &mut self,
        address: u32,
        value: u32,
        clock_enabled: bool,
        output_enabled: bool,
    ) {
        debug_assert!(address % 4 == 0);
        let index = (address - SPI_CONTROLLER_BEGIN) >> 2;
        match index {
            Self::CONTROL_REGISTER_INDEX => {
                if value & SPI_DISABLE != 0 {
                    self.status = 0;
                } else if value & SPI_ENABLE != 0 && clock_enabled {
                    self.status |= SPI_ENABLE_STATUS | TRANSMIT_DATA_REGISTER_EMPTY;
                }
                if value & SPI_SOFTWARE_RESET != 0 {
                    self.reset();
                }
            }
            Self::MODE_REGISTER_INDEX => {
                if value & (PERIPHERAL_SELECT | CHIP_SELECT_DECODE | PERIPHERAL_CHIP_SELECT) != 0 {
                    panic!("Unsupported SPI Mode register value {value:#010X}");
                }
                self.mode = value & MODE_BITS;
            }
            Self::RECEIVE_DATA_REGISTER_INDEX => (),
            Self::TRANSMIT_DATA_REGISTER_INDEX => {
                if !(output_enabled
                    && (self.status & SPI_ENABLE_STATUS != 0)
                    && (self.mode & MASTER_SLAVE_MODE != 0))
                {
                    return;
                }
                let received = self
                    .spi_device
                    .borrow_mut()
                    .data_received(value, self.chip_select0);
                if let Option::Some(data) = received {
                    self.received_data = data;
                    if self.status & RECEIVE_DATA_REGISTER_FULL != 0 {
                        self.status |= OVERRUN_ERROR_STATUS;
                    }
                    self.status |= RECEIVE_DATA_REGISTER_FULL;
                }
            }
            Self::STATUS_REGISTER_INDEX => (),
            Self::CHIP_SELECT0_REGISTER_INDEX => self.chip_select0 = value,
            _ => panic!("Unsupported SPI register {address:#010X}"),
        }
    }

    pub fn reset(&mut self) {
        self.mode = 0;
        self.status = 0;
        self.received_data = 0;
        self.chip_select0 = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::GraphicsCard;

    #[derive(Default)]
    pub struct IncrementSpiDevice {}

    impl SpiDevice for IncrementSpiDevice {
        fn data_received(&mut self, data: u32, _chip_select: u32) -> Option<u32> {
            Option::Some(data + 1)
        }
    }

    #[test]
    fn get_set_spi_device() {
        let mut spi = SerialPeripheralInterfaceController::new();
        let increment_spi_device = Rc::new(RefCell::new(IncrementSpiDevice::default()));

        spi.set_spi_device(increment_spi_device.clone());
        let spi_device = spi.get_spi_device();

        assert!(Rc::ptr_eq(
            &spi_device,
            &(increment_spi_device as Rc<RefCell<dyn SpiDevice>>)
        ));
    }

    #[test]
    fn get32_aligned() {
        let mut spi = SerialPeripheralInterfaceController::new();
        spi.mode = 123;
        spi.received_data = 789;
        spi.status = 0x45600 | OVERRUN_ERROR_STATUS | RECEIVE_DATA_REGISTER_FULL;
        spi.chip_select0 = 0xABC;

        assert_eq!(spi.get32_aligned(CONTROL_REGISTER), 0);
        assert_eq!(spi.get32_aligned(MODE_REGISTER), 123);
        assert_eq!(spi.get32_aligned(STATUS_REGISTER), 0x45609);
        assert_eq!(spi.get32_aligned(STATUS_REGISTER), 0x45601);
        assert_eq!(spi.get32_aligned(RECEIVE_DATA_REGISTER), 789);
        assert_eq!(spi.get32_aligned(STATUS_REGISTER), 0x45600);
        assert_eq!(spi.get32_aligned(TRANSMIT_DATA_REGISTER), 0);
        assert_eq!(spi.get32_aligned(CHIP_SELECT0_REGISTER), 0xABC);
    }

    #[test]
    #[should_panic(expected = "Unsupported SPI register 0x40008034")]
    fn get32_aligned_unsupported() {
        SerialPeripheralInterfaceController::new().get32_aligned(0x40008034);
    }

    #[test]
    fn set32_aligned_control_spi_enable_clock_enabled() {
        let mut spi = SerialPeripheralInterfaceController::new();
        spi.status = RECEIVE_DATA_REGISTER_FULL;

        spi.set32_aligned(CONTROL_REGISTER, SPI_ENABLE, true, false);

        assert_eq!(
            spi.status,
            RECEIVE_DATA_REGISTER_FULL | SPI_ENABLE_STATUS | TRANSMIT_DATA_REGISTER_EMPTY
        );
    }

    #[test]
    fn set32_aligned_control_spi_enable_clock_disabled() {
        let mut spi = SerialPeripheralInterfaceController::new();
        spi.status = RECEIVE_DATA_REGISTER_FULL;

        spi.set32_aligned(CONTROL_REGISTER, SPI_ENABLE, false, false);

        assert_eq!(spi.status, RECEIVE_DATA_REGISTER_FULL);
    }

    #[test]
    fn set32_aligned_control_spi_disable() {
        let mut spi = SerialPeripheralInterfaceController::new();
        spi.status = SPI_ENABLE_STATUS | RECEIVE_DATA_REGISTER_FULL;

        spi.set32_aligned(CONTROL_REGISTER, SPI_ENABLE | SPI_DISABLE, false, false);

        assert_eq!(spi.status, 0);
    }

    #[test]
    fn set32_aligned_control_software_reset() {
        let mut spi = SerialPeripheralInterfaceController::new();
        spi.mode = 123;
        spi.received_data = 456;
        spi.status = 789;
        spi.chip_select0 = 123456;
        spi.spi_device = Rc::new(RefCell::new(GraphicsCard::default()));

        spi.set32_aligned(CONTROL_REGISTER, SPI_SOFTWARE_RESET, false, false);

        assert_eq!(spi.mode, 0);
        assert_eq!(spi.received_data, 0);
        assert_eq!(spi.status, 0);
        assert_eq!(spi.chip_select0, 0);
    }

    #[test]
    fn set32_aligned_mode() {
        let mut spi = SerialPeripheralInterfaceController::new();

        spi.set32_aligned(MODE_REGISTER, 0xFFF00011, false, false);

        assert_eq!(spi.mode, 0xFF000011);
    }

    #[test]
    #[should_panic(expected = "Unsupported SPI Mode register value 0x00000002")]
    fn set32_aligned_mode_unsupported_ps() {
        let mut spi = SerialPeripheralInterfaceController::new();

        spi.set32_aligned(MODE_REGISTER, PERIPHERAL_SELECT, false, false);
    }

    #[test]
    #[should_panic(expected = "Unsupported SPI Mode register value 0x00000004")]
    fn set32_aligned_mode_unsupported_pcsdec() {
        let mut spi = SerialPeripheralInterfaceController::new();

        spi.set32_aligned(MODE_REGISTER, CHIP_SELECT_DECODE, false, false);
    }

    #[test]
    #[should_panic(expected = "Unsupported SPI Mode register value 0x00010000")]
    fn set32_aligned_mode_unsupported_pcs() {
        let mut spi = SerialPeripheralInterfaceController::new();

        spi.set32_aligned(MODE_REGISTER, 0x10000, false, false);
    }

    #[test]
    fn set32_aligned_receive_data() {
        let mut spi = SerialPeripheralInterfaceController::new();

        spi.set32_aligned(RECEIVE_DATA_REGISTER, 123, true, true);

        assert_eq!(spi.received_data, 0);
    }

    #[test]
    fn set32_aligned_transmit_data_disabled() {
        let mut spi = SerialPeripheralInterfaceController::new();
        spi.mode = MASTER_SLAVE_MODE;
        spi.spi_device = Rc::new(RefCell::new(IncrementSpiDevice::default()));

        spi.set32_aligned(TRANSMIT_DATA_REGISTER, 123, true, true);

        assert_eq!(spi.received_data, 0);
        assert_eq!(spi.status, 0);
    }

    #[test]
    fn set32_aligned_transmit_data_output_disabled() {
        let mut spi = SerialPeripheralInterfaceController::new();
        spi.mode = MASTER_SLAVE_MODE;
        spi.status = SPI_ENABLE_STATUS;
        spi.spi_device = Rc::new(RefCell::new(IncrementSpiDevice::default()));

        spi.set32_aligned(TRANSMIT_DATA_REGISTER, 123, true, false);

        assert_eq!(spi.received_data, 0);
        assert_eq!(spi.status, SPI_ENABLE_STATUS);
    }

    #[test]
    fn set32_aligned_transmit_data_slave_mode() {
        let mut spi = SerialPeripheralInterfaceController::new();
        spi.status = SPI_ENABLE_STATUS;
        spi.spi_device = Rc::new(RefCell::new(IncrementSpiDevice::default()));

        spi.set32_aligned(TRANSMIT_DATA_REGISTER, 123, true, true);

        assert_eq!(spi.received_data, 0);
        assert_eq!(spi.status, SPI_ENABLE_STATUS);
    }

    #[test]
    fn set32_aligned_transmit_data_no_response() {
        let mut spi = SerialPeripheralInterfaceController::new();
        spi.mode = MASTER_SLAVE_MODE;
        spi.status = SPI_ENABLE_STATUS;

        spi.set32_aligned(TRANSMIT_DATA_REGISTER, 123, true, true);

        assert_eq!(spi.received_data, 0);
        assert_eq!(spi.status, SPI_ENABLE_STATUS);
    }

    #[test]
    fn set32_aligned_transmit_data_ok() {
        let mut spi = SerialPeripheralInterfaceController::new();
        spi.mode = MASTER_SLAVE_MODE;
        spi.status = SPI_ENABLE_STATUS;
        spi.spi_device = Rc::new(RefCell::new(IncrementSpiDevice::default()));

        spi.set32_aligned(TRANSMIT_DATA_REGISTER, 123, true, true);

        assert_eq!(spi.received_data, 124);
        assert_eq!(spi.status, SPI_ENABLE_STATUS | RECEIVE_DATA_REGISTER_FULL);
    }

    #[test]
    fn set32_aligned_transmit_data_overrun() {
        let mut spi = SerialPeripheralInterfaceController::new();
        spi.mode = MASTER_SLAVE_MODE;
        spi.received_data = 456;
        spi.status = SPI_ENABLE_STATUS | RECEIVE_DATA_REGISTER_FULL;
        spi.spi_device = Rc::new(RefCell::new(IncrementSpiDevice::default()));

        spi.set32_aligned(TRANSMIT_DATA_REGISTER, 123, true, true);

        assert_eq!(spi.received_data, 124);
        assert_eq!(
            spi.status,
            SPI_ENABLE_STATUS | RECEIVE_DATA_REGISTER_FULL | OVERRUN_ERROR_STATUS
        );
    }

    #[test]
    fn set32_aligned_status() {
        let mut spi = SerialPeripheralInterfaceController::new();

        spi.set32_aligned(STATUS_REGISTER, 123, true, true);

        assert_eq!(spi.status, 0);
    }

    #[test]
    fn set32_aligned_chip_select0() {
        let mut spi = SerialPeripheralInterfaceController::new();

        spi.set32_aligned(CHIP_SELECT0_REGISTER, 123, true, true);

        assert_eq!(spi.chip_select0, 123);
    }

    #[test]
    #[should_panic(expected = "Unsupported SPI register 0x40008034")]
    fn set32_aligned_unsupported() {
        SerialPeripheralInterfaceController::new().set32_aligned(0x40008034, 123, true, true);
    }

    #[test]
    fn reset() {
        let spi_device = Rc::new(RefCell::new(IncrementSpiDevice::default()));
        let mut spi = SerialPeripheralInterfaceController::new();
        spi.mode = 123;
        spi.received_data = 456;
        spi.status = 789;
        spi.chip_select0 = 123456;
        spi.spi_device = spi_device.clone();

        spi.reset();

        assert_eq!(spi.mode, 0);
        assert_eq!(spi.received_data, 0);
        assert_eq!(spi.status, 0);
        assert_eq!(spi.chip_select0, 0);
        assert!(Rc::ptr_eq(
            &spi.spi_device,
            &(spi_device as Rc<RefCell<dyn SpiDevice>>)
        ));
    }
}
