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
    bus::{FLASH_BEGIN, FLASH_END},
    memory::MemoryBank,
};

pub const FLASH_CONTROLLER_BEGIN: u32 = 0x400E0A00;
pub const FLASH_CONTROLLER_END: u32 = 0x400E0E00;
pub const FLASH_CONTROLLER_LAST: u32 = FLASH_CONTROLLER_END - 1;

pub const MODE_REGISTER0: u32 = 0x400E0A00;
pub const COMMAND_REGISTER0: u32 = 0x400E0A04;
pub const STATUS_REGISTER0: u32 = 0x400E0A08;
pub const RESULT_REGISTER0: u32 = 0x400E0A0C;
pub const MODE_REGISTER1: u32 = 0x400E0C00;
pub const COMMAND_REGISTER1: u32 = 0x400E0C04;
pub const STATUS_REGISTER1: u32 = 0x400E0C08;

// Mode Register flags.
const READY_INTERRUPT_ENABLE: u32 = 0x1;
const MODE_BITS: u32 = 0x1000F01;

// Control Register flags.
const FLASH_PROTECTION_KEY: u32 = 0x5A000000;
const ERASE_AND_WRITE_PAGE_COMMAND: u32 = 0x3;
const SET_GPNVM_COMMAND: u32 = 0xB;
const CLEAR_GPNVM_COMMAND: u32 = 0xC;
const GET_GPNVM_COMMAND: u32 = 0xD;

// Status Register flags.
const READY_STATUS: u32 = 0x1;
const ERROR_STATUS: u32 = 0x2;

const FLASH_PAGE_BYTES: u32 = 256;
const FLASH_PAGE_WORDS: u32 = FLASH_PAGE_BYTES / 4;
const FLASH_PAGES_PER_CONTROLLER: u32 = (FLASH_END - FLASH_BEGIN) / FLASH_PAGE_BYTES / 2;

/// The Enhanced Embedded Flash Controller. See section 18, p292 of the Atmel SAM3X Datasheet.
/// This implementation does not support interrupts.
#[derive(Clone)]
pub struct FlashController {
    mode_register0: u32,
    status_register0: u32,
    result_register0: u32,
    mode_register1: u32,
    status_register1: u32,
    boot_from_flash: bool,
    latch_buffer: [Option<u32>; FLASH_PAGE_WORDS as usize],
}

impl FlashController {
    const MODE_REGISTER0_INDEX: u32 = (MODE_REGISTER0 - FLASH_CONTROLLER_BEGIN) >> 2;
    const COMMAND_REGISTER0_INDEX: u32 = (COMMAND_REGISTER0 - FLASH_CONTROLLER_BEGIN) >> 2;
    const STATUS_REGISTER0_INDEX: u32 = (STATUS_REGISTER0 - FLASH_CONTROLLER_BEGIN) >> 2;
    const RESULT_REGISTER0_INDEX: u32 = (RESULT_REGISTER0 - FLASH_CONTROLLER_BEGIN) >> 2;
    const MODE_REGISTER1_INDEX: u32 = (MODE_REGISTER1 - FLASH_CONTROLLER_BEGIN) >> 2;
    const COMMAND_REGISTER1_INDEX: u32 = (COMMAND_REGISTER1 - FLASH_CONTROLLER_BEGIN) >> 2;
    const STATUS_REGISTER1_INDEX: u32 = (STATUS_REGISTER1 - FLASH_CONTROLLER_BEGIN) >> 2;

    pub fn uninitialized() -> Self {
        Self {
            mode_register0: 0,
            status_register0: 0,
            result_register0: 0,
            mode_register1: 0,
            status_register1: 0,
            boot_from_flash: false,
            latch_buffer: [Option::None; FLASH_PAGE_WORDS as usize],
        }
    }

    #[cfg(test)]
    pub fn new() -> Self {
        let mut result = Self::uninitialized();
        result.erase();
        result.reset();
        result
    }

