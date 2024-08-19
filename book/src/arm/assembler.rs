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

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ops::Range;

use crate::arm::Instruction;
use crate::arm::Instruction::*;
use crate::context::{ordered_map, Label, MemoryRegion, RegionKind};

pub enum Condition {
    EQ = 0b0000,
    NE = 0b0001,
    GE = 0b0010,
    LT = 0b0011,
    GT = 0b1000,
    LE = 0b1001,
}

pub enum ThenElse {
    Then,
    Else,
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct Assembler {
    base: u32,
    offset: u32,
    instructions: Vec<Instruction>,
    #[serde(serialize_with = "ordered_map")]
    label_offsets: HashMap<String, u32>,
}

impl Assembler {
    pub fn new(base: u32) -> Self {
        Self {
            base,
            ..Default::default()
        }
    }

    pub fn copy(&self, base: u32) -> Assembler {
        Assembler {
            base,
            offset: self.offset,
            instructions: self.instructions.clone(),
            label_offsets: self.label_offsets.clone(),
        }
    }

    pub fn add_rdn_imm8(&mut self, rdn: u32, imm8: u32) {
        self.instructions.push(AddRdnImm8 { rdn, imm8 });
        self.offset += 2;
    }

    pub fn add_rd_rn_rm(&mut self, rd: u32, rn: u32, rm: u32) {
        self.instructions.push(AddRdRnRm { rd, rn, rm });
        self.offset += 2;
    }

    pub fn add_rd_sp_imm8(&mut self, rd: u32, imm: u32) {
        self.instructions.push(AddRdSpImm8 { rd, imm });
        self.offset += 2;
    }

    pub fn add_sp_sp_imm7(&mut self, imm: u32) {
        self.instructions.push(AddSpSpImm7 { imm });
        self.offset += 2;
    }

    pub fn adr_rd_minus_imm12(&mut self, rd: u32, label: &str) {
        self.instructions.push(AdrRdMinusImm12 {
            rd,
            label: label.into(),
        });
        self.offset += 4;
    }

    pub fn and_rdn_rm(&mut self, rdn: u32, rm: u32) {
        self.instructions.push(AndRdnRm { rdn, rm });
        self.offset += 2;
    }

    pub fn b_imm11(&mut self, label: &str) {
        self.instructions.push(BImm11 {
            label: label.into(),
        });
        self.offset += 2;
    }

    pub fn bl_imm22(&mut self, label: &str) {
        self.instructions.push(BlImm22 {
            label: label.into(),
        });
        self.offset += 4;
    }

    pub fn blx_rm(&mut self, rm: u32) {
        self.instructions.push(BlxRm { rm });
        self.offset += 2;
    }

    pub fn bx_rm(&mut self, rm: u32) {
        self.instructions.push(BxRm { rm });
        self.offset += 2;
    }

    pub fn cbz_rn_imm6(&mut self, rn: u32, label: &str) {
        self.instructions.push(CbzRnImm6 {
            rn,
            label: label.into(),
        });
        self.offset += 2;
    }

    pub fn cmp_rn_imm8(&mut self, rn: u32, imm8: u32) {
        self.instructions.push(CmpRnImm8 { rn, imm8 });
        self.offset += 2;
    }

    pub fn cmp_rn_rm(&mut self, rn: u32, rm: u32) {
        self.instructions.push(CmpRnRm { rn, rm });
        self.offset += 2;
    }

    pub fn it(&mut self, first_cond_and_mask: u32) {
        self.instructions.push(It {
            first_cond_and_mask,
        });
        self.offset += 2;
    }

    pub fn if_then(&mut self, first_cond: Condition, then_else: &[ThenElse]) {
        debug_assert!(then_else.len() < 4);
        let cond = first_cond as u32;
        let mut mask = 0b1000;
        for x in then_else.iter().rev() {
            match x {
                ThenElse::Then => mask = (cond << 3 | mask >> 1) & 0xF,
                ThenElse::Else => mask = ((!cond) << 3 | mask >> 1) & 0xF,
            }
        }
        self.it(cond << 4 | mask);
    }

    pub fn ldrb_rt_rn_imm5(&mut self, rt: u32, rn: u32, imm5: u32) {
        self.instructions.push(LdrbRtRnImm5 { rt, rn, imm5 });
        self.offset += 2;
    }

