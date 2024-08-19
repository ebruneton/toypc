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

pub const NESTED_VECTOR_INTERRUPT_CONTROLLER_BEGIN: u32 = 0xE000E100;
pub const NESTED_VECTOR_INTERRUPT_CONTROLLER_END: u32 = 0xE000E420;
pub const NESTED_VECTOR_INTERRUPT_CONTROLLER_LAST: u32 = NESTED_VECTOR_INTERRUPT_CONTROLLER_END - 1;

pub const SET_ENABLE_REGISTER0: u32 = 0xE000E100;
pub const CLEAR_ENABLE_REGISTER0: u32 = 0xE000E180;
pub const SET_PENDING_REGISTER0: u32 = 0xE000E200;
pub const CLEAR_PENDING_REGISTER0: u32 = 0xE000E280;
pub const ACTIVE_BITS_REGISTER0: u32 = 0xE000E300;

/// The Nested Vector Interrupt Controller. See section 10.20, p152 of the Atmel SAM3X Datasheet.
/// This implementation does not support configurable priorities, and supports interrupts [0..31]
/// only.
#[derive(Clone)]
pub struct NestedVectorInterruptController {
    enabled: u32,
    pending: u32,
    active: u32,
}

impl NestedVectorInterruptController {
    pub fn new() -> Self {
        Self {
            enabled: 0,
            pending: 0,
            active: 0,
        }
    }

    pub fn get32_aligned(&mut self, address: u32) -> u32 {
        debug_assert!(address % 4 == 0);
        match address {
            SET_ENABLE_REGISTER0 | CLEAR_ENABLE_REGISTER0 => self.enabled,
            SET_PENDING_REGISTER0 | CLEAR_PENDING_REGISTER0 => self.pending,
            ACTIVE_BITS_REGISTER0 => self.active,
            _ => panic!("Unsupported NVIC register {address:#010X}"),
        }
    }

    pub fn set32_aligned(&mut self, address: u32, value: u32) {
        debug_assert!(address % 4 == 0);
        match address {
            SET_ENABLE_REGISTER0 => self.enabled |= value,
            CLEAR_ENABLE_REGISTER0 => self.enabled &= !value,
            SET_PENDING_REGISTER0 => self.pending |= value,
            CLEAR_PENDING_REGISTER0 => self.pending &= !value,
            ACTIVE_BITS_REGISTER0 => (),
            _ => panic!("Unsupported NVIC register {address:#010X}"),
        }
    }

    #[inline]
    pub fn maybe_activate_interrupt(&mut self, level_interrupts: u32) -> Option<u8> {
        self.pending |= level_interrupts;
        // Since all interrupts have the same priority (for now), an interrupt can't be activated if
        // there is already one active interrupt. Otherwise, an interrupt must be both enabled and
        // pending to be activable.
        let activable = self.enabled & self.pending;
        if self.active == 0 && activable != 0 {
            let result = activable.trailing_zeros() as u8;
            self.pending &= !(1 << result);
            self.active |= 1 << result;
            Option::Some(result)
        } else {
            Option::None
        }
    }

    #[inline]
    pub fn deactivate_interrupt(&mut self, level_interrupts: u32) {
        debug_assert!(self.active.count_ones() == 1);
        if level_interrupts & self.active == 0 {
            self.pending &= !self.active;
        } else {
            self.pending |= self.active;
        }
        self.active = 0;
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
        let mut nvic = NestedVectorInterruptController::new();
        nvic.enabled = 0x123;
        nvic.pending = 0x456;
        nvic.active = 0x789;

        assert_eq!(nvic.get32_aligned(SET_ENABLE_REGISTER0), 0x123);
        assert_eq!(nvic.get32_aligned(CLEAR_ENABLE_REGISTER0), 0x123);
        assert_eq!(nvic.get32_aligned(SET_PENDING_REGISTER0), 0x456);
        assert_eq!(nvic.get32_aligned(CLEAR_PENDING_REGISTER0), 0x456);
        assert_eq!(nvic.get32_aligned(ACTIVE_BITS_REGISTER0), 0x789);
    }

    #[test]
    #[should_panic(expected = "Unsupported NVIC register 0xE000E104")]
    fn get32_aligned_unsupported() {
        NestedVectorInterruptController::new().get32_aligned(SET_ENABLE_REGISTER0 + 4);
    }