    pub fn get32_aligned(&mut self, address: u32) -> u32 {
        debug_assert!(address % 4 == 0);
        match (address - FLASH_CONTROLLER_BEGIN) >> 2 {
            Self::MODE_REGISTER0_INDEX => self.mode_register0,
            Self::COMMAND_REGISTER0_INDEX => 0,
            Self::STATUS_REGISTER0_INDEX => {
                let result = self.status_register0;
                self.status_register0 = READY_STATUS;
                result
            }
            Self::RESULT_REGISTER0_INDEX => {
                let result = self.result_register0;
                self.result_register0 = 0;
                result
            }
            Self::MODE_REGISTER1_INDEX => self.mode_register1,
            Self::COMMAND_REGISTER1_INDEX => 0,
            Self::STATUS_REGISTER1_INDEX => {
                let result = self.status_register1;
                self.status_register1 = READY_STATUS;
                result
            }
            _ => panic!("Unsupported EEFC register {address:#010X}"),
        }
    }

    pub fn set32_aligned(&mut self, address: u32, value: u32, flash: &mut MemoryBank) {
        debug_assert!(address % 4 == 0);
        match (address - FLASH_CONTROLLER_BEGIN) >> 2 {
            Self::MODE_REGISTER0_INDEX => {
                if value & READY_INTERRUPT_ENABLE == 0 {
                    self.mode_register0 = value & MODE_BITS;
                } else {
                    panic!("Unsupported EEFC Mode register value {value:#010X}");
                }
            }
            Self::COMMAND_REGISTER0_INDEX => {
                if Self::flash_writing_protection_key(value) != FLASH_PROTECTION_KEY {
                    self.status_register0 = ERROR_STATUS | READY_STATUS;
                    return;
                }
                let argument = Self::flash_command_argument(value);
                match Self::flash_command(value) {
                    ERASE_AND_WRITE_PAGE_COMMAND => {
                        if argument < FLASH_PAGES_PER_CONTROLLER {
                            let base_index = argument * FLASH_PAGE_WORDS;
                            self.erase_and_flash(base_index, flash);
                        }
                        self.status_register0 = READY_STATUS;
                    }
                    SET_GPNVM_COMMAND => {
                        if argument == 1 {
                            self.boot_from_flash = true;
                            self.status_register0 = READY_STATUS;
                        } else {
                            panic!("Unsupported GPNVM bit {argument}");
                        }
                    }
                    CLEAR_GPNVM_COMMAND => {
                        if argument == 1 {
                            self.boot_from_flash = false;
                            self.status_register0 = READY_STATUS;
                        } else {
                            panic!("Unsupported GPNVM bit {argument}");
                        }
                    }
                    GET_GPNVM_COMMAND => {
                        self.result_register0 = (self.boot_from_flash as u32) << 1;
                        self.status_register0 = READY_STATUS;
                    }
                    _ => panic!("Unsupported EEFC Command register value {value:#010X}"),
                }
            }
            Self::STATUS_REGISTER0_INDEX => (),
            Self::RESULT_REGISTER0_INDEX => (),
            Self::MODE_REGISTER1_INDEX => {
                if value & READY_INTERRUPT_ENABLE == 0 {
                    self.mode_register1 = value & MODE_BITS;
                } else {
                    panic!("Unsupported EEFC Mode register value {value:#010X}");
                }
            }
            Self::COMMAND_REGISTER1_INDEX => {
                if Self::flash_writing_protection_key(value) != FLASH_PROTECTION_KEY {
                    self.status_register1 = ERROR_STATUS | READY_STATUS;
                    return;
                }
                match Self::flash_command(value) {
                    ERASE_AND_WRITE_PAGE_COMMAND => {
                        let page = Self::flash_command_argument(value);
                        if page < FLASH_PAGES_PER_CONTROLLER {
                            let base_index = (page + FLASH_PAGES_PER_CONTROLLER) * FLASH_PAGE_WORDS;
                            self.erase_and_flash(base_index, flash);
                        }
                        self.status_register1 = READY_STATUS;
                    }
                    SET_GPNVM_COMMAND => (),
                    CLEAR_GPNVM_COMMAND => (),
                    GET_GPNVM_COMMAND => (),
                    _ => panic!("Unsupported EEFC Command register value {value:#010X}"),
                }
            }
            Self::STATUS_REGISTER1_INDEX => (),
            _ => panic!("Unsupported EEFC register {address:#010X}"),
        }
    }

