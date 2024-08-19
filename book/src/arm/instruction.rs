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

use crate::arm::types::*;

/// An Armv7-M Thumb instruction. Only a small subset of all the existing instructions is supported.
#[derive(Clone, Serialize, Deserialize)]
pub enum Instruction {
    /// ADD (immediate), section A7.7.3, encoding T2.
    AddRdnImm8 {
        rdn: u32,
        imm8: u32,
    },
    /// ADD (register), section A7.7.4, encoding T1.
    AddRdRnRm {
        rd: u32,
        rn: u32,
        rm: u32,
    },
    /// ADD (SP plus immediate), section A7.7.5, encoding T1.
    AddRdSpImm8 {
        rd: u32,
        imm: u32,
    },
    /// ADD (SP plus immediate), section A7.7.5, encoding T2.
    AddSpSpImm7 {
        imm: u32,
    },
    /// ADR, section A7.7.7, encoding T2.
    AdrRdMinusImm12 {
        rd: u32,
        label: String,
    },
    /// AND (register), section A7.7.9, encoding T1.
    AndRdnRm {
        rdn: u32,
        rm: u32,
    },
    /// B, section A7.7.12, encoding T2.
    BImm11 {
        label: String,
    },
    /// BL, section A7.7.18, encoding T1.
    BlImm22 {
        label: String,
    },
    /// BLX (register), section A7.7.19, encoding T1.
    BlxRm {
        rm: u32,
    },
    /// BX, section A7.7.20, encoding T1.
    BxRm {
        rm: u32,
    },
    /// CBZ, section A7.7.21, encoding T1.
    CbzRnImm6 {
        rn: u32,
        label: String,
    },
    /// CMP (immediate), section A7.7.27, encoding T1.
    CmpRnImm8 {
        rn: u32,
        imm8: u32,
    },
    /// CMP (register), section A7.7.28, encoding T1.
    CmpRnRm {
        rn: u32,
        rm: u32,
    },
    /// IT, section A7.7.38, enconding T1.
    It {
        first_cond_and_mask: u32,
    },
    /// LDRB (immediate), section A7.7.46, encoding T1.
    LdrbRtRnImm5 {
        rt: u32,
        rn: u32,
        imm5: u32,
    },
    /// LDRH (immediate), section A7.7.55, encoding T1.
    LdrhRtRnImm5 {
        rt: u32,
        rn: u32,
        imm: u32,
    },
    /// LDR (literal), section A7.7.44, encoding T1.
    LdrRtPcImm8 {
        rt: u32,
        label: String,
    },
    /// LDR (immediate), section A7.7.43, encoding T1.
    LdrRtRnImm5 {
        rt: u32,
        rn: u32,
        imm: u32,
    },
    /// LDR (immediate), section A7.7.43, encoding T2.
    LdrRtSpImm8 {
        rt: u32,
        imm: u32,
    },
    /// LSL (register), section A7.7.69, encoding T1.
    LslRdnRm {
        rdn: u32,
        rm: u32,
    },
    /// LSL (immediate), section A7.7.68, encoding T1.
    LslRdRmImm5 {
        rd: u32,
        rm: u32,
        imm5: u32,
    },
    /// LSR (register), section A7.7.71, encoding T1.
    LsrRdnRm {
        rdn: u32,
        rm: u32,
    },
    /// LSR (immediate), section A7.7.70, encoding T1.
    LsrRdRmImm5 {
        rd: u32,
        rm: u32,
        imm: u32,
    },
    /// MOV (immediate), section A7.7.76, encoding T1.
    MovRdImm8 {
        rd: u32,
        imm8: u32,
    },
    /// MOV (register), section A7.7.77, encoding T1.
    MovRdRm {
        rd: u32,
        rm: u32,
    },
    /// MOVT, section A7.7.79, encoding T1.
    MovtRdImm16 {
        rd: u32,
        imm16: u32,
    },
    /// MOV (immediate), section A7.7.76, encoding T3.
    MovwRdImm16 {
        rd: u32,
        imm16: u32,
    },
    /// MUL, section A7.7.84, encoding T1.
    MulRdmRn {
        rdm: u32,
        rn: u32,
    },
    /// ORR (register), section A7.7.92, encoding T1.
    OrrRdnRm {
        rdn: u32,
        rm: u32,
    },
    /// POP, section A7.7.99, encoding T1.
    Pop {
        registers: u32,
        pc: bool,
    },
    /// PUSH, section A7.7.101, encoding T1.
    Push {
        registers: u32,
        lr: bool,
    },
    /// STRB (immediate) , section A7.7.163, encoding T1.
    StrbRtRnImm5 {
        rt: u32,
        rn: u32,
        imm5: u32,
    },
    /// STRH (immediate), section A7.7.170, encoding T1.
    StrhRtRnImm5 {
        rt: u32,
        rn: u32,
        imm: u32,
    },
    /// STR (immediate), section A7.7.161, encoding T1.
    StrRtRnImm5 {
        rt: u32,
        rn: u32,
        imm: u32,
    },
    /// STR (immediate), section A7.7.161, encoding T2.
    StrRtSpImm8 {
        rt: u32,
        imm: u32,
    },
    /// SUB (immediate), section A7.7.174, encoding T2.
    SubRdnImm8 {
        rdn: u32,
        imm8: u32,
    },
    /// SUB (register), section A7.7.175, encoding T1.
    SubRdRnRm {
        rd: u32,
        rn: u32,
        rm: u32,
    },
    /// SUB (SP minus immediate), section A7.7.176, encoding T1.
    SubSpSpImm7 {
        imm: u32,
    },
    /// TBB, section A7.7.185, encoding T1.
    TbbRnRm {
        rn: u32,
        rm: u32,
    },
    TbbCase {
        base: u32,
        label: String,
    },
    // UDF, section A7.7.194, encoding T1.
    Udf {
        imm8: u32,
    },
    /// UDIV, section A7.7.195, encoding T1.
    UdivRdRnRm {
        rd: u32,
        rn: u32,
        rm: u32,
    },
    /// Arbitrary data.
    U32Data {
        data: u32,
        comment: String,
    },
    /// Arbitrary data.
    U16Data {
        data: u32,
        comment: String,
    },
    /// Arbitrary data.
    U8Data {
        data: u32,
    },
}