    #[test]
    fn set32_aligned() {
        let mut nvic = NestedVectorInterruptController::new();
        nvic.enabled = 0x123;
        nvic.pending = 0x456;
        nvic.active = 0x789;

        nvic.set32_aligned(SET_ENABLE_REGISTER0, 0xA000);
        nvic.set32_aligned(CLEAR_ENABLE_REGISTER0, 0x1);
        nvic.set32_aligned(SET_PENDING_REGISTER0, 0xB000);
        nvic.set32_aligned(CLEAR_PENDING_REGISTER0, 0x2);
        nvic.set32_aligned(ACTIVE_BITS_REGISTER0, 0);

        assert_eq!(nvic.enabled, 0xA122);
        assert_eq!(nvic.pending, 0xB454);
        assert_eq!(nvic.active, 0x789);
    }

    #[test]
    #[should_panic(expected = "Unsupported NVIC register 0xE000E104")]
    fn set32_aligned_unsupported() {
        NestedVectorInterruptController::new().set32_aligned(SET_ENABLE_REGISTER0 + 4, 0);
    }

    #[test]
    fn maybe_activate_enabled_inactive() {
        let mut nvic = NestedVectorInterruptController::new();
        nvic.enabled = 8;
        nvic.pending = 8;

        let result = nvic.maybe_activate_interrupt(0);

        assert_eq!(result, Option::Some(3));
        assert_eq!(nvic.enabled, 8);
        assert_eq!(nvic.pending, 0);
        assert_eq!(nvic.active, 8);
    }

    #[test]
    fn maybe_activate_enabled_already_active() {
        let mut nvic = NestedVectorInterruptController::new();
        nvic.enabled = 8;
        nvic.pending = 8;
        nvic.active = 1;

        let result = nvic.maybe_activate_interrupt(0);

        assert_eq!(result, Option::None);
        assert_eq!(nvic.enabled, 8);
        assert_eq!(nvic.pending, 8);
        assert_eq!(nvic.active, 1);
    }

    #[test]
    fn maybe_activate_enabled_no_pending() {
        let mut nvic = NestedVectorInterruptController::new();
        nvic.enabled = 8;

        let result = nvic.maybe_activate_interrupt(0);

        assert_eq!(result, Option::None);
        assert_eq!(nvic.enabled, 8);
        assert_eq!(nvic.pending, 0);
        assert_eq!(nvic.active, 0);
    }

    #[test]
    fn maybe_activate_pending_disabled() {
        let mut nvic = NestedVectorInterruptController::new();
        nvic.pending = 8;

        let result = nvic.maybe_activate_interrupt(0);

        assert_eq!(result, Option::None);
        assert_eq!(nvic.enabled, 0);
        assert_eq!(nvic.pending, 8);
        assert_eq!(nvic.active, 0);
    }

    #[test]
    fn maybe_activate_enabled_level_interrupt() {
        let mut nvic = NestedVectorInterruptController::new();
        nvic.enabled = 12;

        let result = nvic.maybe_activate_interrupt(8);

        assert_eq!(result, Option::Some(3));
        assert_eq!(nvic.enabled, 12);
        assert_eq!(nvic.pending, 0);
        assert_eq!(nvic.active, 8);
    }

    #[test]
    fn maybe_activate_disabled_level_interrupt() {
        let mut nvic = NestedVectorInterruptController::new();

        let result = nvic.maybe_activate_interrupt(8);

        assert_eq!(result, Option::None);
        assert_eq!(nvic.enabled, 0);
        assert_eq!(nvic.pending, 8);
        assert_eq!(nvic.active, 0);
    }

    #[test]
    fn deactivate_interrupt_level_high() {
        let mut nvic = NestedVectorInterruptController::new();
        nvic.active = 8;

        nvic.deactivate_interrupt(12);

        assert_eq!(nvic.pending, 8);
        assert_eq!(nvic.active, 0);
    }

    #[test]
    fn deactivate_interrupt_level_low() {
        let mut nvic = NestedVectorInterruptController::new();
        nvic.active = 8;

        nvic.deactivate_interrupt(4);

        assert_eq!(nvic.pending, 0);
        assert_eq!(nvic.active, 0);
    }
}
