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

use std::cmp::Ordering;

pub const SYSTEM_TIMER_BEGIN: u32 = 0xE000E010;
pub const SYSTEM_TIMER_END: u32 = 0xE000E020;
pub const SYSTEM_TIMER_LAST: u32 = SYSTEM_TIMER_END - 1;

pub const CONTROL_AND_STATUS_REGISTER: u32 = 0xE000E010;
pub const RELOAD_VALUE_REGISTER: u32 = 0xE000E014;
pub const CURRENT_VALUE_REGISTER: u32 = 0xE000E018;
pub const CALIBRATION_VALUE_REGISTER: u32 = 0xE000E01C;

// Control and Status Register flags.
const COUNT_FLAG: u32 = 1 << 16;
const CLOCK_SOURCE: u32 = 1 << 2;
const TICK_INTERRUPT: u32 = 1 << 1;
const ENABLE: u32 = 1;

// Calibration Value Register.
const CALIBRATION_VALUE: u32 = 10500;

const INCREMENT: u32 = 1000;

pub type WaitFunction = fn(u32) -> ();

/// The System Timer, SysTick. See section 10.22, p192 of the Atmel SAM3X Datasheet.
#[derive(Clone)]
pub struct SystemTimer {
    control_and_status: u32,
    reload_value: u32,
    current_value: u32,
    wait_function: Option<WaitFunction>,
}

impl Default for SystemTimer {
    fn default() -> Self {
        Self::new(None)
    }
}

impl SystemTimer {
    pub fn new(wait_function: Option<WaitFunction>) -> Self {
        Self {
            control_and_status: 0x4,
            reload_value: 0,
            current_value: 0,
            wait_function,
        }
    }

    pub fn get32_aligned(&mut self, address: u32) -> u32 {
        debug_assert!(address % 4 == 0);
        match address {
            CONTROL_AND_STATUS_REGISTER => {
                let mut result = self.control_and_status;
                if let Some(wait) = self.wait_function {
                    if result & CLOCK_SOURCE != 0 {
                        wait(125 * (self.current_value / CALIBRATION_VALUE));
                    } else {
                        wait(1000 * (self.current_value / CALIBRATION_VALUE));
                    };
                    self.current_value = self.reload_value;
                    result |= COUNT_FLAG;
                }
                self.control_and_status &= !COUNT_FLAG;
                result
            }
            RELOAD_VALUE_REGISTER => self.reload_value,
            CURRENT_VALUE_REGISTER => self.current_value,
            _ => {
                debug_assert!(address == CALIBRATION_VALUE_REGISTER);
                CALIBRATION_VALUE
            }
        }
    }

    pub fn set32_aligned(&mut self, address: u32, value: u32) {
        debug_assert!(address % 4 == 0);
        match address {
            CONTROL_AND_STATUS_REGISTER => {
                if value & TICK_INTERRUPT != 0 {
                    panic!("Unsupported SysTick CTRL register value {value}");
                }
                self.control_and_status &= !(CLOCK_SOURCE | ENABLE);
                self.control_and_status |= value & (CLOCK_SOURCE | ENABLE);
            }
            RELOAD_VALUE_REGISTER => self.reload_value = value,
            CURRENT_VALUE_REGISTER => {
                self.control_and_status &= !COUNT_FLAG;
                self.current_value = 0;
            }
            _ => {
                debug_assert!(address == CALIBRATION_VALUE_REGISTER);
            }
        }
    }

    pub fn update(&mut self) {
        if (self.control_and_status & ENABLE) != 0 {
            let increment = INCREMENT;
            match self.current_value.cmp(&increment) {
                Ordering::Greater => self.current_value -= increment,
                Ordering::Equal => {
                    self.control_and_status |= COUNT_FLAG;
                    self.current_value = 0;
                }
                Ordering::Less => {
                    if self.current_value != 0 {
                        self.control_and_status |= COUNT_FLAG;
                    }
                    if self.reload_value == 0 {
                        self.current_value = 0;
                    } else {
                        self.current_value += self.reload_value - increment;
                    }
                }
            }
        }
    }

