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

use crate::arm::{decode_insn, is32bit_insn, Instruction};

/// A continuous memory range with 32 bit words and a cache of corresponding decoded instructions.
#[derive(Clone)]
pub struct MemoryBank {
    words: Vec<u32>,
    instructions: Vec<Instruction>,
}

impl MemoryBank {
    pub fn uninitialized(num_words: u32) -> Self {
        Self {
            words: vec![0; num_words as usize],
            instructions: vec![Instruction::Unknown; 2 * num_words as usize],
        }
    }

    #[cfg(test)]
    pub fn new(initial_value: u32, num_words: u32) -> Self {
        let mut result = Self::uninitialized(num_words);
        result.reset(initial_value);
        result
    }

    pub fn get_content(&self) -> &Vec<u32> {
        &self.words
    }

    #[inline]
    pub fn get32_aligned(&self, address: u32) -> u32 {
        debug_assert!(address % 4 == 0);
        self.words[(address >> 2) as usize]
    }

    #[inline]
    pub fn set32_aligned(&mut self, address: u32, value: u32) {
        debug_assert!(address % 4 == 0);
        self.words[(address >> 2) as usize] = value;
        self.invalidate_insns(address >> 2);
    }

    #[inline]
    pub fn get_insn(&self, address: u32) -> Instruction {
        debug_assert!(address % 2 == 0);
        self.instructions[(address >> 1) as usize]
    }

    #[inline]
    fn invalidate_insns(&mut self, word_index: u32) {
        let insn_index = (word_index << 1) as usize;
        self.instructions[insn_index] = Instruction::Unknown;
        self.instructions[insn_index + 1] = Instruction::Unknown;
        self.instructions[insn_index.saturating_sub(1)] = Instruction::Unknown;
    }

    pub fn decode_insns(&mut self, address: u32, max_count: usize) {
        debug_assert!(address % 2 == 0);
        let mut insn_index = (address >> 1) as usize;
        let insn_count = self.instructions.len();
        let max_index = std::cmp::min(insn_index + max_count, insn_count);

        while insn_index < max_index && self.instructions[insn_index] == Instruction::Unknown {
            let mut raw_insn = self.get_raw_insn16(insn_index);
            if is32bit_insn(raw_insn) {
                if insn_index + 1 < insn_count {
                    raw_insn |= self.get_raw_insn16(insn_index + 1) << 16;
                } else {
                    break;
                }
            }
            self.instructions[insn_index] = decode_insn(raw_insn);
            debug_assert!(self.instructions[insn_index] != Instruction::Unknown);
            insn_index += 1;
        }
    }

    #[inline]
    fn get_raw_insn16(&self, index: usize) -> u32 {
        let word = self.words[index >> 1];
        let shift = (index & 1) << 4;
        (word >> shift) & 0xFFFF
    }

    pub fn reset(&mut self, initial_value: u32) {
        self.words.fill(initial_value);
        self.instructions.fill(Instruction::Unknown);
    }

    pub fn serialize(&self, output: &mut Vec<u32>) {
        output.extend(&self.words);
    }

    pub fn deserialize(&mut self, input: &mut Vec<u32>) {
        assert!(input.len() >= self.words.len());
        let new_len = input.len() - self.words.len();
        self.words.copy_from_slice(&input[new_len..]);
        input.truncate(new_len);
    }
}

#[cfg(test)]
#[allow(clippy::unusual_byte_groupings)]
mod tests {
    use super::MemoryBank;
    use crate::arm::Instruction;

    fn set16(words: &mut [u32], address: usize, value: u32) {
        debug_assert!(address % 2 == 0);
        let word = &mut words[address >> 2];
        let shift = (address & 3) << 3;
        *word = (*word & !(0xFFFF << shift)) | ((value & 0xFFFF) << shift);
    }

    #[test]
    fn new() {
        let memory_bank = MemoryBank::new(0, 16);
        for i in 0..16 {
            assert_eq!(memory_bank.get32_aligned(4 * i), 0);
            assert_eq!(memory_bank.get_insn(4 * i), Instruction::Unknown);
            assert_eq!(memory_bank.get_insn(4 * i + 2), Instruction::Unknown);
        }
    }

    #[test]
    fn get32() {
        let mut memory_bank = MemoryBank::new(0, 2);
        memory_bank.words[0] = 0x44332211;
        memory_bank.words[1] = 0x88776655;

        assert_eq!(memory_bank.get32_aligned(0), 0x44332211);
        assert_eq!(memory_bank.get32_aligned(4), 0x88776655);
    }

    #[test]
    fn set32() {
        let mut memory_bank = MemoryBank::new(0, 2);

        memory_bank.set32_aligned(0, 0x44332211);
        memory_bank.set32_aligned(4, 0x88776655);

        assert_eq!(memory_bank.get32_aligned(0), 0x44332211);
        assert_eq!(memory_bank.get32_aligned(4), 0x88776655);
    }

    #[test]
    fn decode_insns_full_range() {
        let mut memory_bank = MemoryBank::new(0, 2);
        set16(&mut memory_bank.words, 0, 0b010001110_0001_000);
        set16(&mut memory_bank.words, 2, 0b010001110_0010_000);

        memory_bank.decode_insns(0, 8);

        assert_eq!(memory_bank.get_insn(0), Instruction::BxRm { rm: 0b0001 });
        assert_eq!(memory_bank.get_insn(2), Instruction::BxRm { rm: 0b0010 });
        assert_eq!(memory_bank.get_insn(4), Instruction::Unsupported);
        assert_eq!(memory_bank.get_insn(6), Instruction::Unsupported);
    }

