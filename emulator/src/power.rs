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

pub const PM_CONTROLLER_BEGIN: u32 = 0x400E0600;
pub const PM_CONTROLLER_END: u32 = 0x400E0710;
pub const PM_CONTROLLER_LAST: u32 = PM_CONTROLLER_END - 1;

pub const PERIPHERAL_CLOCK_ENABLE_REGISTER_0: u32 = 0x400E0610;
pub const PERIPHERAL_CLOCK_DISABLE_REGISTER_0: u32 = 0x400E0614;
pub const PERIPHERAL_CLOCK_STATUS_REGISTER_0: u32 = 0x400E0618;
pub const MAIN_OSCILLATOR_REGISTER: u32 = 0x400E0620;
pub const PLLA_REGISTER: u32 = 0x400E0628;
pub const MASTER_CLOCK_REGISTER: u32 = 0x400E0630;
pub const STATUS_REGISTER: u32 = 0x400E0668;

// Peripheral Clock Status bits.
const PERIPHERAL_CLOCK_STATUS_BITS: u32 = 0xFFFFFFFC;

// Main Oscillator Register bits.
const MAIN_OSCILLATOR_KEY_MASK: u32 = 0x00FF0000;
const MAIN_OSCILLATOR_KEY: u32 = 0x00370000;
const MAIN_OSCILLATOR_BITS: u32 = 0x0300FF7B;

// PLLA Register used bits.
const PLLA_BITS: u32 = 0x07FF3FFF;

// Master Clock Register used bits.
const MASTER_CLOCK_BITS: u32 = 0x3073;

// Status Register flags.
const MAIN_CRYSTAL_OSCILLATOR_STABILIZED: u32 = 1 << 0;
const PPLA_LOCKED: u32 = 1 << 1;
const MASTER_CLOCK_READY: u32 = 1 << 3;
const MAIN_OSCILLATOR_SELECTION_DONE: u32 = 1 << 16;
const MAIN_ON_CHIP_OSCILLATOR_STABILIZED: u32 = 1 << 17;

/// The Power Managemenet Controller. See section 28, p526 of the Atmel SAM3X Datasheet.
/// This implementation does not support interrupts.
#[derive(Clone)]
pub struct PowerManagementController {
    main_oscillator: u32,
    plla: u32,
    master_clock: u32,
    peripheral_clock_status0: u32,
}

impl PowerManagementController {
    const PERIPHERAL_CLOCK_ENABLE_REGISTER_0_INDEX: u32 =
        (PERIPHERAL_CLOCK_ENABLE_REGISTER_0 - PM_CONTROLLER_BEGIN) / 4;
    const PERIPHERAL_CLOCK_DISABLE_REGISTER_0_INDEX: u32 =
        (PERIPHERAL_CLOCK_DISABLE_REGISTER_0 - PM_CONTROLLER_BEGIN) / 4;
    const PERIPHERAL_CLOCK_STATUS_REGISTER_0_INDEX: u32 =
        (PERIPHERAL_CLOCK_STATUS_REGISTER_0 - PM_CONTROLLER_BEGIN) / 4;
    const MAIN_OSCILLATOR_REGISTER_INDEX: u32 =
        (MAIN_OSCILLATOR_REGISTER - PM_CONTROLLER_BEGIN) / 4;
    const PLLA_REGISTER_INDEX: u32 = (PLLA_REGISTER - PM_CONTROLLER_BEGIN) / 4;
    const MASTER_CLOCK_REGISTER_INDEX: u32 = (MASTER_CLOCK_REGISTER - PM_CONTROLLER_BEGIN) / 4;
    const STATUS_REGISTER_INDEX: u32 = (STATUS_REGISTER - PM_CONTROLLER_BEGIN) / 4;

    pub fn new() -> Self {
        Self {
            main_oscillator: 0x1,
            plla: 0x3F00,
            master_clock: 0x1,
            peripheral_clock_status0: 0x0,
        }
    }

