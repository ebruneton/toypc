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

pub const WATCHDOG_TIMER_BEGIN: u32 = 0x400E1A50;
pub const WATCHDOG_TIMER_END: u32 = 0x400E1A5C;
pub const WATCHDOG_TIMER_LAST: u32 = WATCHDOG_TIMER_END - 1;

pub const MODE_REGISTER: u32 = 0x400E1A54;
const INITIAL_MODE: u32 = 0x3FFF2FFF;

/// The Watchdog Timer. See section 15, p260 of the Atmel SAM3X Datasheet.
/// This dummy implementation never decrements its counter, and thus never triggers a reset.
#[derive(Clone)]
pub struct WatchdogTimer {
    mode: u32,
    written_once: bool,
}

impl WatchdogTimer {
    pub fn uninitialized() -> Self {
        Self {
            mode: 0,
            written_once: false,
        }
    }

    #[cfg(test)]
    fn new() -> Self {
        let mut result = Self::uninitialized();
        result.reset();
        result
    }

    pub fn get32_aligned(&self, address: u32) -> u32 {
        if address == MODE_REGISTER {
            self.mode
        } else {
            0
        }
    }

    pub fn set32_aligned(&mut self, address: u32, value: u32) {
        if address == MODE_REGISTER && !self.written_once {
            self.mode = value;
            self.written_once = true;
        }
    }

    pub fn reset(&mut self) {
        self.mode = INITIAL_MODE;
        self.written_once = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CONTROL_REGISTER: u32 = 0x400E1A50;
    const STATUS_REGISTER: u32 = 0x400E1A58;

    #[test]
    fn get32_aligned() {
        let watchdog = WatchdogTimer::new();

        assert_eq!(watchdog.get32_aligned(CONTROL_REGISTER), 0);
        assert_eq!(watchdog.get32_aligned(MODE_REGISTER), INITIAL_MODE);
        assert_eq!(watchdog.get32_aligned(STATUS_REGISTER), 0);
    }

    #[test]
    fn set32_aligned() {
        let mut watchdog = WatchdogTimer::new();

        watchdog.set32_aligned(CONTROL_REGISTER, 123);
        watchdog.set32_aligned(MODE_REGISTER, 456);
        watchdog.set32_aligned(MODE_REGISTER, 0);
        watchdog.set32_aligned(STATUS_REGISTER, 789);

        assert_eq!(watchdog.get32_aligned(CONTROL_REGISTER), 0);
        assert_eq!(watchdog.get32_aligned(MODE_REGISTER), 456);
        assert_eq!(watchdog.get32_aligned(STATUS_REGISTER), 0);
    }

    #[test]
    fn reset() {
        let mut watchdog = WatchdogTimer::new();

        watchdog.set32_aligned(MODE_REGISTER, 456);
        watchdog.reset();

        assert_eq!(watchdog.get32_aligned(MODE_REGISTER), INITIAL_MODE);
    }

    #[test]
    fn write_after_reset() {
        let mut watchdog = WatchdogTimer::new();

        watchdog.set32_aligned(MODE_REGISTER, 456);
        watchdog.reset();
        watchdog.set32_aligned(MODE_REGISTER, 789);

        assert_eq!(watchdog.get32_aligned(MODE_REGISTER), 789);
    }
}