    fn flash_writing_protection_key(command: u32) -> u32 {
        const FLASH_WRITING_PROTECTION_KEY_MASK: u32 = 0xFF000000;
        command & FLASH_WRITING_PROTECTION_KEY_MASK
    }

    fn flash_command(command: u32) -> u32 {
        const FLASH_COMMAND_MASK: u32 = 0xFF;
        command & FLASH_COMMAND_MASK
    }

    fn flash_command_argument(command: u32) -> u32 {
        const FLASH_COMMAND_ARGUMENT_MASK: u32 = 0x00FFFF00;
        (command & FLASH_COMMAND_ARGUMENT_MASK) >> 8
    }

    fn erase_and_flash(&mut self, base_index: u32, flash: &mut MemoryBank) {
        let mut word_count = 0;
        for word in self.latch_buffer {
            if word.is_some() {
                word_count += 1;
            }
        }
        if word_count == self.latch_buffer.len() {
            for (index, word) in self.latch_buffer.iter().enumerate() {
                flash.set32_aligned((base_index + index as u32) << 2, word.unwrap());
            }
        } else {
            for index in 0..self.latch_buffer.len() {
                flash.set32_aligned((base_index + index as u32) << 2, 0xFFFFFFFF);
            }
        }
        self.latch_buffer.fill(Option::None);
    }

    pub fn boot_from_flash(&self) -> bool {
        self.boot_from_flash
    }

    pub fn set_latch_buffer(&mut self, address: u32, value: u32) {
        self.latch_buffer[((address >> 2) % FLASH_PAGE_WORDS) as usize] = Option::Some(value);
    }

    pub fn erase(&mut self) {
        self.boot_from_flash = false;
    }

    pub fn reset(&mut self) {
        self.mode_register0 = if self.boot_from_flash { 0 } else { 0x200 };
        self.status_register0 = READY_STATUS;
        self.result_register0 = 0;
        self.mode_register1 = if self.boot_from_flash { 0 } else { 0x200 };
        self.status_register1 = READY_STATUS;
        self.latch_buffer.fill(Option::None);
    }

    pub fn serialize(&self, output: &mut Vec<u32>) {
        output.push(self.boot_from_flash as u32);
    }