    #[test]
    fn decode_insns_partial_range() {
        let mut memory_bank = MemoryBank::new(0, 2);
        set16(&mut memory_bank.words, 0, 0b010001110_0001_000);
        set16(&mut memory_bank.words, 2, 0b010001110_0010_000);

        memory_bank.decode_insns(2, 1);

        assert_eq!(memory_bank.get_insn(0), Instruction::Unknown);
        assert_eq!(memory_bank.get_insn(2), Instruction::BxRm { rm: 0b0010 });
        assert_eq!(memory_bank.get_insn(4), Instruction::Unknown);
        assert_eq!(memory_bank.get_insn(6), Instruction::Unknown);
    }

    #[test]
    fn decode_insns_stops_at_already_decoded_insn() {
        let mut memory_bank = MemoryBank::new(0, 2);
        set16(&mut memory_bank.words, 0, 0b010001110_0001_000);
        set16(&mut memory_bank.words, 2, 0b010001110_0010_000);
        memory_bank.instructions[1] = Instruction::BxRm { rm: 0b1111 };
        memory_bank.instructions[2] = Instruction::BxRm { rm: 0b1110 };
        memory_bank.instructions[3] = Instruction::BxRm { rm: 0b1101 };

        memory_bank.decode_insns(0, 4);

        assert_eq!(memory_bank.get_insn(0), Instruction::BxRm { rm: 0b0001 });
        assert_eq!(memory_bank.get_insn(2), Instruction::BxRm { rm: 0b1111 });
        assert_eq!(memory_bank.get_insn(4), Instruction::BxRm { rm: 0b1110 });
        assert_eq!(memory_bank.get_insn(6), Instruction::BxRm { rm: 0b1101 });
    }

    #[test]
    fn decode_insns_32bit_insns() {
        let mut memory_bank = MemoryBank::new(0, 2);
        memory_bank.set32_aligned(0, 0b0_000_0011_00000111_11110_0_100100_0000);
        memory_bank.set32_aligned(4, 0b0_001_0011_00000111_11110_0_100100_0000);

        memory_bank.decode_insns(0, 4);

        assert_eq!(
            memory_bank.get_insn(0),
            Instruction::MovwRdImm16 { rd: 3, imm16: 7 }
        );
        assert_eq!(
            memory_bank.get_insn(2),
            Instruction::LslRdRmImm5 {
                rd: 7,
                rm: 0,
                imm5: 12
            }
        );
        assert_eq!(
            memory_bank.get_insn(4),
            Instruction::MovwRdImm16 { rd: 3, imm16: 263 }
        );
        assert_eq!(memory_bank.get_insn(6), Instruction::Unsupported);
    }

    #[test]
    fn decode_insns_truncated_32bit_insn() {
        let mut memory_bank = MemoryBank::new(0, 2);
        set16(&mut memory_bank.words, 0, 0b010001110_0001_000);
        set16(&mut memory_bank.words, 2, 0b010001110_0010_000);
        set16(&mut memory_bank.words, 4, 0b010001110_0011_000);
        set16(&mut memory_bank.words, 6, 0b111110000100_0010);

        memory_bank.decode_insns(0, 4);

        assert_eq!(memory_bank.get_insn(4), Instruction::BxRm { rm: 0b0011 });
        assert_eq!(memory_bank.get_insn(6), Instruction::Unknown);
    }

    #[test]
    fn set32_invalidates_insns() {
        let mut memory_bank = MemoryBank::new(0, 3);
        set16(&mut memory_bank.words, 0, 0b010001110_0001_000);
        set16(&mut memory_bank.words, 2, 0b010001110_0010_000);
        set16(&mut memory_bank.words, 4, 0b010001110_0011_000);
        set16(&mut memory_bank.words, 6, 0b010001110_0100_000);
        set16(&mut memory_bank.words, 8, 0b010001110_0101_000);
        set16(&mut memory_bank.words, 10, 0b010001110_0110_000);

        memory_bank.decode_insns(0, 8);
        memory_bank.set32_aligned(4, 0xAABBCCDD);

        assert_eq!(memory_bank.get_insn(0), Instruction::BxRm { rm: 0b0001 });
        assert_eq!(memory_bank.get_insn(2), Instruction::Unknown);
        assert_eq!(memory_bank.get_insn(4), Instruction::Unknown);
        assert_eq!(memory_bank.get_insn(6), Instruction::Unknown);
        assert_eq!(memory_bank.get_insn(8), Instruction::BxRm { rm: 0b0101 });
        assert_eq!(memory_bank.get_insn(10), Instruction::BxRm { rm: 0b0110 });
    }

    #[test]
    fn reset() {
        let mut memory_bank = MemoryBank::new(0, 2);
        memory_bank.set32_aligned(0, 0x44332211);
        memory_bank.set32_aligned(4, 0x88776655);
        memory_bank.instructions[1] = Instruction::BxRm { rm: 1 };
        memory_bank.instructions[2] = Instruction::BxRm { rm: 2 };
        memory_bank.instructions[3] = Instruction::BxRm { rm: 3 };
        memory_bank.instructions[3] = Instruction::BxRm { rm: 4 };

        memory_bank.reset(123);

        assert_eq!(memory_bank.get32_aligned(0), 123);
        assert_eq!(memory_bank.get32_aligned(4), 123);
        assert_eq!(memory_bank.get_insn(0), Instruction::Unknown);
        assert_eq!(memory_bank.get_insn(2), Instruction::Unknown);
        assert_eq!(memory_bank.get_insn(4), Instruction::Unknown);
        assert_eq!(memory_bank.get_insn(6), Instruction::Unknown);
    }
}