    pub fn ldrh_rt_rn_imm5(&mut self, rt: u32, rn: u32, imm: u32) {
        self.instructions.push(LdrhRtRnImm5 { rt, rn, imm });
        self.offset += 2;
    }

    pub fn ldr_rt_pc_imm8(&mut self, rt: u32, label: &str) {
        self.instructions.push(LdrRtPcImm8 {
            rt,
            label: label.into(),
        });
        self.offset += 2;
    }

    pub fn ldr_rt_rn_imm5(&mut self, rt: u32, rn: u32, imm: u32) {
        self.instructions.push(LdrRtRnImm5 { rt, rn, imm });
        self.offset += 2;
    }

    pub fn ldr_rt_sp_imm8(&mut self, rt: u32, imm: u32) {
        self.instructions.push(LdrRtSpImm8 { rt, imm });
        self.offset += 2;
    }

    pub fn lsl_rdn_rm(&mut self, rdn: u32, rm: u32) {
        self.instructions.push(LslRdnRm { rdn, rm });
        self.offset += 2;
    }

    pub fn lsl_rd_rm_imm5(&mut self, rd: u32, rm: u32, imm5: u32) {
        self.instructions.push(LslRdRmImm5 { rd, rm, imm5 });
        self.offset += 2;
    }

    pub fn lsr_rdn_rm(&mut self, rdn: u32, rm: u32) {
        self.instructions.push(LsrRdnRm { rdn, rm });
        self.offset += 2;
    }

    pub fn lsr_rd_rm_imm5(&mut self, rd: u32, rm: u32, imm: u32) {
        self.instructions.push(LsrRdRmImm5 { rd, rm, imm });
        self.offset += 2;
    }

    pub fn mov_rd_imm8(&mut self, rd: u32, imm8: u32) {
        self.instructions.push(MovRdImm8 { rd, imm8 });
        self.offset += 2;
    }

    pub fn mov_rd_rm(&mut self, rd: u32, rm: u32) {
        self.instructions.push(MovRdRm { rd, rm });
        self.offset += 2;
    }

    pub fn movt_rd_imm16(&mut self, rd: u32, imm16: u32) {
        self.instructions.push(MovtRdImm16 { rd, imm16 });
        self.offset += 4;
    }

    pub fn movw_rd_imm16(&mut self, rd: u32, imm16: u32) {
        self.instructions.push(MovwRdImm16 { rd, imm16 });
        self.offset += 4;
    }

    pub fn mul_rdm_rn(&mut self, rdm: u32, rn: u32) {
        self.instructions.push(MulRdmRn { rdm, rn });
        self.offset += 2;
    }

    pub fn orr_rdn_rm(&mut self, rdn: u32, rm: u32) {
        self.instructions.push(OrrRdnRm { rdn, rm });
        self.offset += 2;
    }

    pub fn pop(&mut self, registers: u32, pc: bool) {
        self.instructions.push(Pop { registers, pc });
        self.offset += 2;
    }

    pub fn pop_list(&mut self, register_list: &[u32], pc: bool) {
        let mut registers = 0;
        for register in register_list {
            registers |= 1 << register;
        }
        self.pop(registers, pc);
    }

    pub fn push(&mut self, registers: u32, lr: bool) {
        self.instructions.push(Push { registers, lr });
        self.offset += 2;
    }

    pub fn push_list(&mut self, register_list: &[u32], lr: bool) {
        let mut registers = 0;
        for register in register_list {
            registers |= 1 << register;
        }
        self.push(registers, lr);
    }

    pub fn strb_rt_rn_imm5(&mut self, rt: u32, rn: u32, imm5: u32) {
        self.instructions.push(StrbRtRnImm5 { rt, rn, imm5 });
        self.offset += 2;
    }

    pub fn strh_rt_rn_imm5(&mut self, rt: u32, rn: u32, imm: u32) {
        self.instructions.push(StrhRtRnImm5 { rt, rn, imm });
        self.offset += 2;
    }

    pub fn str_rt_rn_imm5(&mut self, rt: u32, rn: u32, imm: u32) {
        self.instructions.push(StrRtRnImm5 { rt, rn, imm });
        self.offset += 2;
    }

    pub fn str_rt_sp_imm8(&mut self, rt: u32, imm: u32) {
        self.instructions.push(StrRtSpImm8 { rt, imm });
        self.offset += 2;
    }

    pub fn sub_rdn_imm8(&mut self, rdn: u32, imm8: u32) {
        self.instructions.push(SubRdnImm8 { rdn, imm8 });
        self.offset += 2;
    }