    pub fn deserialize(&mut self, input: &mut Vec<u32>) {
        self.boot_from_flash = input.pop().unwrap() == (true as u32);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get32_aligned() {
        let mut flash = FlashController::new();
        flash.mode_register0 = 123;
        flash.status_register0 = 456;
        flash.result_register0 = 789;
        flash.mode_register1 = 0xABC;
        flash.status_register1 = 0xDEF;

        assert_eq!(flash.get32_aligned(MODE_REGISTER0), 123);
        assert_eq!(flash.get32_aligned(COMMAND_REGISTER0), 0);
        assert_eq!(flash.get32_aligned(STATUS_REGISTER0), 456);
        assert_eq!(flash.get32_aligned(STATUS_REGISTER0), READY_STATUS);
        assert_eq!(flash.get32_aligned(RESULT_REGISTER0), 789);
        assert_eq!(flash.get32_aligned(RESULT_REGISTER0), 0);
        assert_eq!(flash.get32_aligned(MODE_REGISTER1), 0xABC);
        assert_eq!(flash.get32_aligned(COMMAND_REGISTER1), 0);
        assert_eq!(flash.get32_aligned(STATUS_REGISTER1), 0xDEF);
        assert_eq!(flash.get32_aligned(STATUS_REGISTER1), READY_STATUS);
    }

    #[test]
    #[should_panic(expected = "Unsupported EEFC register 0x400E0C0C")]
    fn get32_aligned_unsupported() {
        FlashController::new().get32_aligned(0x400E0C0C);
    }

    #[test]
    fn set32_aligned_mode() {
        let mut flash = FlashController::new();
        let mut memory = MemoryBank::new(0, 256);

        flash.set32_aligned(MODE_REGISTER0, 0xF100, &mut memory);
        flash.set32_aligned(MODE_REGISTER1, 0xF100, &mut memory);

        assert_eq!(flash.mode_register0, 0x0100);
        assert_eq!(flash.mode_register1, 0x0100);
    }

    #[test]
    #[should_panic(expected = "Unsupported EEFC Mode register value 0x00000001")]
    fn set32_aligned_mode0_unsupported() {
        let mut flash = FlashController::new();
        let mut memory = MemoryBank::new(0, 256);
        flash.set32_aligned(MODE_REGISTER0, READY_INTERRUPT_ENABLE, &mut memory);
    }

    #[test]
    #[should_panic(expected = "Unsupported EEFC Mode register value 0x00000001")]
    fn set32_aligned_mode1_unsupported() {
        let mut flash = FlashController::new();
        let mut memory = MemoryBank::new(0, 256);
        flash.set32_aligned(MODE_REGISTER1, READY_INTERRUPT_ENABLE, &mut memory);
    }

    #[test]
    fn set32_aligned_control0() {
        let mut flash = FlashController::new();
        let mut memory = MemoryBank::new(0, 256);

        flash.set32_aligned(MODE_REGISTER0, 0xF100, &mut memory);
        flash.set32_aligned(MODE_REGISTER1, 0xF100, &mut memory);

        assert_eq!(flash.mode_register0, 0x0100);
        assert_eq!(flash.mode_register1, 0x0100);
    }

    #[test]
    fn set32_aligned_control0_write_page_ok() {
        let mut flash = FlashController::new();
        flash.status_register0 = 123;
        for i in 0..64 {
            flash.set_latch_buffer(4 * i + 2560, i);
        }
        let mut memory = MemoryBank::new(0, 256);

        flash.set32_aligned(COMMAND_REGISTER0, 0x5A000103, &mut memory);

        assert_eq!(flash.status_register0, READY_STATUS);
        for i in 0..64 {
            assert_eq!(memory.get32_aligned(4 * i), 0);
            assert_eq!(memory.get32_aligned(4 * i + 256), i);
        }
    }

    #[test]
    fn set32_aligned_control0_write_page_bad_key() {
        let mut flash = FlashController::new();
        for i in 0..64 {
            flash.set_latch_buffer(4 * i + 2560, i);
        }
        let mut memory = MemoryBank::new(0, 256);

        flash.set32_aligned(COMMAND_REGISTER0, 0x5B000103, &mut memory);

        for i in 0..64 {
            assert_eq!(memory.get32_aligned(4 * i + 256), 0);
        }
    }

    #[test]
    fn set32_aligned_control0_write_page_bad_page() {
        let mut flash = FlashController::new();
        for i in 0..64 {
            flash.set_latch_buffer(4 * i + 2560, i);
        }
        let mut memory = MemoryBank::new(0, 256);

        flash.set32_aligned(COMMAND_REGISTER0, 0x5A040103, &mut memory);

        for i in 0..64 {
            assert_eq!(memory.get32_aligned(4 * i + 256), 0);
        }
    }

    #[test]
    fn set32_aligned_control0_write_page_missing_values() {
        let mut flash = FlashController::new();
        flash.status_register0 = 123;
        for i in 0..32 {
            flash.set_latch_buffer(4 * i + 2560, i);
        }
        let mut memory = MemoryBank::new(0, 256);

        flash.set32_aligned(COMMAND_REGISTER0, 0x5A000103, &mut memory);

        assert_eq!(flash.status_register0, READY_STATUS);
        for i in 0..64 {
            assert_eq!(memory.get32_aligned(4 * i + 256), 0xFFFFFFFF);
        }
    }

    #[test]
    fn set32_aligned_control0_write_two_pages_missing_values() {
        let mut flash = FlashController::new();
        flash.status_register0 = 123;
        for i in 0..64 {
            flash.set_latch_buffer(4 * i + 2560, i);
        }
        let mut memory = MemoryBank::new(0, 256);

        flash.set32_aligned(COMMAND_REGISTER0, 0x5A000103, &mut memory);
        flash.set32_aligned(COMMAND_REGISTER0, 0x5A000203, &mut memory);

        assert_eq!(flash.status_register0, READY_STATUS);
        for i in 0..64 {
            assert_eq!(memory.get32_aligned(4 * i + 256), i);
            assert_eq!(memory.get32_aligned(4 * i + 512), 0xFFFFFFFF);
        }
    }

    #[test]
    fn set32_aligned_control0_set_bit_ok() {
        let mut flash = FlashController::new();
        flash.status_register0 = 123;
        let mut memory = MemoryBank::new(0, 256);

        flash.set32_aligned(COMMAND_REGISTER0, 0x5A00010B, &mut memory);

        assert_eq!(flash.status_register0, READY_STATUS);
        assert!(flash.boot_from_flash());
    }

    #[test]
    #[should_panic(expected = "Unsupported GPNVM bit 2")]
    fn set32_aligned_control0_set_bit_unsupported() {
        let mut flash = FlashController::new();
        let mut memory = MemoryBank::new(0, 256);

        flash.set32_aligned(COMMAND_REGISTER0, 0x5A00020B, &mut memory);
    }

    #[test]
    fn set32_aligned_control0_clear_bit_ok() {
        let mut flash = FlashController::new();
        flash.status_register0 = 123;
        flash.boot_from_flash = true;
        let mut memory = MemoryBank::new(0, 256);

        flash.set32_aligned(COMMAND_REGISTER0, 0x5A00010C, &mut memory);

        assert_eq!(flash.status_register0, READY_STATUS);
        assert!(!flash.boot_from_flash());
    }

    #[test]
    #[should_panic(expected = "Unsupported GPNVM bit 2")]
    fn set32_aligned_control0_clear_bit_unsupported() {
        let mut flash = FlashController::new();
        let mut memory = MemoryBank::new(0, 256);

        flash.set32_aligned(COMMAND_REGISTER0, 0x5A00020C, &mut memory);
    }

    #[test]
    fn set32_aligned_control0_get_bit() {
        let mut flash = FlashController::new();
        flash.status_register0 = 123;
        flash.boot_from_flash = true;
        let mut memory = MemoryBank::new(0, 256);

        flash.set32_aligned(COMMAND_REGISTER0, 0x5A00010D, &mut memory);

        assert_eq!(flash.status_register0, READY_STATUS);
        assert_eq!(flash.get32_aligned(RESULT_REGISTER0), 2);
        assert!(flash.boot_from_flash());
    }

    #[test]
    #[should_panic(expected = "Unsupported EEFC Command register value 0x5A000004")]
    fn set32_aligned_control0_unsupported() {
        let mut flash = FlashController::new();
        let mut memory = MemoryBank::new(0, 256);

        flash.set32_aligned(COMMAND_REGISTER0, 0x5A000004, &mut memory);
    }

    #[test]
    fn set32_aligned_control1_write_page_ok() {
        let mut flash = FlashController::new();
        for i in 0..64 {
            flash.set_latch_buffer(4 * i + 256, i);
        }
        let mut memory = MemoryBank::new(0, 512 * 1024);

        flash.set32_aligned(COMMAND_REGISTER1, 0x5A000103, &mut memory);

        for i in 0..64 {
            assert_eq!(memory.get32_aligned(4 * i), 0);
            assert_eq!(memory.get32_aligned(4 * i + 256 * 1025), i);
        }
    }

    #[test]
    fn set32_aligned_control1_write_page_bad_key() {
        let mut flash = FlashController::new();
        for i in 0..64 {
            flash.set_latch_buffer(4 * i + 2560, i);
        }
        let mut memory = MemoryBank::new(0, 512 * 1024);

        flash.set32_aligned(COMMAND_REGISTER1, 0x5B000103, &mut memory);

        for i in 0..64 {
            assert_eq!(memory.get32_aligned(4 * i + 256 * 1025), 0);
        }
    }

    #[test]
    fn set32_aligned_control1_write_page_bad_page() {
        let mut flash = FlashController::new();
        for i in 0..64 {
            flash.set_latch_buffer(4 * i + 2560, i);
        }
        let mut memory = MemoryBank::new(0, 512 * 1024);

        flash.set32_aligned(COMMAND_REGISTER1, 0x5A040103, &mut memory);

        for i in 0..64 {
            assert_eq!(memory.get32_aligned(4 * i + 256 * 1025), 0);
        }
    }

    #[test]
    fn set32_aligned_control1_set_bit_ok() {
        let mut flash = FlashController::new();
        flash.status_register0 = 123;
        let mut memory = MemoryBank::new(0, 256);

        flash.set32_aligned(COMMAND_REGISTER1, 0x5A00010B, &mut memory);

        assert_eq!(flash.status_register0, 123);
        assert!(!flash.boot_from_flash());
    }

    #[test]
    fn set32_aligned_control1_clear_bit_ok() {
        let mut flash = FlashController::new();
        flash.status_register0 = 123;
        flash.boot_from_flash = true;
        let mut memory = MemoryBank::new(0, 256);

        flash.set32_aligned(COMMAND_REGISTER1, 0x5A00010C, &mut memory);

        assert_eq!(flash.status_register0, 123);
        assert!(flash.boot_from_flash());
    }

    #[test]
    fn set32_aligned_control1_get_bit() {
        let mut flash = FlashController::new();
        flash.status_register0 = 123;
        flash.boot_from_flash = true;
        let mut memory = MemoryBank::new(0, 256);

        flash.set32_aligned(COMMAND_REGISTER1, 0x5A00010D, &mut memory);

        assert_eq!(flash.status_register0, 123);
        assert_eq!(flash.get32_aligned(RESULT_REGISTER0), 0);
        assert!(flash.boot_from_flash());
    }

    #[test]
    #[should_panic(expected = "Unsupported EEFC Command register value 0x5A000004")]
    fn set32_aligned_control1_unsupported() {
        let mut flash = FlashController::new();
        let mut memory = MemoryBank::new(0, 256);

        flash.set32_aligned(COMMAND_REGISTER1, 0x5A000004, &mut memory);
    }

    #[test]
    fn set32_aligned_status() {
        let mut flash = FlashController::new();
        flash.status_register0 = 123;
        flash.status_register1 = 456;
        let mut memory = MemoryBank::new(0, 256);

        flash.set32_aligned(STATUS_REGISTER0, 0, &mut memory);
        flash.set32_aligned(STATUS_REGISTER1, 0, &mut memory);

        assert_eq!(flash.status_register0, 123);
        assert_eq!(flash.status_register1, 456);
    }

    #[test]
    #[should_panic(expected = "Unsupported EEFC register 0x400E0C0C")]
    fn set32_aligned_unsupported() {
        let mut memory = MemoryBank::new(0, 256);
        FlashController::new().set32_aligned(0x400E0C0C, 0, &mut memory);
    }

    #[test]
    fn set32_aligned_result() {
        let mut flash = FlashController::new();
        flash.result_register0 = 123;
        let mut memory = MemoryBank::new(0, 256);

        flash.set32_aligned(RESULT_REGISTER0, 0, &mut memory);

        assert_eq!(flash.result_register0, 123);
    }

    #[test]
    fn erase() {
        let mut flash = FlashController::new();
        flash.boot_from_flash = true;

        flash.erase();

        assert!(!flash.boot_from_flash());
    }

    #[test]
    fn reset_boot_from_rom() {
        let mut flash = FlashController::new();
        flash.mode_register0 = 123;
        flash.status_register0 = 456;
        flash.result_register0 = 789;
        flash.mode_register1 = 0xABC;
        flash.status_register1 = 0xDEF;
        flash.boot_from_flash = false;
        flash.latch_buffer[0] = Option::Some(123456);

        flash.reset();

        assert_eq!(flash.mode_register0, 0x200);
        assert_eq!(flash.status_register0, READY_STATUS);
        assert_eq!(flash.result_register0, 0);
        assert_eq!(flash.mode_register1, 0x200);
        assert_eq!(flash.status_register1, READY_STATUS);
        assert!(!flash.boot_from_flash());
        assert_eq!(flash.latch_buffer[0], Option::None);
    }

    #[test]
    fn reset_boot_from_flash() {
        let mut flash = FlashController::new();
        flash.mode_register0 = 123;
        flash.status_register0 = 456;
        flash.result_register0 = 789;
        flash.mode_register1 = 0xABC;
        flash.status_register1 = 0xDEF;
        flash.boot_from_flash = true;
        flash.latch_buffer[0] = Option::Some(123456);

        flash.reset();

        assert_eq!(flash.mode_register0, 0);
        assert_eq!(flash.status_register0, READY_STATUS);
        assert_eq!(flash.result_register0, 0);
        assert_eq!(flash.mode_register1, 0);
        assert_eq!(flash.status_register1, READY_STATUS);
        assert!(flash.boot_from_flash());
        assert_eq!(flash.latch_buffer[0], Option::None);
    }
}
