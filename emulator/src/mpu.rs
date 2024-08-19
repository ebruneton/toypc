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

pub const MEMORY_PROTECTION_UNIT_BEGIN: u32 = 0xE000ED90;
pub const MEMORY_PROTECTION_UNIT_END: u32 = 0xE000EDA4;
pub const MEMORY_PROTECTION_UNIT_LAST: u32 = MEMORY_PROTECTION_UNIT_END - 1;

pub const TYPE_REGISTER: u32 = 0xE000ED90;
pub const CONTROL_REGISTER: u32 = 0xE000ED94;
pub const REGION_NUMBER_REGISTER: u32 = 0xE000ED98;
pub const REGION_BASE_ADDRESS_REGISTER: u32 = 0xE000ED9C;
pub const REGION_ATTRIBUTES_AND_SIZE_REGISTER: u32 = 0xE000EDA0;

#[derive(Clone, Copy, Default)]
struct MemoryRegion {
    enable: bool,
    base_address: u32,
    size: u8,
    attributes: u16,
    disabled_subregions: u8,
}

/// The MemoryProtectionUnit (MPU). See section 10.23, p197 of the Atmel SAM3X Datasheet.
#[derive(Clone)]
pub struct MemoryProtectionUnit {
    pub enable: bool,
    background_region_enable: bool,
    region_number: u32,
    regions: [MemoryRegion; 8],
    old_regions: [MemoryRegion; 8],
    // One bit per 32 bytes chunk, for the 4GB address space (=> 16MB).
    access_bits: Vec<u32>,
    is_dirty: bool,
}

impl MemoryProtectionUnit {
    // XN = 0 (no Execute Never), AP = 3 (Full Access), TEX = 0, S = 1, C = 1, B = 0 (recommended
    // attributes for internal SRAM, see Section 10.23.9.1 p209 of the Atmel SAM3X Datasheet).
    const SUPPORTED_ATTRIBUTES: u16 = 0x0306;
    const LOG2_CHUNK_SIZE: u8 = 5;
    const PRIVATE_PERIPHERAL_BUS_START: usize = 0xE0000000 >> (Self::LOG2_CHUNK_SIZE + 5);
    const PRIVATE_PERIPHERAL_BUS_END: usize = 0xE0100000 >> (Self::LOG2_CHUNK_SIZE + 5);

    pub fn uninitialized() -> Self {
        Self {
            enable: false,
            background_region_enable: false,
            region_number: 0,
            regions: [MemoryRegion::default(); 8],
            old_regions: [MemoryRegion::default(); 8],
            access_bits: Vec::new(),
            is_dirty: false,
        }
    }

    #[cfg(test)]
    pub fn new() -> Self {
        let mut result = Self::uninitialized();
        result.reset();
        result
    }

    pub fn get32_aligned(&self, address: u32) -> u32 {
        debug_assert!(address % 4 == 0);
        match address {
            TYPE_REGISTER => 0x00000800,
            CONTROL_REGISTER => ((self.background_region_enable as u32) << 2) | self.enable as u32,
            REGION_NUMBER_REGISTER => self.region_number,
            REGION_BASE_ADDRESS_REGISTER => {
                let region = &self.regions[self.region_number as usize];
                region.base_address | self.region_number
            }
            REGION_ATTRIBUTES_AND_SIZE_REGISTER => {
                let region = &self.regions[self.region_number as usize];
                (region.attributes as u32) << 16
                    | (region.disabled_subregions as u32) << 8
                    | (region.size as u32) << 1
                    | region.enable as u32
            }
            _ => panic!("Unsupported MPU register {address:#010X}"),
        }
    }