    pub fn reset(&mut self) {
        self.control_and_status = 0x4;
        self.reload_value = 0;
        self.current_value = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get32_aligned() {
        let mut timer = SystemTimer {
            control_and_status: COUNT_FLAG | CLOCK_SOURCE | ENABLE,
            reload_value: 123,
            current_value: 456,
            ..Default::default()
        };

        assert_eq!(
            timer.get32_aligned(CONTROL_AND_STATUS_REGISTER),
            COUNT_FLAG | CLOCK_SOURCE | ENABLE
        );
        assert_eq!(
            timer.get32_aligned(CONTROL_AND_STATUS_REGISTER),
            CLOCK_SOURCE | ENABLE
        );
        assert_eq!(timer.get32_aligned(RELOAD_VALUE_REGISTER), 123);
        assert_eq!(timer.get32_aligned(CURRENT_VALUE_REGISTER), 456);
        assert_eq!(
            timer.get32_aligned(CALIBRATION_VALUE_REGISTER),
            CALIBRATION_VALUE
        );
    }

    #[test]
    fn set32_aligned_control() {
        let mut timer = SystemTimer {
            control_and_status: COUNT_FLAG | CLOCK_SOURCE,
            ..Default::default()
        };

        timer.set32_aligned(CONTROL_AND_STATUS_REGISTER, 0xFFFFFF00 | ENABLE);

        assert_eq!(timer.control_and_status, COUNT_FLAG | ENABLE);
    }

    #[test]
    #[should_panic(expected = "Unsupported SysTick CTRL register value 2")]
    fn set32_aligned_control_tick_interrupt() {
        let mut timer = SystemTimer {
            control_and_status: COUNT_FLAG | CLOCK_SOURCE,
            ..Default::default()
        };

        timer.set32_aligned(CONTROL_AND_STATUS_REGISTER, TICK_INTERRUPT);
    }

    #[test]
    fn set32_aligned_reload_value() {
        let mut timer = SystemTimer {
            reload_value: 123,
            ..Default::default()
        };

        timer.set32_aligned(RELOAD_VALUE_REGISTER, 456);

        assert_eq!(timer.reload_value, 456);
    }

    #[test]
    fn set32_aligned_current_value() {
        let mut timer = SystemTimer {
            control_and_status: COUNT_FLAG | CLOCK_SOURCE | ENABLE,
            current_value: 123,
            ..Default::default()
        };

        timer.set32_aligned(CURRENT_VALUE_REGISTER, 456);

        assert_eq!(timer.control_and_status, CLOCK_SOURCE | ENABLE);
        assert_eq!(timer.current_value, 0);
    }

    #[test]
    fn set32_aligned_calibration_value() {
        let mut timer = SystemTimer::default();

        timer.set32_aligned(CALIBRATION_VALUE_REGISTER, 456);

        assert_eq!(
            timer.get32_aligned(CALIBRATION_VALUE_REGISTER),
            CALIBRATION_VALUE
        );
    }

    #[test]
    fn update() {
        let mut timer = SystemTimer::default();
        timer.set32_aligned(CONTROL_AND_STATUS_REGISTER, ENABLE);
        timer.set32_aligned(RELOAD_VALUE_REGISTER, 5 * INCREMENT);
        timer.current_value = 4 * INCREMENT;

        timer.update();

        assert_eq!(timer.current_value, 3 * INCREMENT);
        assert_eq!(timer.control_and_status & COUNT_FLAG, 0);
    }

    #[test]
    fn update_from_increment() {
        let mut timer = SystemTimer::default();
        timer.set32_aligned(RELOAD_VALUE_REGISTER, 5 * INCREMENT);
        timer.set32_aligned(CONTROL_AND_STATUS_REGISTER, CLOCK_SOURCE | ENABLE);
        timer.current_value = INCREMENT;

        timer.update();

        assert_eq!(timer.current_value, 0);
        assert_eq!(timer.control_and_status & COUNT_FLAG, COUNT_FLAG);
    }

    #[test]
    fn update_from_one() {
        let mut timer = SystemTimer::default();
        timer.set32_aligned(RELOAD_VALUE_REGISTER, 5 * INCREMENT);
        timer.set32_aligned(CONTROL_AND_STATUS_REGISTER, CLOCK_SOURCE | ENABLE);
        timer.current_value = 1;

        timer.update();

        assert_eq!(timer.current_value, 4 * INCREMENT + 1);
        assert_eq!(timer.control_and_status & COUNT_FLAG, COUNT_FLAG);
    }

    #[test]
    fn update_from_zero() {
        let mut timer = SystemTimer::default();
        timer.set32_aligned(RELOAD_VALUE_REGISTER, 5 * INCREMENT);
        timer.set32_aligned(CONTROL_AND_STATUS_REGISTER, CLOCK_SOURCE | ENABLE);
        timer.current_value = 0;

        timer.update();

        assert_eq!(timer.current_value, 4 * INCREMENT);
        assert_eq!(timer.control_and_status & COUNT_FLAG, 0);
    }

    #[test]
    fn update_disabled() {
        let mut timer = SystemTimer {
            current_value: 4 * INCREMENT,
            ..Default::default()
        };

        timer.update();

        assert_eq!(timer.current_value, 4 * INCREMENT);
        assert_eq!(timer.control_and_status & COUNT_FLAG, 0);
    }

    #[test]
    fn reset() {
        let mut timer = SystemTimer {
            control_and_status: COUNT_FLAG | ENABLE,
            reload_value: 123,
            current_value: 456,
            ..Default::default()
        };

        timer.reset();

        assert_eq!(timer.control_and_status, CLOCK_SOURCE);
        assert_eq!(timer.reload_value, 0);
        assert_eq!(timer.current_value, 0);
    }
}
