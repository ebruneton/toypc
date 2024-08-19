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

pub const BACKUP_REGISTERS_BEGIN: u32 = 0x400E1A90;
pub const BACKUP_REGISTERS_END: u32 = 0x400E1AB0;
pub const BACKUP_REGISTERS_LAST: u32 = BACKUP_REGISTERS_END - 1;

/// The General Purpose Backup Registers (GPBR). See section 17, p289 of the Atmel SAM3X Datasheet.
#[derive(Clone)]
pub struct BackupRegistersController {
    values: [u32; 8],
}

impl BackupRegistersController {
    pub fn new() -> Self {
        Self {
            values: [1, 2, 3, 4, 5, 6, 7, 8],
        }
    }

    pub fn get32_aligned(&mut self, address: u32) -> u32 {
        debug_assert!(address % 4 == 0);
        self.values[(address - BACKUP_REGISTERS_BEGIN) as usize >> 2]
    }

    pub fn set32_aligned(&mut self, address: u32, value: u32) {
        debug_assert!(address % 4 == 0);
        self.values[(address - BACKUP_REGISTERS_BEGIN) as usize >> 2] = value;
    }

    pub fn reset(&mut self) {
        self.values = [1, 2, 3, 4, 5, 6, 7, 8];
    }
}