    pub fn set32_aligned(&mut self, address: u32, value: u32) {
        debug_assert!(address % 4 == 0);
        match address {
            TYPE_REGISTER => (),
            CONTROL_REGISTER => {
                if (value & 2) != 0 {
                    panic!("Unsupported MPU Control Register value {value:#010X}");
                }
                self.enable = (value & 1) != 0;
                self.background_region_enable = (value & 4) != 0;
            }
            REGION_NUMBER_REGISTER => {
                let region_number = value & 0xFF;
                if region_number >= 8 {
                    panic!("Unsupported MPU Region Number Register value {value}");
                }
                self.region_number = region_number;
            }
            REGION_BASE_ADDRESS_REGISTER => {
                self.set_dirty();
                if (value & 0x10) != 0 {
                    let region_number = value & 0xF;
                    if region_number >= 8 {
                        panic!("Unsupported MPU Region Number Register value {value:#010X}");
                    }
                    self.region_number = region_number;
                }
                self.regions[self.region_number as usize].base_address = value & 0xFFFFFFE0;
            }
            REGION_ATTRIBUTES_AND_SIZE_REGISTER => {
                self.set_dirty();
                let region = &mut self.regions[self.region_number as usize];
                region.attributes = (value >> 16) as u16;
                region.disabled_subregions = (value >> 8) as u8;
                region.size = ((value >> 1) & 0x1F) as u8;
                region.enable = (value & 1) != 0;
                if !region.enable {
                    return;
                }
                let n = region.size + 1;
                if region.attributes != Self::SUPPORTED_ATTRIBUTES
                    || n < Self::LOG2_CHUNK_SIZE
                    || (n < Self::LOG2_CHUNK_SIZE + 3 && region.disabled_subregions != 0)
                {
                    panic!(
                        "Unsupported MPU Region Attributes and Size Register value {:#010X}",
                        value
                    );
                }
            }
            _ => panic!("Unsupported MPU register {address:#010X}"),
        }
    }

    #[inline]
    pub fn validate_address(&mut self, address: u32, is_privileged: bool) -> bool {
        if !self.enable | (is_privileged & self.background_region_enable) {
            return true;
        }
        if self.is_dirty {
            self.update_access_bits();
        }
        let chunk = address >> Self::LOG2_CHUNK_SIZE;
        self.access_bits[(chunk >> 5) as usize] & (1 << (chunk & 31)) != 0
    }

    pub fn reset(&mut self) {
        self.enable = false;
        self.background_region_enable = false;
        self.region_number = 0;
        self.regions = [MemoryRegion::default(); 8];
        self.old_regions = [MemoryRegion::default(); 8];
        self.access_bits = vec![0; 4 * 1024 * 1024];
        self.is_dirty = false;
        self.set_private_peripheral_bus_access_bits();
    }

    fn set_dirty(&mut self) {
        if !self.is_dirty {
            self.old_regions = self.regions;
            self.is_dirty = true;
        }
    }

    fn update_access_bits(&mut self) {
        for region in &mut self.old_regions {
            Self::set_access_bits(&mut self.access_bits, region, false);
        }
        for region in &mut self.regions {
            Self::set_access_bits(&mut self.access_bits, region, true);
        }
        self.old_regions = self.regions;
        self.is_dirty = false;
        self.set_private_peripheral_bus_access_bits();
    }

    fn set_access_bits(access_bits: &mut [u32], region: &MemoryRegion, value: bool) {
        if !region.enable {
            return;
        }
        let n = region.size + 1;
        let start_chunk = (region.base_address >> n) << (n - Self::LOG2_CHUNK_SIZE);
        let end_chunk = start_chunk + (1 << (n - Self::LOG2_CHUNK_SIZE));
        let chunks_per_subregion = (1 << (n - 3)) >> Self::LOG2_CHUNK_SIZE;
        for chunk in start_chunk..end_chunk {
            if chunks_per_subregion != 0 {
                let subregion = (chunk - start_chunk) / chunks_per_subregion;
                if region.disabled_subregions & (1 << subregion) != 0 {
                    continue;
                }
            }
            if value {
                access_bits[(chunk >> 5) as usize] |= 1 << (chunk & 31);
            } else {
                access_bits[(chunk >> 5) as usize] &= !(1 << (chunk & 31));
            }
        }
    }