    pub fn sub_rd_rn_rm(&mut self, rd: u32, rn: u32, rm: u32) {
        self.instructions.push(SubRdRnRm { rd, rn, rm });
        self.offset += 2;
    }

    pub fn sub_sp_sp_imm7(&mut self, imm: u32) {
        self.instructions.push(SubSpSpImm7 { imm });
        self.offset += 2;
    }

    pub fn tbb_rn_rm(&mut self, rn: u32, rm: u32) {
        self.instructions.push(TbbRnRm { rn, rm });
        self.offset += 4;
    }

    pub fn tbb_rm_labels(&mut self, rm: u32, labels: &[&str]) {
        self.tbb_rn_rm(15, rm);
        let base = self.offset;
        for label in labels {
            self.instructions.push(TbbCase {
                base,
                label: String::from(*label),
            });
            self.offset += 1;
        }
    }

    pub fn udf(&mut self, imm8: u32) {
        self.instructions.push(Udf { imm8 });
        self.offset += 2;
    }

    pub fn udiv_rd_rn_rm(&mut self, rd: u32, rn: u32, rm: u32) {
        self.instructions.push(UdivRdRnRm { rd, rn, rm });
        self.offset += 4;
    }

    pub fn u32_data(&mut self, data: u32, comment: &str) {
        self.instructions.push(U32Data {
            data,
            comment: String::from(comment),
        });
        self.offset += 4;
    }

    pub fn u16_data(&mut self, data: u32, comment: &str) {
        self.instructions.push(U16Data {
            data,
            comment: String::from(comment),
        });
        self.offset += 2;
    }

    pub fn u8_data(&mut self, data: u32) {
        self.instructions.push(U8Data { data });
        self.offset += 1;
    }

    pub fn label(&mut self, label: &str) {
        assert_eq!(self.label_offsets.insert(label.into(), self.offset), None);
    }

    pub fn label_offset(&self, label: &str) -> u32 {
        *self.label_offsets.get(label).unwrap()
    }

    pub fn label_offsets(&self) -> &HashMap<String, u32> {
        &self.label_offsets
    }

    pub fn get_instruction_count(&self) -> u32 {
        self.instructions.len() as u32
    }

    pub fn get_instruction(&self, index: usize) -> (&Instruction, u32) {
        let mut offset = 0;
        for i in 0..index {
            offset += self.instructions[i].size_bytes();
        }
        let insn = &self.instructions[index];
        (insn, insn.encode(offset, &self.label_offsets))
    }

    pub fn get_listing_header() -> &'static str {
        r"\noindent {\scriptsize instruction}\hfill\makebox[2.8em][r]{\scriptsize encoding}\makebox[2.5em][r]{\scriptsize\color{gray} offset}\vspace{-1pt}"
    }

    pub fn get_listing(&self, range: Range<usize>) -> String {
        let mut offset = 0;
        for i in 0..range.start {
            offset += self.instructions[i].size_bytes();
        }
        let mut result = String::new();
        result.push_str(r"\vspace{0.4\baselineskip}\noindent ");
        for i in range {
            let insn = &self.instructions[i];
            if let U32Data { data, comment } = insn {
                result.push_str("\\makebox[3em][l]{data}");
                result.push_str(&format!(
                    "({})\\hfill\\makebox[5.6em][r]{{\\tt{:08X}}}",
                    comment, data
                ));
                result.push_str(&format!(
                    "\\makebox[2.5em][r]{{\\tt\\color{{gray}} {offset:03X}}}\\\\\n"
                ));
                offset += 4;
                continue;
            } else if let U16Data { data, comment } = insn {
                result.push_str("\\makebox[3em][l]{data}");
                result.push_str(&format!(
                    "({})\\hfill\\makebox[5.6em][r]{{\\tt{:04X}}}",
                    comment, data
                ));
                result.push_str(&format!(
                    "\\makebox[2.5em][r]{{\\tt\\color{{gray}} {offset:03X}}}\\\\\n"
                ));
                offset += 2;
                continue;
            }
            let format = insn.format();
            let encoding = insn.encode(offset, &self.label_offsets);
            let semantics = format.concrete_semantics(encoding, true);
            result.push_str(&format!("\\makebox[3em][l]{{\\arm{{{}}}}}", format.name()));
            result.push_str(semantics.as_str());
            result.push_str("\\hfill");
            result.push_str(format.concrete_bit_pattern(encoding).as_str());
            result.push_str(&format!(
                "\\makebox[2.8em][r]{{\\tt {:04X}}}",
                encoding & 0xFFFF
            ));
            result.push_str(&format!(
                "\\makebox[2.5em][r]{{\\tt\\color{{gray}} {offset:03X}}}"
            ));
            result.push_str("\\\\\n");
            if insn.is32bit() {
                result.push_str(r"\phantom{continued}\hfill");
                result.push_str(format.concrete_top_bit_pattern(encoding).as_str());
                result.push_str(&format!(
                    "\\makebox[2.8em][r]{{\\tt {:04X}}}",
                    encoding >> 16
                ));
                result.push_str(&format!(
                    "\\makebox[2.5em][r]{{\\tt\\color{{gray}} {:03X}}}",
                    offset + 2
                ));
                result.push_str("\\\\\n");
            }
            offset += self.instructions[i].size_bytes();
        }
        result.pop();
        result.push_str("[-0.5\\baselineskip]\n");
        result
    }

