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

pub const RESET_CONTROLLER_BEGIN: u32 = 0x400E1A00;
pub const RESET_CONTROLLER_END: u32 = 0x400E1A0C;
pub const RESET_CONTROLLER_LAST: u32 = RESET_CONTROLLER_END - 1;

pub const CONTROL_REGISTER: u32 = 0x400E1A00;
const KEY_MASK: u32 = 0xFF000000;
const KEY: u32 = 0xA5000000;
const RESET_BITS: u32 = 0b1101;

/// The Reset Controller. See section 12, p222 of the Atmel SAM3X Datasheet.
/// This implementation only supports the Control Register.
#[derive(Clone)]
pub struct ResetController {
    reset_requested: bool,
}

impl ResetController {
    pub fn new() -> Self {
        Self {
            reset_requested: false,
        }
    }

    pub fn get32_aligned(&self, address: u32) -> u32 {
        debug_assert!(address % 4 == 0);
        if address == CONTROL_REGISTER {
            0
        } else {
            panic!("Unsupported Reset Controller Register {address:#010X}");
        }
    }

    pub fn set32_aligned(&mut self, address: u32, value: u32) {
        debug_assert!(address % 4 == 0);
        if address == CONTROL_REGISTER {
            if value & KEY_MASK == KEY {
                if value & RESET_BITS == RESET_BITS {
                    self.reset_requested = true;
                } else if value & RESET_BITS != 0 {
                    panic!("Unsupported Reset Controller Control Register value {value:#010X}");
                }
            }
        } else {
            panic!("Unsupported Reset Controller Register {address:#010X}");
        }
    }

    #[inline]
    pub fn reset_requested(&self) -> bool {
        self.reset_requested
    }

    pub fn reset(&mut self) {
        self.reset_requested = false;
    }
}

#[cfg(test)]
mod tests {
    use super::{ResetController, CONTROL_REGISTER};

    const STATUS_REGISTER: u32 = 0x400E1A04;
    const MODE_REGISTER: u32 = 0x400E1A08;

    #[test]
    fn get32_aligned_control() {
        let reset_controller = ResetController::new();

        assert_eq!(reset_controller.get32_aligned(CONTROL_REGISTER), 0);
        assert!(!reset_controller.reset_requested());
    }

    #[test]
    #[should_panic(expected = "Unsupported Reset Controller Register 0x400E1A04")]
    fn get32_aligned_status() {
        ResetController::new().get32_aligned(STATUS_REGISTER);
    }

    #[test]
    #[should_panic(expected = "Unsupported Reset Controller Register 0x400E1A08")]
    fn get32_aligned_mode() {
        ResetController::new().get32_aligned(MODE_REGISTER);
    }

    #[test]
    fn set32_aligned_control() {
        let mut reset_controller = ResetController::new();

        reset_controller.set32_aligned(CONTROL_REGISTER, 0xA500000D);

        assert!(reset_controller.reset_requested());
    }

    #[test]
    fn set32_aligned_control_bad_key() {
        let mut reset_controller = ResetController::new();

        reset_controller.set32_aligned(CONTROL_REGISTER, 0xA600000D);

        assert!(!reset_controller.reset_requested());
    }

    #[test]
    fn set32_aligned_control_no_reset_bits() {
        let mut reset_controller = ResetController::new();

        reset_controller.set32_aligned(CONTROL_REGISTER, 0xA5000000);

        assert!(!reset_controller.reset_requested());
    }

    #[test]
    #[should_panic(expected = "Unsupported Reset Controller Control Register value 0xA500000E")]
    fn set32_aligned_control_bad_value() {
        let mut reset_controller = ResetController::new();

        reset_controller.set32_aligned(CONTROL_REGISTER, 0xA500000E);
    }

    #[test]
    #[should_panic(expected = "Unsupported Reset Controller Register 0x400E1A04")]
    fn set32_aligned_status() {
        ResetController::new().set32_aligned(STATUS_REGISTER, 123);
    }

    #[test]
    #[should_panic(expected = "Unsupported Reset Controller Register 0x400E1A08")]
    fn set32_aligned_mode() {
        ResetController::new().set32_aligned(MODE_REGISTER, 123);
    }

    #[test]
    fn reset() {
        let mut reset_controller = ResetController::new();

        reset_controller.set32_aligned(CONTROL_REGISTER, 0xA500000D);
        reset_controller.reset();

        assert!(!reset_controller.reset_requested());
    }
}