fn div(numerator: u32, denominator: u32) -> u32 {
    assert_eq!(numerator % denominator, 0);
    numerator / denominator
}

fn check(value: u32, num_bits: u32) -> u32 {
    assert!(value < (1 << num_bits));
    value
}

impl Instruction {
    pub fn is32bit(&self) -> bool {
        use Instruction::*;
        matches!(
            self,
            AdrRdMinusImm12 { .. }
                | BlImm22 { .. }
                | MovtRdImm16 { .. }
                | MovwRdImm16 { .. }
                | TbbRnRm { .. }
                | UdivRdRnRm { .. }
                | U32Data { .. }
        )
    }

    pub fn is8bit(&self) -> bool {
        use Instruction::*;
        matches!(self, TbbCase { .. } | U8Data { .. })
    }

    pub fn size_bytes(&self) -> u32 {
        if self.is32bit() {
            4
        } else if self.is8bit() {
            1
        } else {
            2
        }
    }

    pub fn format(&self) -> &'static dyn InstructionFormat {
        use Instruction::*;
        match self {
            AddRdnImm8 { .. } => &ADD_RDN_IMM8,
            AddRdRnRm { .. } => &ADD_RD_RN_RM,
            AddRdSpImm8 { .. } => &ADD_RD_SP_IMM8,
            AddSpSpImm7 { .. } => &ADD_SP_SP_IMM7,
            AdrRdMinusImm12 { .. } => &ADR_RD_MINUS_IMM12,
            AndRdnRm { .. } => &AND_RDN_RM,
            BImm11 { .. } => &B_IMM11,
            BlImm22 { .. } => &BL_IMM22,
            BlxRm { .. } => &BLX_RM,
            BxRm { .. } => &BX_RM,
            CbzRnImm6 { .. } => &CBZ_RN_IMM6,
            CmpRnImm8 { .. } => &CMP_RN_IMM8,
            CmpRnRm { .. } => &CMP_RN_RM,
            It { .. } => &IT,
            LdrbRtRnImm5 { .. } => &LDRB_RT_RN_IMM5,
            LdrhRtRnImm5 { .. } => &LDRH_RT_RN_IMM5,
            LdrRtPcImm8 { .. } => &LDR_RT_PC_IMM8,
            LdrRtRnImm5 { .. } => &LDR_RT_RN_IMM5,
            LdrRtSpImm8 { .. } => &LDR_RT_SP_IMM8,
            LslRdnRm { .. } => &LSL_RDN_RM,
            LslRdRmImm5 { .. } => &LSL_RD_RM_IMM5,
            LsrRdnRm { .. } => &LSR_RDN_RM,
            LsrRdRmImm5 { .. } => &LSR_RD_RM_IMM5,
            MovRdImm8 { .. } => &MOV_RD_IMM8,
            MovRdRm { .. } => &MOV_RD_RM,
            MovtRdImm16 { .. } => &MOVT_RD_IMM16,
            MovwRdImm16 { .. } => &MOVW_RD_IMM16,
            MulRdmRn { .. } => &MUL_RDM_RN,
            OrrRdnRm { .. } => &ORR_RDN_RM,
            Pop { .. } => &POP,
            Push { .. } => &PUSH,
            StrbRtRnImm5 { .. } => &STRB_RT_RN_IMM5,
            StrhRtRnImm5 { .. } => &STRH_RT_RN_IMM5,
            StrRtRnImm5 { .. } => &STR_RT_RN_IMM5,
            StrRtSpImm8 { .. } => &STR_RT_SP_IMM8,
            SubRdnImm8 { .. } => &SUB_RDN_IMM8,
            SubRdRnRm { .. } => &SUB_RD_RN_RM,
            SubSpSpImm7 { .. } => &SUB_SP_SP_IMM7,
            TbbRnRm { .. } => &TBB_RN_RM,
            Udf { .. } => &UDF,
            UdivRdRnRm { .. } => &UDIV_RD_RN_RM,
            TbbCase { .. } | U32Data { .. } | U16Data { .. } | U8Data { .. } => panic!(),
        }
    }

    pub fn encode(&self, offset: u32, label_offsets: &HashMap<String, u32>) -> u32 {
        use Instruction::*;
        match self {
            AddRdnImm8 { rdn, imm8 } => ADD_RDN_IMM8.encode(&[*rdn, *imm8]),
            AddRdRnRm { rd, rn, rm } => ADD_RD_RN_RM.encode(&[*rd, *rn, *rm]),
            AddRdSpImm8 { rd, imm } => ADD_RD_SP_IMM8.encode(&[*rd, *imm]),
            AddSpSpImm7 { imm } => ADD_SP_SP_IMM7.encode(&[*imm]),
            AdrRdMinusImm12 { rd, label } => {
                let source = (offset + 4) & 0xFFFFFFFC;
                let target = *label_offsets.get(label).unwrap();
                assert!(target <= source);
                ADR_RD_MINUS_IMM12.encode(&[*rd, source - target])
            }
            AndRdnRm { rdn, rm } => AND_RDN_RM.encode(&[*rdn, *rm]),
            BImm11 { label } => {
                let source = offset + 4;
                let target = *label_offsets.get(label).unwrap();
                let imm32 = target as i32 - source as i32;
                B_IMM11.encode(&[imm32 as u32])
            }
            BlImm22 { label } => {
                let source = offset + 4;
                let target = *label_offsets.get(label).unwrap();
                let imm32 = target as i32 - source as i32;
                BL_IMM22.encode(&[imm32 as u32])
            }
            BlxRm { rm } => BLX_RM.encode(&[*rm]),
            BxRm { rm } => BX_RM.encode(&[*rm]),
            CbzRnImm6 { rn, label } => {
                let source = offset + 4;
                let target = *label_offsets.get(label).unwrap();
                assert!(target >= source);
                CBZ_RN_IMM6.encode(&[*rn, target - source])
            }
            CmpRnImm8 { rn, imm8 } => CMP_RN_IMM8.encode(&[*rn, *imm8]),
            CmpRnRm { rn, rm } => CMP_RN_RM.encode(&[*rn, *rm]),
            It {
                first_cond_and_mask,
            } => IT.encode(&[first_cond_and_mask >> 4, first_cond_and_mask & 0xF]),
            LdrbRtRnImm5 { rt, rn, imm5 } => LDRB_RT_RN_IMM5.encode(&[*rt, *rn, *imm5]),
            LdrhRtRnImm5 { rt, rn, imm } => LDRH_RT_RN_IMM5.encode(&[*rt, *rn, *imm]),
            LdrRtPcImm8 { rt, label } => {
                let source = (offset + 4) & 0xFFFFFFFC;
                let target = *label_offsets.get(label).unwrap();
                assert!(target >= source);
                LDR_RT_PC_IMM8.encode(&[*rt, target - source])
            }
            LdrRtRnImm5 { rt, rn, imm } => LDR_RT_RN_IMM5.encode(&[*rt, *rn, *imm]),
            LdrRtSpImm8 { rt, imm } => LDR_RT_SP_IMM8.encode(&[*rt, *imm]),
            LslRdnRm { rdn, rm } => LSL_RDN_RM.encode(&[*rdn, *rm]),
            LslRdRmImm5 { rd, rm, imm5 } => LSL_RD_RM_IMM5.encode(&[*rd, *rm, *imm5]),
            LsrRdnRm { rdn, rm } => LSR_RDN_RM.encode(&[*rdn, *rm]),
            LsrRdRmImm5 { rd, rm, imm } => LSR_RD_RM_IMM5.encode(&[*rd, *rm, *imm]),
            MovRdImm8 { rd, imm8 } => MOV_RD_IMM8.encode(&[*rd, *imm8]),
            MovRdRm { rd, rm } => MOV_RD_RM.encode(&[*rd, *rm]),
            MovtRdImm16 { rd, imm16 } => MOVT_RD_IMM16.encode(&[*rd, *imm16]),
            MovwRdImm16 { rd, imm16 } => MOVW_RD_IMM16.encode(&[*rd, *imm16]),
            MulRdmRn { rdm, rn } => MUL_RDM_RN.encode(&[*rdm, *rn]),
            OrrRdnRm { rdn, rm } => ORR_RDN_RM.encode(&[*rdn, *rm]),
            Pop { registers, pc } => POP.encode(&[*registers, *pc as u32]),
            Push { registers, lr } => PUSH.encode(&[*registers, *lr as u32]),
            StrbRtRnImm5 { rt, rn, imm5 } => STRB_RT_RN_IMM5.encode(&[*rt, *rn, *imm5]),
            StrhRtRnImm5 { rt, rn, imm } => STRH_RT_RN_IMM5.encode(&[*rt, *rn, *imm]),
            StrRtRnImm5 { rt, rn, imm } => STR_RT_RN_IMM5.encode(&[*rt, *rn, *imm]),
            StrRtSpImm8 { rt, imm } => STR_RT_SP_IMM8.encode(&[*rt, *imm]),
            SubRdnImm8 { rdn, imm8 } => SUB_RDN_IMM8.encode(&[*rdn, *imm8]),
            SubRdRnRm { rd, rn, rm } => SUB_RD_RN_RM.encode(&[*rd, *rn, *rm]),
            SubSpSpImm7 { imm } => SUB_SP_SP_IMM7.encode(&[*imm]),
            TbbRnRm { rn, rm } => TBB_RN_RM.encode(&[*rn, *rm]),
            TbbCase { base, label } => {
                let target = *label_offsets.get(label).unwrap();
                assert!(target > *base);
                let offset = target - *base;
                check(div(offset, 2), 8)
            }
            Udf { imm8 } => UDF.encode(&[*imm8]),
            UdivRdRnRm { rd, rn, rm } => UDIV_RD_RN_RM.encode(&[*rd, *rn, *rm]),
            U32Data { data, comment: _ } => *data,
            U16Data { data, comment: _ } => check(*data, 16),
            U8Data { data } => check(*data, 8),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::arm::instruction::Instruction;
    use crate::arm::instruction::Instruction::*;
    use std::collections::HashMap;

    type BitPattern = str;
    type BitValues = u32;

    fn value(pattern: &BitPattern, values: BitValues) -> u32 {
        let mut result = 0;
        for c in pattern.chars() {
            result = match c {
                '_' => result,
                '0' => result << 1,
                '1' => (result << 1) | 1,
                'a'..='z' => (result << 1) | (values >> (c as u32 - 'a' as u32)) & 1,
                'A'..='Z' => (result << 1) | !(values >> (c as u32 - 'A' as u32)) & 1,
                _ => panic!("Unsupported bit pattern character {}", c),
            }
        }
        result
    }

    fn encode(insn: Instruction) -> u32 {
        insn.encode(0, &HashMap::default())
    }

    fn test_cases(pattern: &BitPattern) -> Vec<BitValues> {
        let num_bits = value(pattern.replace('1', "0").as_str(), 0xFFFFFFFF).count_ones();
        let mut result = Vec::new();
        for i in 0..num_bits {
            result.push(1 << i);
            for j in 0..num_bits {
                if i != j {
                    result.push((1 << i) | (1 << j));
                }
            }
        }
        result
    }

    #[test]
    fn add_rdn_imm8() {
        let insn = "00110_abc_defghijk";
        for bit_values in test_cases(insn) {
            assert_eq!(
                encode(AddRdnImm8 {
                    rdn: value("abc", bit_values),
                    imm8: value("defghijk", bit_values)
                }),
                value(insn, bit_values)
            );
        }
    }

    #[test]
    fn add_rd_rn_rm() {
        let insn = "0001100_abc_def_ghi";
        for bit_values in test_cases(insn) {
            assert_eq!(
                encode(AddRdRnRm {
                    rd: value("ghi", bit_values),
                    rn: value("def", bit_values),
                    rm: value("abc", bit_values)
                }),
                value(insn, bit_values)
            );
        }
    }

    #[test]
    fn add_rd_sp_imm8() {
        let insn = "10101_abc_defghijk";
        for bit_values in test_cases(insn) {
            assert_eq!(
                encode(AddRdSpImm8 {
                    rd: value("abc", bit_values),
                    imm: value("defghijk_00", bit_values)
                }),
                value(insn, bit_values)
            );
        }
    }

    #[test]
    fn add_sp_sp_imm7() {
        let insn = "101100000_abcdefg";
        for bit_values in test_cases(insn) {
            assert_eq!(
                encode(AddSpSpImm7 {
                    imm: value("abcdefg_00", bit_values)
                }),
                value(insn, bit_values)
            );
        }
    }

    #[test]
    fn adr_rd_minus_imm12() {
        let offset = (1 << 20) + 14;
        let insn = "0_abc_defg_hijklmno_11110z1010101111";
        for bit_values in test_cases(insn) {
            let mut label_offsets = HashMap::<String, u32>::default();
            let label = String::from("label");
            label_offsets.insert(
                label.clone(),
                ((offset + 4) & 0xFFFFFFFC) - value("z_abc_hijklmno", bit_values),
            );
            assert_eq!(
                AdrRdMinusImm12 {
                    rd: value("defg", bit_values),
                    label
                }
                .encode(offset, &label_offsets),
                value(insn, bit_values)
            );
        }
    }

    #[test]
    fn and_rdn_rm() {
        let insn = "0100000000_abc_def";
        for bit_values in test_cases(insn) {
            assert_eq!(
                encode(AndRdnRm {
                    rdn: value("def", bit_values),
                    rm: value("abc", bit_values)
                }),
                value(insn, bit_values)
            );
        }
    }

    #[test]
    fn b_imm11() {
        let offset = (1 << 16) + 2;
        let insn = "11100_abcdefghijk";
        for bit_values in test_cases(insn) {
            let mut label_offsets = HashMap::<String, u32>::default();
            let label = String::from("label");
            let imm12 = value("abcdefghijk_0", bit_values);
            let imm32 = ((imm12 as i32) << 20) >> 20;
            label_offsets.insert(label.clone(), (((offset + 4) as i32) + imm32) as u32);
            assert_eq!(
                BImm11 { label }.encode(offset, &label_offsets),
                value(insn, bit_values)
            );
        }
    }

    #[test]
    fn bl_plus_imm21() {
        let offset = 2;
        let insn = "11_1_1_1_cdefghijklm_111100_nopqrstuvw";
        for bit_values in test_cases(insn) {
            let mut label_offsets = HashMap::<String, u32>::default();
            let label = String::from("label");
            label_offsets.insert(
                label.clone(),
                (offset + 4) + value("nopqrstuvw_cdefghijklm_0", bit_values),
            );
            assert_eq!(
                BlImm22 { label }.encode(offset, &label_offsets),
                value(insn, bit_values)
            );
        }
    }

    #[test]
    fn bl_minus_imm21() {
        let offset: u32 = (1 << 28) + 2;
        let insn = "11_1_1_1_cdefghijklm_111101_nopqrstuvw";
        for bit_values in test_cases(insn) {
            let mut label_offsets = HashMap::<String, u32>::default();
            let label = String::from("label");
            label_offsets.insert(
                label.clone(),
                (offset + 4)
                    .wrapping_add(value("11111111_1_1_nopqrstuvw_cdefghijklm_0", bit_values)),
            );
            assert_eq!(
                BlImm22 { label }.encode(offset, &label_offsets),
                value(insn, bit_values)
            );
        }
    }

    #[test]
    fn blx_rm() {
        let insn = "010001111_abcd_000";
        for bit_values in test_cases(insn) {
            assert_eq!(
                encode(BlxRm {
                    rm: value("abcd", bit_values),
                }),
                value(insn, bit_values)
            );
        }
    }

    #[test]
    fn bx_rm() {
        let insn = "010001110_abcd_000";
        for bit_values in test_cases(insn) {
            assert_eq!(
                encode(BxRm {
                    rm: value("abcd", bit_values),
                }),
                value(insn, bit_values),
            );
        }
    }

    #[test]
    fn cbz_rm_imm6() {
        let offset = 2;
        let insn = "101100_i_1_abcde_fgh";
        for bit_values in test_cases(insn) {
            let mut label_offsets = HashMap::<String, u32>::default();
            let label = String::from("label");
            label_offsets.insert(label.clone(), (offset + 4) + value("i_abcde_0", bit_values));
            assert_eq!(
                CbzRnImm6 {
                    rn: value("fgh", bit_values),
                    label,
                }
                .encode(offset, &label_offsets),
                value(insn, bit_values)
            );
        }
    }

    #[test]
    fn cmp_rn_imm8() {
        let insn = "00101_abc_defghijk";
        for bit_values in test_cases(insn) {
            assert_eq!(
                encode(CmpRnImm8 {
                    rn: value("abc", bit_values),
                    imm8: value("defghijk", bit_values),
                }),
                value(insn, bit_values)
            );
        }
    }

    #[test]
    fn cmp_rn_rm() {
        let insn = "0100001010_abc_def";
        for bit_values in test_cases(insn) {
            assert_eq!(
                encode(CmpRnRm {
                    rn: value("def", bit_values),
                    rm: value("abc", bit_values),
                }),
                value(insn, bit_values)
            );
        }
    }

    #[test]
    fn it() {
        let insn = "10111111_abcd_efgh";
        for bit_values in test_cases(insn) {
            assert_eq!(
                encode(It {
                    first_cond_and_mask: value("abcd_efgh", bit_values),
                }),
                value(insn, bit_values)
            );
        }
    }

    #[test]
    fn ldrb_rt_rn_imm5() {
        let insn = "01111_abcde_fgh_ijk";
        for bit_values in test_cases(insn) {
            assert_eq!(
                encode(LdrbRtRnImm5 {
                    rt: value("ijk", bit_values),
                    rn: value("fgh", bit_values),
                    imm5: value("abcde", bit_values),
                }),
                value(insn, bit_values)
            );
        }
    }

    #[test]
    fn ldrh_rt_rn_imm5() {
        let insn = "10001_abcde_fgh_ijk";
        for bit_values in test_cases(insn) {
            assert_eq!(
                encode(LdrhRtRnImm5 {
                    rt: value("ijk", bit_values),
                    rn: value("fgh", bit_values),
                    imm: value("abcde_0", bit_values),
                }),
                value(insn, bit_values),
            );
        }
    }

    #[test]
    fn ldr_rt_pc_imm8() {
        let offset = 2;
        let insn = "01001_abc_defghijk";
        for bit_values in test_cases(insn) {
            let mut label_offsets = HashMap::<String, u32>::default();
            let label = String::from("label");
            label_offsets.insert(
                label.clone(),
                ((offset + 4) & 0xFFFFFFFC) + value("defghijk_00", bit_values),
            );
            assert_eq!(
                LdrRtPcImm8 {
                    rt: value("abc", bit_values),
                    label,
                }
                .encode(offset, &label_offsets),
                value(insn, bit_values)
            );
        }
    }

    #[test]
    fn ldr_rt_rn_imm5() {
        let insn = "01101_abcde_fgh_ijk";
        for bit_values in test_cases(insn) {
            assert_eq!(
                encode(LdrRtRnImm5 {
                    rt: value("ijk", bit_values),
                    rn: value("fgh", bit_values),
                    imm: value("abcde_00", bit_values),
                }),
                value(insn, bit_values)
            );
        }
    }

    #[test]
    fn ldr_rt_sp_imm8() {
        let insn = "10011_abc_efghijkl";
        for bit_values in test_cases(insn) {
            assert_eq!(
                encode(LdrRtSpImm8 {
                    rt: value("abc", bit_values),
                    imm: value("efghijkl_00", bit_values),
                }),
                value(insn, bit_values)
            );
        }
    }

    #[test]
    fn lsl_rdn_rm() {
        let insn = "0100000010_abc_def";
        for bit_values in test_cases(insn) {
            assert_eq!(
                encode(LslRdnRm {
                    rdn: value("def", bit_values),
                    rm: value("abc", bit_values),
                }),
                value(insn, bit_values)
            );
        }
    }

    #[test]
    fn lsl_rd_rm_imm5() {
        let insn = "00000_abcde_fgh_ijk";
        for bit_values in test_cases(insn) {
            if value("abcde", bit_values) == 0 {
                continue;
            }
            assert_eq!(
                encode(LslRdRmImm5 {
                    rd: value("ijk", bit_values),
                    rm: value("fgh", bit_values),
                    imm5: value("abcde", bit_values),
                }),
                value(insn, bit_values)
            );
        }
    }

    #[test]
    fn lsr_rdn_rm() {
        let insn = "0100000011_abc_def";
        for bit_values in test_cases(insn) {
            assert_eq!(
                encode(LsrRdnRm {
                    rdn: value("def", bit_values),
                    rm: value("abc", bit_values),
                }),
                value(insn, bit_values)
            );
        }
    }

    #[test]
    fn lsr_rd_rm_imm5() {
        let insn = "00001_abcde_fgh_ijk";
        for bit_values in test_cases(insn) {
            let imm5 = value("abcde", bit_values);
            assert_eq!(
                encode(LsrRdRmImm5 {
                    rd: value("ijk", bit_values),
                    rm: value("fgh", bit_values),
                    imm: if imm5 == 0 { 32 } else { imm5 },
                }),
                value(insn, bit_values)
            );
        }
    }

    #[test]
    fn mov_rd_imm8() {
        let insn = "00100_abc_defghijk";
        for bit_values in test_cases(insn) {
            assert_eq!(
                encode(MovRdImm8 {
                    rd: value("abc", bit_values),
                    imm8: value("defghijk", bit_values),
                }),
                value(insn, bit_values)
            );
        }
    }

    #[test]
    fn mov_rd_rm() {
        let insn = "01000110_a_bcde_fgh";
        for bit_values in test_cases(insn) {
            assert_eq!(
                encode(MovRdRm {
                    rd: value("a_fgh", bit_values),
                    rm: value("bcde", bit_values),
                }),
                value(insn, bit_values)
            );
        }
    }

    #[test]
    fn movt_rd_imm16() {
        let insn = "0_abc_defg_hijklmno_11110_p_101100_qrst";
        for bit_values in test_cases(insn) {
            assert_eq!(
                encode(MovtRdImm16 {
                    rd: value("defg", bit_values),
                    imm16: value("qrst_p_abc_hijklmno", bit_values),
                }),
                value(insn, bit_values)
            );
        }
    }

    #[test]
    fn movw_rd_imm16() {
        let insn = "0_abc_defg_hijklmno_11110_p_100100_qrst";
        for bit_values in test_cases(insn) {
            assert_eq!(
                encode(MovwRdImm16 {
                    rd: value("defg", bit_values),
                    imm16: value("qrst_p_abc_hijklmno", bit_values),
                }),
                value(insn, bit_values)
            );
        }
    }

    #[test]
    fn mul_rdm_rn() {
        let insn = "0100001101_abc_def";
        for bit_values in test_cases(insn) {
            assert_eq!(
                encode(MulRdmRn {
                    rdm: value("def", bit_values),
                    rn: value("abc", bit_values),
                }),
                value(insn, bit_values)
            );
        }
    }

    #[test]
    fn orr_rdn_rm() {
        let insn = "0100001100_abc_def";
        for bit_values in test_cases(insn) {
            assert_eq!(
                encode(OrrRdnRm {
                    rdn: value("def", bit_values),
                    rm: value("abc", bit_values),
                }),
                value(insn, bit_values)
            );
        }
    }

    #[test]
    fn pop() {
        let insn = "1011110_a_bcdefghi";
        for bit_values in test_cases(insn) {
            assert_eq!(
                encode(Pop {
                    registers: value("bcdefghi", bit_values),
                    pc: value("a", bit_values) == 1,
                }),
                value(insn, bit_values)
            );
        }
    }

    #[test]
    fn push() {
        let insn = "1011010_a_bcdefghi";
        for bit_values in test_cases(insn) {
            assert_eq!(
                encode(Push {
                    registers: value("bcdefghi", bit_values),
                    lr: value("a", bit_values) == 1,
                }),
                value(insn, bit_values)
            );
        }
    }

    #[test]
    fn strb_rt_rn_imm5() {
        let insn = "01110_abcde_fgh_ijk";
        for bit_values in test_cases(insn) {
            assert_eq!(
                encode(StrbRtRnImm5 {
                    rt: value("ijk", bit_values),
                    rn: value("fgh", bit_values),
                    imm5: value("abcde", bit_values),
                }),
                value(insn, bit_values)
            );
        }
    }

    #[test]
    fn strh_rt_rn_imm5() {
        let insn = "10000_abcde_fgh_ijk";
        for bit_values in test_cases(insn) {
            assert_eq!(
                encode(StrhRtRnImm5 {
                    rt: value("ijk", bit_values),
                    rn: value("fgh", bit_values),
                    imm: value("abcde_0", bit_values),
                }),
                value(insn, bit_values)
            );
        }
    }

    #[test]
    fn str_rt_rn_imm5() {
        let insn = "01100_abcde_fgh_ijk";
        for bit_values in test_cases(insn) {
            assert_eq!(
                encode(StrRtRnImm5 {
                    rt: value("ijk", bit_values),
                    rn: value("fgh", bit_values),
                    imm: value("abcde_00", bit_values),
                }),
                value(insn, bit_values)
            );
        }
    }

    #[test]
    fn str_rt_sp_imm8() {
        let insn = "10010_abc_efghijkl";
        for bit_values in test_cases(insn) {
            assert_eq!(
                encode(StrRtSpImm8 {
                    rt: value("abc", bit_values),
                    imm: value("efghijkl_00", bit_values),
                }),
                value(insn, bit_values)
            );
        }
    }

    #[test]
    fn sub_rdn_imm8() {
        let insn = "00111_abc_defghijk";
        for bit_values in test_cases(insn) {
            assert_eq!(
                encode(SubRdnImm8 {
                    rdn: value("abc", bit_values),
                    imm8: value("defghijk", bit_values)
                }),
                value(insn, bit_values)
            );
        }
    }

    #[test]
    fn sub_rd_rn_rm() {
        let insn = "0001101_abc_def_ghi";
        for bit_values in test_cases(insn) {
            assert_eq!(
                encode(SubRdRnRm {
                    rd: value("ghi", bit_values),
                    rn: value("def", bit_values),
                    rm: value("abc", bit_values)
                }),
                value(insn, bit_values)
            );
        }
    }

    #[test]
    fn sub_sp_sp_imm7() {
        let insn = "101100001_abcdefg";
        for bit_values in test_cases(insn) {
            assert_eq!(
                encode(SubSpSpImm7 {
                    imm: value("abcdefg_00", bit_values)
                }),
                value(insn, bit_values)
            );
        }
    }

    #[test]
    fn tbb_rn_rm() {
        let insn = "111100000000_abcd_111010001101_efgh";
        for bit_values in test_cases(insn) {
            assert_eq!(
                encode(TbbRnRm {
                    rn: value("efgh", bit_values),
                    rm: value("abcd", bit_values),
                }),
                value(insn, bit_values)
            );
        }
    }

    #[test]
    fn udiv_rd_rn_rm() {
        let insn = "1111_abcd_1111_efgh_111110111011_ijkl";
        for bit_values in test_cases(insn) {
            assert_eq!(
                encode(UdivRdRnRm {
                    rd: value("abcd", bit_values),
                    rn: value("ijkl", bit_values),
                    rm: value("efgh", bit_values),
                }),
                value(insn, bit_values)
            );
        }
    }

    #[test]
    fn u32_data() {
        for i in 0..32 {
            assert_eq!(
                encode(U32Data {
                    data: 1 << i,
                    comment: String::from("")
                }),
                1 << i
            );
        }
    }
}