    pub fn machine_code_size(&self) -> u32 {
        self.offset
    }

    pub fn machine_code(&self) -> Vec<u32> {
        let mut result = vec![0; (self.offset + 3) as usize / 4];
        let mut set = |address: u32, value: u32| {
            let index = address as usize >> 2;
            let shift = (address & 3) << 3;
            result[index] |= value << shift;
        };
        let mut offset = 0;
        for insn in &self.instructions {
            let value = insn.encode(offset, &self.label_offsets);
            if insn.is32bit() {
                set(offset, value & 0xFFFF);
                set(offset + 2, value >> 16);
                offset += 4;
            } else if insn.is8bit() {
                debug_assert!(value & 0xFFFFFF00 == 0);
                set(offset, value);
                offset += 1;
            } else {
                debug_assert!(value & 0xFFFF0000 == 0);
                set(offset, value);
                offset += 2;
            }
        }
        debug_assert!(offset == self.offset);
        result
    }

    pub fn get_machine_code_listing(&self, range: Range<usize>) -> String {
        let mut start_offset = 0;
        for i in 0..range.start {
            start_offset += self.instructions[i].size_bytes();
        }
        let mut end_offset = start_offset;
        for i in range {
            end_offset += self.instructions[i].size_bytes();
        }
        let words = self.machine_code();
        let byte = |i| (words[i / 4] >> ((i % 4) * 8)) & 0xFF;

        let mut result = String::new();
        let mut offset = start_offset;
        let mut base = start_offset;
        result.push_str("\\vspace{0.5\\baselineskip}\\noindent");
        while offset < end_offset {
            result.push_str(r"\phantom{x}\hfill {\tt ");
            let end = 24 - offset % 4;
            for i in (0..end).rev() {
                if base + i < end_offset {
                    result.push_str(&format!("{:02X}", byte((base + i) as usize)));
                }
                if i != 0 && (offset + i) % 4 == 0 {
                    result.push(' ');
                }
            }
            for _ in 0..(offset % 4) {
                result.push_str("..");
            }
            offset -= offset % 4;
            result.push_str(&format!(
                "}} \\makebox[2.5em][r]{{\\tt\\color{{gray}} {offset:03X}}}\\\\\n"
            ));
            offset += 24;
            base += end;
        }
        result.pop();
        result.push_str("[-0.5\\baselineskip]\n");
        result
    }

    pub fn memory_region(&self) -> MemoryRegion {
        let mut labels = HashMap::new();
        for (label, offset) in &self.label_offsets {
            labels.insert(
                label.clone(),
                Label {
                    offset: *offset,
                    description: String::new(),
                },
            );
        }
        let mut instruction_count = 0;
        let mut data_bytes = 0;
        for insn in &self.instructions {
            match insn {
                TbbCase { .. } => data_bytes += 1,
                U8Data { .. } => data_bytes += 1,
                U16Data { .. } => data_bytes += 2,
                U32Data { .. } => data_bytes += 4,
                _ => instruction_count += 1,
            }
        }
        MemoryRegion::new(
            RegionKind::Default,
            self.base,
            self.machine_code_size(),
            &labels,
            instruction_count,
            self.machine_code_size() - data_bytes,
            data_bytes,
            self.machine_code(),
        )
    }

    pub fn boot_assistant_commands(&self) -> Vec<String> {
        crate::util::boot_assistant_commands(&self.machine_code(), self.base)
    }
}
