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

pub const SYSTEM_CONTROL_BLOCK_BEGIN: u32 = 0xE000ED00;
pub const SYSTEM_CONTROL_BLOCK_END: u32 = 0xE000ED3C;
pub const SYSTEM_CONTROL_BLOCK_LAST: u32 = SYSTEM_CONTROL_BLOCK_END - 1;

pub const VECTOR_TABLE_OFFSET_REGISTER: u32 = 0xE000ED08;
pub const SYSTEM_HANDLER_PRIORITY_REGISTER2: u32 = 0xE000ED1C;

/// The System Control Block. See section 10.21, p165 of the Atmel SAM3X Datasheet.
/// This implementation only supports the Vector Table Offset Register and the SVC
/// Handler Priority Register.
#[derive(Clone)]
pub struct SystemControlBlock {
    vector_table_offset: u32,
    svc_priority: u8,
}

impl SystemControlBlock {
    pub fn new() -> Self {
        Self {
            vector_table_offset: 0,
            svc_priority: 0,
        }
    }

    pub fn get32_aligned(&self, address: u32) -> u32 {
        debug_assert!(address % 4 == 0);
        match address {
            VECTOR_TABLE_OFFSET_REGISTER => self.vector_table_offset,
            SYSTEM_HANDLER_PRIORITY_REGISTER2 => (self.svc_priority as u32) << 24,
            _ => panic!("Unsupported System Control Block Register {address:#010X}"),
        }
    }

    pub fn set32_aligned(&mut self, address: u32, value: u32) {
        debug_assert!(address % 4 == 0);
        match address {
            VECTOR_TABLE_OFFSET_REGISTER => self.vector_table_offset = value & 0x3FFFFF80,
            SYSTEM_HANDLER_PRIORITY_REGISTER2 => self.svc_priority = (value >> 24) as u8,
            _ => panic!("Unsupported System Control Block Register {address:#010X}"),
        }
    }

    pub fn reset(&mut self) {
        self.vector_table_offset = 0;
        self.svc_priority = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get32_aligned() {
        let system = SystemControlBlock::new();

        assert_eq!(system.get32_aligned(VECTOR_TABLE_OFFSET_REGISTER), 0);
        assert_eq!(system.get32_aligned(SYSTEM_HANDLER_PRIORITY_REGISTER2), 0);
    }

    #[test]
    #[should_panic(expected = "Unsupported System Control Block Register 0xE000ED00")]
    fn get32_aligned_unsupported() {
        SystemControlBlock::new().get32_aligned(SYSTEM_CONTROL_BLOCK_BEGIN);
    }

    #[test]
    fn set32_aligned() {
        let mut system = SystemControlBlock::new();

        system.set32_aligned(VECTOR_TABLE_OFFSET_REGISTER, 0xFFFF);
        system.set32_aligned(SYSTEM_HANDLER_PRIORITY_REGISTER2, 0xABCDEF01);

        assert_eq!(system.get32_aligned(VECTOR_TABLE_OFFSET_REGISTER), 0xFF80);
        assert_eq!(
            system.get32_aligned(SYSTEM_HANDLER_PRIORITY_REGISTER2),
            0xAB000000
        );
        assert_eq!(system.vector_table_offset, 0xFF80);
        assert_eq!(system.svc_priority, 0xAB);
    }

    #[test]
    #[should_panic(expected = "Unsupported System Control Block Register 0xE000ED00")]
    fn set32_aligned_unsupported() {
        SystemControlBlock::new().set32_aligned(SYSTEM_CONTROL_BLOCK_BEGIN, 123);
    }

    #[test]
    fn reset() {
        let mut system = SystemControlBlock::new();

        system.set32_aligned(VECTOR_TABLE_OFFSET_REGISTER, 0xFFFF);
        system.set32_aligned(SYSTEM_HANDLER_PRIORITY_REGISTER2, 0xFF000000);
        system.reset();

        assert_eq!(system.get32_aligned(VECTOR_TABLE_OFFSET_REGISTER), 0);
        assert_eq!(system.get32_aligned(SYSTEM_HANDLER_PRIORITY_REGISTER2), 0);
    }
}