    pub fn get32_aligned(&self, address: u32) -> u32 {
        debug_assert!(address % 4 == 0);
        match (address - PM_CONTROLLER_BEGIN) >> 2 {
            Self::PERIPHERAL_CLOCK_ENABLE_REGISTER_0_INDEX
            | Self::PERIPHERAL_CLOCK_DISABLE_REGISTER_0_INDEX => 0,
            Self::PERIPHERAL_CLOCK_STATUS_REGISTER_0_INDEX => self.peripheral_clock_status0,
            Self::MAIN_OSCILLATOR_REGISTER_INDEX => self.main_oscillator,
            Self::PLLA_REGISTER_INDEX => self.plla,
            Self::MASTER_CLOCK_REGISTER_INDEX => self.master_clock,
            Self::STATUS_REGISTER_INDEX => {
                MAIN_CRYSTAL_OSCILLATOR_STABILIZED
                    | PPLA_LOCKED
                    | MASTER_CLOCK_READY
                    | MAIN_OSCILLATOR_SELECTION_DONE
                    | MAIN_ON_CHIP_OSCILLATOR_STABILIZED
            }
            _ => panic!("Unsupported PMC register {address:#010X}"),
        }
    }

    pub fn set32_aligned(&mut self, address: u32, value: u32) {
        debug_assert!(address % 4 == 0);
        match (address - PM_CONTROLLER_BEGIN) >> 2 {
            Self::PERIPHERAL_CLOCK_ENABLE_REGISTER_0_INDEX => {
                self.peripheral_clock_status0 |= value & PERIPHERAL_CLOCK_STATUS_BITS;
            }
            Self::PERIPHERAL_CLOCK_DISABLE_REGISTER_0_INDEX => {
                self.peripheral_clock_status0 &= !value;
            }
            Self::PERIPHERAL_CLOCK_STATUS_REGISTER_0_INDEX => (),
            Self::MAIN_OSCILLATOR_REGISTER_INDEX => {
                if value & MAIN_OSCILLATOR_KEY_MASK == MAIN_OSCILLATOR_KEY {
                    self.main_oscillator = value & MAIN_OSCILLATOR_BITS;
                }
            }
            Self::PLLA_REGISTER_INDEX => {
                if value & (1 << 29) == 0 {
                    panic!("Bit 29 must be set in PMC PPLA Register value {value:#010X}");
                }
                self.plla = value & PLLA_BITS;
            }
            Self::MASTER_CLOCK_REGISTER_INDEX => self.master_clock = value & MASTER_CLOCK_BITS,
            _ => panic!("Unsupported PMC register {address:#010X}"),
        }
    }

    pub fn spi_clock_enabled(&self) -> bool {
        const SPI_ID: u32 = 24; // See section 9.1 Peripheral Identifiers.
        self.peripheral_clock_status0 & (1 << SPI_ID) != 0
    }