    fn set_private_peripheral_bus_access_bits(&mut self) {
        // Access to the Private Peripheral Bus (0xE00xxxxx) is always permitted by the MPU (but is
        // further restricted in unpriviledged mode).
        self.access_bits[Self::PRIVATE_PERIPHERAL_BUS_START..Self::PRIVATE_PERIPHERAL_BUS_END]
            .fill(0xFFFFFFFF);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PRIVATE_PERIPHERAL_BUS_START: usize = 0xE0000000 >> 10;

    #[test]
    fn new() {
        let mpu = MemoryProtectionUnit::new();

        assert_eq!(mpu.enable, false);
        assert_eq!(mpu.background_region_enable, false);
        assert_eq!(mpu.region_number, 0);
        for i in 0..8 {
            assert_eq!(mpu.regions[i].enable, false);
            assert_eq!(mpu.regions[i].base_address, 0);
            assert_eq!(mpu.regions[i].size, 0);
            assert_eq!(mpu.regions[i].attributes, 0);
            assert_eq!(mpu.regions[i].disabled_subregions, 0);
        }
        assert_eq!(mpu.is_dirty, false);
    }

    #[test]
    fn get32_aligned_type_register() {
        let mpu = MemoryProtectionUnit::new();

        assert_eq!(mpu.get32_aligned(TYPE_REGISTER), 0x800);
    }

    #[test]
    fn get32_aligned_control_register_default() {
        let mpu = MemoryProtectionUnit::new();

        assert_eq!(mpu.get32_aligned(CONTROL_REGISTER), 0);
    }

    #[test]
    fn get32_aligned_control_register_enabled() {
        let mut mpu = MemoryProtectionUnit::new();
        mpu.enable = true;

        assert_eq!(mpu.get32_aligned(CONTROL_REGISTER), 1);
    }

    #[test]
    fn get32_aligned_control_register_background_region_enabled() {
        let mut mpu = MemoryProtectionUnit::new();
        mpu.enable = true;
        mpu.background_region_enable = true;

        assert_eq!(mpu.get32_aligned(CONTROL_REGISTER), 5);
    }

    #[test]
    fn get32_aligned_region_number_register() {
        let mut mpu = MemoryProtectionUnit::new();
        mpu.region_number = 7;

        assert_eq!(mpu.get32_aligned(REGION_NUMBER_REGISTER), 7);
    }

    #[test]
    fn get32_aligned_region_base_register() {
        let mut mpu = MemoryProtectionUnit::new();
        mpu.region_number = 3;
        mpu.regions[3].base_address = 0x12340000;

        assert_eq!(mpu.get32_aligned(REGION_BASE_ADDRESS_REGISTER), 0x12340003);
    }

    #[test]
    fn get32_aligned_region_attributes_and_size_register() {
        let mut mpu = MemoryProtectionUnit::new();
        mpu.region_number = 3;
        mpu.regions[3].enable = true;
        mpu.regions[3].base_address = 0x12340000;
        mpu.regions[3].size = 15;
        mpu.regions[3].attributes = 0xCAFE;
        mpu.regions[3].disabled_subregions = 0xAB;

        assert_eq!(
            mpu.get32_aligned(REGION_ATTRIBUTES_AND_SIZE_REGISTER),
            0xCAFEAB1F
        );
    }

    #[test]
    fn set32_aligned_type_register() {
        let mut mpu = MemoryProtectionUnit::new();

        mpu.set32_aligned(TYPE_REGISTER, 0x12345678);

        assert_eq!(mpu.get32_aligned(TYPE_REGISTER), 0x800);
    }

    #[test]
    fn set32_aligned_control_register() {
        let mut mpu = MemoryProtectionUnit::new();

        mpu.set32_aligned(CONTROL_REGISTER, 0xFFFFFFFD);

        assert_eq!(mpu.get32_aligned(CONTROL_REGISTER), 5);
        assert_eq!(mpu.enable, true);
        assert_eq!(mpu.background_region_enable, true);
    }

    #[test]
    fn set32_aligned_control_register_enable() {
        let mut mpu = MemoryProtectionUnit::new();

        mpu.set32_aligned(CONTROL_REGISTER, 1);

        assert_eq!(mpu.get32_aligned(CONTROL_REGISTER), 1);
        assert_eq!(mpu.enable, true);
        assert_eq!(mpu.background_region_enable, false);
    }

    #[test]
    #[should_panic(expected = "Unsupported MPU Control Register value 0x00000002")]
    fn set32_aligned_control_register_unsupported() {
        let mut mpu = MemoryProtectionUnit::new();

        mpu.set32_aligned(CONTROL_REGISTER, 2);
    }

    #[test]
    fn set32_aligned_region_number_register() {
        let mut mpu = MemoryProtectionUnit::new();

        mpu.set32_aligned(REGION_NUMBER_REGISTER, 3);

        assert_eq!(mpu.get32_aligned(REGION_NUMBER_REGISTER), 3);
        assert_eq!(mpu.region_number, 3);
    }

    #[test]
    #[should_panic(expected = "Unsupported MPU Region Number Register value 8")]
    fn set32_aligned_region_number_register_unsupported() {
        let mut mpu = MemoryProtectionUnit::new();

        mpu.set32_aligned(REGION_NUMBER_REGISTER, 8);
    }

    #[test]
    fn set32_aligned_region_base_address_register() {
        let mut mpu = MemoryProtectionUnit::new();
        mpu.region_number = 3;

        mpu.set32_aligned(REGION_BASE_ADDRESS_REGISTER, 0xABCDEFEF);

        assert_eq!(mpu.get32_aligned(REGION_BASE_ADDRESS_REGISTER), 0xABCDEFE3);
        assert_eq!(mpu.region_number, 3);
        assert_eq!(mpu.regions[3].base_address, 0xABCDEFE0);
    }

    #[test]
    fn set32_aligned_region_base_address_register_valid() {
        let mut mpu = MemoryProtectionUnit::new();
        mpu.region_number = 3;

        mpu.set32_aligned(REGION_BASE_ADDRESS_REGISTER, 0xABCDEF15);

        assert_eq!(mpu.get32_aligned(REGION_BASE_ADDRESS_REGISTER), 0xABCDEF05);
        assert_eq!(mpu.region_number, 5);
        assert_eq!(mpu.regions[5].base_address, 0xABCDEF00);
    }

    #[test]
    #[should_panic(expected = "Unsupported MPU Region Number Register value 0xABCDEF18")]
    fn set32_aligned_region_base_address_register_unsupported() {
        let mut mpu = MemoryProtectionUnit::new();
        mpu.region_number = 3;

        mpu.set32_aligned(REGION_BASE_ADDRESS_REGISTER, 0xABCDEF18);
    }

    #[test]
    fn set32_aligned_region_attributes_and_size_register() {
        let mut mpu = MemoryProtectionUnit::new();
        mpu.region_number = 3;

        mpu.set32_aligned(REGION_ATTRIBUTES_AND_SIZE_REGISTER, 0x0306ABDF);

        assert_eq!(
            mpu.get32_aligned(REGION_ATTRIBUTES_AND_SIZE_REGISTER),
            0x0306AB1F
        );
        assert_eq!(mpu.region_number, 3);
        assert_eq!(mpu.regions[3].enable, true);
        assert_eq!(mpu.regions[3].attributes, 0x0306);
        assert_eq!(mpu.regions[3].size, 15);
        assert_eq!(mpu.regions[3].disabled_subregions, 0xAB);
    }

    #[test]
    fn set32_aligned_region_attributes_and_size_disable() {
        let mut mpu = MemoryProtectionUnit::new();
        mpu.region_number = 3;
        mpu.regions[3].enable = true;
        mpu.regions[3].base_address = 0x12340000;
        mpu.regions[3].size = 15;
        mpu.regions[3].attributes = 0xCAFE;
        mpu.regions[3].disabled_subregions = 0xAB;

        mpu.set32_aligned(REGION_ATTRIBUTES_AND_SIZE_REGISTER, 0);

        assert_eq!(mpu.get32_aligned(REGION_ATTRIBUTES_AND_SIZE_REGISTER), 0);
        assert_eq!(mpu.region_number, 3);
        assert_eq!(mpu.regions[3].enable, false);
        assert_eq!(mpu.regions[3].attributes, 0);
        assert_eq!(mpu.regions[3].size, 0);
        assert_eq!(mpu.regions[3].disabled_subregions, 0);
    }

    #[test]
    #[should_panic(
        expected = "Unsupported MPU Region Attributes and Size Register value 0x1234ABDF"
    )]
    fn set32_aligned_region_attributes_and_size_register_invalid_attributes() {
        let mut mpu = MemoryProtectionUnit::new();
        mpu.region_number = 3;

        mpu.set32_aligned(REGION_ATTRIBUTES_AND_SIZE_REGISTER, 0x1234ABDF);
    }

    #[test]
    #[should_panic(
        expected = "Unsupported MPU Region Attributes and Size Register value 0x03060003"
    )]
    fn set32_aligned_region_attributes_and_size_register_invalid_size() {
        let mut mpu = MemoryProtectionUnit::new();
        mpu.region_number = 3;

        mpu.set32_aligned(REGION_ATTRIBUTES_AND_SIZE_REGISTER, 0x03060003);
    }

    #[test]
    #[should_panic(
        expected = "Unsupported MPU Region Attributes and Size Register value 0x0306AB0D"
    )]
    fn set32_aligned_region_attributes_and_size_register_invalid_disabled_subregions() {
        let mut mpu = MemoryProtectionUnit::new();
        mpu.region_number = 3;

        mpu.set32_aligned(REGION_ATTRIBUTES_AND_SIZE_REGISTER, 0x0306AB0D);
    }

    #[test]
    fn validate_address_disabled() {
        let mut mpu = MemoryProtectionUnit::new();
        // Set-up region 3 to range [0x80000,0x80100[
        mpu.set32_aligned(REGION_BASE_ADDRESS_REGISTER, 0x00080013);
        mpu.set32_aligned(REGION_ATTRIBUTES_AND_SIZE_REGISTER, 0x0306000F);

        assert_eq!(mpu.validate_address(0, false), true);
        assert_eq!(mpu.validate_address(0x00080000, false), true);
        assert_eq!(mpu.validate_address(0x000800FC, false), true);
        assert_eq!(mpu.validate_address(0x00080100, false), true);
        assert_eq!(mpu.validate_address(0xE0000000, false), true);
        assert_eq!(mpu.validate_address(0xE00FFFFC, false), true);
        assert_eq!(mpu.validate_address(0xE0100000, false), true);
    }

    #[test]
    fn validate_address_enabled() {
        let mut mpu = MemoryProtectionUnit::new();
        mpu.set32_aligned(CONTROL_REGISTER, 1);
        // Set-up region 3 to range [0x80000,0x80100[
        mpu.set32_aligned(REGION_BASE_ADDRESS_REGISTER, 0x00080013);
        mpu.set32_aligned(REGION_ATTRIBUTES_AND_SIZE_REGISTER, 0x0306000F);

        assert_eq!(mpu.validate_address(0, false), false);
        assert_eq!(mpu.validate_address(0x00080000, false), true);
        assert_eq!(mpu.validate_address(0x000800FC, false), true);
        assert_eq!(mpu.validate_address(0x00080100, false), false);
        assert_eq!(mpu.validate_address(0xE0000000, false), true);
        assert_eq!(mpu.validate_address(0xE00FFFFC, false), true);
        assert_eq!(mpu.validate_address(0xE0100000, false), false);
    }

    #[test]
    fn validate_address_enabled_privileged_and_background_enabled() {
        let mut mpu = MemoryProtectionUnit::new();
        mpu.set32_aligned(CONTROL_REGISTER, 5);
        // Set-up region 3 to range [0x80000,0x80100[
        mpu.set32_aligned(REGION_BASE_ADDRESS_REGISTER, 0x00080013);
        mpu.set32_aligned(REGION_ATTRIBUTES_AND_SIZE_REGISTER, 0x0306000F);

        assert_eq!(mpu.validate_address(0, true), true);
        assert_eq!(mpu.validate_address(0x00080000, true), true);
        assert_eq!(mpu.validate_address(0x000800FC, true), true);
        assert_eq!(mpu.validate_address(0x00080100, true), true);
        assert_eq!(mpu.validate_address(0xE0000000, true), true);
        assert_eq!(mpu.validate_address(0xE00FFFFC, true), true);
        assert_eq!(mpu.validate_address(0xE0100000, true), true);
    }

    #[test]
    fn validate_address_enabled_privileged_and_background_disabled() {
        let mut mpu = MemoryProtectionUnit::new();
        mpu.set32_aligned(CONTROL_REGISTER, 1);
        // Set-up region 3 to range [0x80000,0x80100[
        mpu.set32_aligned(REGION_BASE_ADDRESS_REGISTER, 0x00080013);
        mpu.set32_aligned(REGION_ATTRIBUTES_AND_SIZE_REGISTER, 0x0306000F);

        assert_eq!(mpu.validate_address(0, true), false);
        assert_eq!(mpu.validate_address(0x00080000, true), true);
        assert_eq!(mpu.validate_address(0x000800FC, true), true);
        assert_eq!(mpu.validate_address(0x00080100, true), false);
        assert_eq!(mpu.validate_address(0xE0000000, true), true);
        assert_eq!(mpu.validate_address(0xE00FFFFC, true), true);
        assert_eq!(mpu.validate_address(0xE0100000, true), false);
    }

    #[test]
    fn validate_address_subregions_disabled() {
        let mut mpu = MemoryProtectionUnit::new();
        mpu.set32_aligned(CONTROL_REGISTER, 1);
        // Set-up region 3 to range [0x80000,0x80100[ but disabled
        mpu.set32_aligned(REGION_BASE_ADDRESS_REGISTER, 0x00080013);
        mpu.set32_aligned(REGION_ATTRIBUTES_AND_SIZE_REGISTER, 0x0306000E);
        // Set-up region 4 to range [0x90000,0x90200[, with subregions 1 and 5 disabled.
        mpu.set32_aligned(REGION_BASE_ADDRESS_REGISTER, 0x00090014);
        mpu.set32_aligned(REGION_ATTRIBUTES_AND_SIZE_REGISTER, 0x03062211);

        assert_eq!(mpu.validate_address(0, false), false);
        // Region 3 (disabled)
        assert_eq!(mpu.validate_address(0x00080000, false), false);
        assert_eq!(mpu.validate_address(0x000800FC, false), false);
        assert_eq!(mpu.validate_address(0x00080100, false), false);
        // Region 4
        assert_eq!(mpu.validate_address(0x00090000, false), true);
        assert_eq!(mpu.validate_address(0x0009002C, false), true);
        //   subregion 1
        assert_eq!(mpu.validate_address(0x00090040, false), false);
        assert_eq!(mpu.validate_address(0x0009007C, false), false);
        //   subregions 2-4
        assert_eq!(mpu.validate_address(0x00090080, false), true);
        assert_eq!(mpu.validate_address(0x0009013C, false), true);
        //   subregion 5
        assert_eq!(mpu.validate_address(0x00090140, false), false);
        assert_eq!(mpu.validate_address(0x0009017C, false), false);
        //   subregions 6-7
        assert_eq!(mpu.validate_address(0x00090180, false), true);
        assert_eq!(mpu.validate_address(0x000901FC, false), true);
        // Private Peripheral Bus
        assert_eq!(mpu.validate_address(0xE0000000, false), true);
        assert_eq!(mpu.validate_address(0xE00FFFFC, false), true);
        assert_eq!(mpu.validate_address(0xE0100000, false), false);
    }

    #[test]
    fn validate_address_updated_region() {
        let mut mpu = MemoryProtectionUnit::new();
        mpu.set32_aligned(CONTROL_REGISTER, 1);
        // Set-up region 3 to range [0x80000,0x80100[
        mpu.set32_aligned(REGION_BASE_ADDRESS_REGISTER, 0x00080013);
        mpu.set32_aligned(REGION_ATTRIBUTES_AND_SIZE_REGISTER, 0x0306000F);
        // Set-up region 4 to range [0xE0000000,0xE0000100[
        mpu.set32_aligned(REGION_BASE_ADDRESS_REGISTER, 0xE0000014);
        mpu.set32_aligned(REGION_ATTRIBUTES_AND_SIZE_REGISTER, 0x0306000F);

        let access0 = mpu.validate_address(0x00080000, false);
        let access1 = mpu.validate_address(0x00090000, false);
        let access2 = mpu.validate_address(0xE0000000, false);

        // Disable region 4
        mpu.set32_aligned(REGION_ATTRIBUTES_AND_SIZE_REGISTER, 0);
        // Update region 3 to range [0x90000,0x90100[
        mpu.set32_aligned(REGION_BASE_ADDRESS_REGISTER, 0x00090013);
        mpu.set32_aligned(REGION_ATTRIBUTES_AND_SIZE_REGISTER, 0x0306000F);

        assert_eq!(access0, true);
        assert_eq!(access1, false);
        assert_eq!(access2, true);
        assert_eq!(mpu.validate_address(0x00080000, false), false);
        assert_eq!(mpu.validate_address(0x00090000, false), true);
        assert_eq!(mpu.validate_address(0xE0000000, false), true);
    }

    #[test]
    fn reset() {
        let mut mpu = MemoryProtectionUnit::new();
        mpu.enable = true;
        mpu.background_region_enable = true;
        mpu.region_number = 3;
        mpu.regions[3].enable = true;
        mpu.regions[3].base_address = 0x12340000;
        mpu.regions[3].size = 15;
        mpu.regions[3].attributes = 0xCAFE;
        mpu.regions[3].disabled_subregions = 0xAB;
        mpu.access_bits[7] = 0xFF;
        mpu.access_bits[PRIVATE_PERIPHERAL_BUS_START] = 0;
        mpu.is_dirty = true;

        mpu.reset();

        assert_eq!(mpu.enable, false);
        assert_eq!(mpu.background_region_enable, false);
        assert_eq!(mpu.region_number, 0);
        assert_eq!(mpu.regions[3].enable, false);
        assert_eq!(mpu.regions[3].base_address, 0);
        assert_eq!(mpu.regions[3].size, 0);
        assert_eq!(mpu.regions[3].attributes, 0);
        assert_eq!(mpu.regions[3].disabled_subregions, 0);
        assert_eq!(mpu.access_bits[7], 0);
        assert_eq!(mpu.access_bits[PRIVATE_PERIPHERAL_BUS_START], 0xFFFFFFFF);
        assert_eq!(mpu.is_dirty, false);
    }
}