    pub fn usart_clock_enabled(&self) -> bool {
        const USART0_ID: u32 = 17; // See section 9.1 Peripheral Identifiers.
        self.peripheral_clock_status0 & (1 << USART0_ID) != 0
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get32_aligned() {
        let mut power = PowerManagementController::new();
        power.main_oscillator = 123;
        power.plla = 456;
        power.master_clock = 789;
        power.peripheral_clock_status0 = 0xABC;

        assert_eq!(power.get32_aligned(PERIPHERAL_CLOCK_ENABLE_REGISTER_0), 0);
        assert_eq!(power.get32_aligned(PERIPHERAL_CLOCK_DISABLE_REGISTER_0), 0);
        assert_eq!(
            power.get32_aligned(PERIPHERAL_CLOCK_STATUS_REGISTER_0),
            0xABC
        );
        assert_eq!(power.get32_aligned(MAIN_OSCILLATOR_REGISTER), 123);
        assert_eq!(power.get32_aligned(PLLA_REGISTER), 456);
        assert_eq!(power.get32_aligned(MASTER_CLOCK_REGISTER), 789);
        assert_eq!(
            power.get32_aligned(STATUS_REGISTER),
            MAIN_CRYSTAL_OSCILLATOR_STABILIZED
                | PPLA_LOCKED
                | MASTER_CLOCK_READY
                | MAIN_OSCILLATOR_SELECTION_DONE
                | MAIN_ON_CHIP_OSCILLATOR_STABILIZED
        );
    }

    #[test]
    #[should_panic(expected = "Unsupported PMC register 0x400E061C")]
    fn get32_aligned_unsupported() {
        PowerManagementController::new().get32_aligned(0x400E061C);
    }

    #[test]
    fn set32_aligned_peripheral_clock_enable() {
        let mut power = PowerManagementController::new();
        power.peripheral_clock_status0 = 0x80;

        power.set32_aligned(PERIPHERAL_CLOCK_ENABLE_REGISTER_0, 0xABCDEF0F);

        assert_eq!(power.peripheral_clock_status0, 0xABCDEF8C);
    }

    #[test]
    fn set32_aligned_peripheral_clock_disable() {
        let mut power = PowerManagementController::new();
        power.peripheral_clock_status0 = 0xFFFFFFFF;

        power.set32_aligned(PERIPHERAL_CLOCK_DISABLE_REGISTER_0, 0xABCDEFFF);

        assert_eq!(power.peripheral_clock_status0, 0x54321000);
    }

    #[test]
    fn set32_aligned_peripheral_clock_status() {
        let mut power = PowerManagementController::new();
        power.peripheral_clock_status0 = 123;

        power.set32_aligned(PERIPHERAL_CLOCK_STATUS_REGISTER_0, 456);

        assert_eq!(power.peripheral_clock_status0, 123);
    }

    #[test]
    fn set32_aligned_main_oscillator_ok() {
        let mut power = PowerManagementController::new();
        power.main_oscillator = 123;

        power.set32_aligned(MAIN_OSCILLATOR_REGISTER, 0xFF37ABFF);

        assert_eq!(power.main_oscillator, 0x0300AB7B);
    }

    #[test]
    fn set32_aligned_main_oscillator_bad_key() {
        let mut power = PowerManagementController::new();
        power.main_oscillator = 123;

        power.set32_aligned(MAIN_OSCILLATOR_REGISTER, 0xFF38ABFF);

        assert_eq!(power.main_oscillator, 123);
    }

    #[test]
    fn set32_aligned_plla() {
        let mut power = PowerManagementController::new();
        power.plla = 123;

        power.set32_aligned(PLLA_REGISTER, 0xFFAB1234);

        assert_eq!(power.plla, 0x07AB1234);
    }

    #[test]
    #[should_panic(expected = "Bit 29 must be set in PMC PPLA Register value 0x0000007B")]
    fn set32_aligned_plla_unsupported() {
        let mut power = PowerManagementController::new();
        power.set32_aligned(PLLA_REGISTER, 123);
    }

    #[test]
    fn set32_aligned_master_clock() {
        let mut power = PowerManagementController::new();
        power.master_clock = 123;

        power.set32_aligned(MASTER_CLOCK_REGISTER, 0xFFFFFFF1);

        assert_eq!(power.master_clock, 0x3071);
    }

    #[test]
    #[should_panic(expected = "Unsupported PMC register 0x400E061C")]
    fn set32_aligned_unsupported() {
        PowerManagementController::new().set32_aligned(0x400E061C, 123);
    }

    #[test]
    fn spi_clock_enabled() {
        let mut power = PowerManagementController::new();

        power.set32_aligned(PERIPHERAL_CLOCK_ENABLE_REGISTER_0, 1 << 24);

        assert!(power.spi_clock_enabled());
    }

    #[test]
    fn usart_clock_enabled() {
        let mut power = PowerManagementController::new();

        power.set32_aligned(PERIPHERAL_CLOCK_ENABLE_REGISTER_0, 1 << 17);

        assert!(power.usart_clock_enabled());
    }

    #[test]
    fn reset() {
        let mut power = PowerManagementController::new();
        power.main_oscillator = 123;
        power.plla = 456;
        power.master_clock = 789;
        power.peripheral_clock_status0 = 0xABC;

        power.reset();

        assert_eq!(power.main_oscillator, 1);
        assert_eq!(power.plla, 0x3F00);
        assert_eq!(power.master_clock, 1);
        assert_eq!(power.peripheral_clock_status0, 0);
        assert!(!power.spi_clock_enabled());
        assert!(!power.usart_clock_enabled());
    }
}
