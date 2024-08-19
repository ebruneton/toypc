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

/// An Armv7-M Thumb instruction. Only a small subset of all the existing instructions is supported.
/// See section A7.7, pA7-186 of the Arm v7-M Architecture Reference Manual.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Instruction {
    /// ADD (immediate), section A7.7.3, encoding T2.
    AddRdnImm8 {
        rdn: u8,
        imm8: u8,
    },
    /// ADD (register), section A7.7.4, encoding T1.
    AddRdRnRm {
        rd: u8,
        rn: u8,
        rm: u8,
    },
    /// ADD (SP plus immediate), section A7.7.5, encoding T1.
    AddRdSpImm8 {
        rd: u8,
        imm: u16,
    },
    /// ADD (SP plus immediate), section A7.7.5, encoding T2.
    AddSpSpImm7 {
        imm: u16,
    },
    /// ADR, section A7.7.7, encoding T2.
    AdrRdMinusImm12 {
        rd: u8,
        imm12: u16,
    },
    /// AND (register), section A7.7.9, encoding T1.
    AndRdnRm {
        rdn: u8,
        rm: u8,
    },
    /// B, section A7.7.12, encoding T2.
    BImm11 {
        imm: i16,
    },
    /// BL, section A7.7.18, encoding T1 (positive offset case).
    BlPlusImm24 {
        imm16: u16,
        imm8: u8,
    },
    /// BL, section A7.7.18, encoding T1 (negative offset case).
    BlMinusImm24 {
        imm16: u16,
        imm8: u8,
    },
    /// BLX (register), section A7.7.19, encoding T1.
    BlxRm {
        rm: u8,
    },
    /// BX, section A7.7.20, encoding T1.
    BxRm {
        rm: u8,
    },
    /// CBZ, section A7.7.21, encoding T1.
    CbzRnImm6 {
        rn: u8,
        imm: u8,
    },
    /// CMP (immediate), section A7.7.27, encoding T1.
    CmpRnImm8 {
        rn: u8,
        imm8: u8,
    },
    /// CMP (register), section A7.7.28, encoding T1.
    CmpRnRm {
        rn: u8,
        rm: u8,
    },
    /// IT, section A7.7.38, enconding T1.
    It {
        first_cond_and_mask: u8,
    },
    /// LDRB (immediate), section A7.7.46, encoding T1.
    LdrbRtRnImm5 {
        rt: u8,
        rn: u8,
        imm5: u8,
    },
    /// LDRH (immediate), section A7.7.55, encoding T1.
    LdrhRtRnImm5 {
        rt: u8,
        rn: u8,
        imm: u8,
    },
    /// LDR (literal), section A7.7.44, encoding T1.
    LdrRtPcImm8 {
        rt: u8,
        imm: u16,
    },
    /// LDR (immediate), section A7.7.43, encoding T1.
    LdrRtRnImm5 {
        rt: u8,
        rn: u8,
        imm: u8,
    },
    /// LDR (immediate), section A7.7.43, encoding T2.
    LdrRtSpImm8 {
        rt: u8,
        imm: u16,
    },
    /// LSL (register), section A7.7.69, encoding T1.
    LslRdnRm {
        rdn: u8,
        rm: u8,
    },
    /// LSL (immediate), section A7.7.68, encoding T1.
    LslRdRmImm5 {
        rd: u8,
        rm: u8,
        imm5: u8,
    },
    /// LSR (register), section A7.7.71, encoding T1.
    LsrRdnRm {
        rdn: u8,
        rm: u8,
    },
    /// LSR (immediate), section A7.7.70, encoding T1.
    LsrRdRmImm5 {
        rd: u8,
        rm: u8,
        imm: u8,
    },
    /// MOV (immediate), section A7.7.76, encoding T1.
    MovRdImm8 {
        rd: u8,
        imm8: u8,
    },
    /// MOV (register), section A7.7.77, encoding T1.
    MovRdRm {
        rd: u8,
        rm: u8,
    },
    /// MOVT, section A7.7.79, encoding T1.
    MovtRdImm16 {
        rd: u8,
        imm16: u16,
    },
    /// MOV (immediate), section A7.7.76, encoding T3.
    MovwRdImm16 {
        rd: u8,
        imm16: u16,
    },
    // MRS, section B5.2.2, encoding T1.
    MrsRdReg {
        rd: u8,
        reg: u8,
    },
    // MSR, section B5.2.3, encoding T1.
    MsrRegRn {
        rn: u8,
        reg: u8,
    },
    /// MUL, section A7.7.84, encoding T1.
    MulRdmRn {
        rdm: u8,
        rn: u8,
    },
    /// ORR (register), section A7.7.92, encoding T1.
    OrrRdnRm {
        rdn: u8,
        rm: u8,
    },
    /// POP, section A7.7.99, encoding T1.
    Pop {
        registers: u8,
        pc: bool,
    },
    /// PUSH, section A7.7.101, encoding T1.
    Push {
        registers: u8,
        lr: bool,
    },
    /// STRB (immediate) , section A7.7.163, encoding T1.
    StrbRtRnImm5 {
        rt: u8,
        rn: u8,
        imm5: u8,
    },
    /// STRH (immediate), section A7.7.170, encoding T1.
    StrhRtRnImm5 {
        rt: u8,
        rn: u8,
        imm: u8,
    },
    /// STR (immediate), section A7.7.161, encoding T1.
    StrRtRnImm5 {
        rt: u8,
        rn: u8,
        imm: u8,
    },
    /// STR (immediate), section A7.7.161, encoding T2.
    StrRtSpImm8 {
        rt: u8,
        imm: u16,
    },
    /// SUB (immediate), section A7.7.174, encoding T2.
    SubRdnImm8 {
        rdn: u8,
        imm8: u8,
    },
    /// SUB (register), section A7.7.175, encoding T1.
    SubRdRnRm {
        rd: u8,
        rn: u8,
        rm: u8,
    },
    /// SUB (SP minus immediate), section A7.7.176, encoding T1.
    SubSpSpImm7 {
        imm: u16,
    },
    /// SVC, section A7.7.178, encoding T1.
    SvcImm8 {
        imm8: u8,
    },
    /// TBB, section A7.7.185, encoding T1.
    TbbRnRm {
        rn: u8,
        rm: u8,
    },
    /// UDIV, section A7.7.195, encoding T1.
    UdivRdRnRm {
        rd: u8,
        rn: u8,
        rm: u8,
    },
    Unknown,
    Unsupported,
    Invalid,
}

macro_rules! bits32 {
    ( $value:expr, $first:literal, $last:literal ) => {
        ($value >> $first) & ((1 << ($last - $first + 1)) - 1)
    };
}

macro_rules! bits16 {
    ( $value:expr, $first:literal , $last:literal ) => {
        bits32!($value, $first, $last) as u16
    };
}

macro_rules! bits8 {
    ( $value:expr, $first:literal , $last:literal ) => {
        bits32!($value, $first, $last) as u8
    };
}

macro_rules! sign_extend16 {
    ( $value:expr, $num_bits:literal ) => {
        (($value << (16 - $num_bits)) as i16) >> (16 - $num_bits)
    };
}

#[inline(always)]
pub fn is32bit_insn(insn: u32) -> bool {
    // See section A5.1, "Thumb instruction set encoding".
    bits8!(insn, 11, 15) > 0b11100
}

pub fn decode_insn(insn: u32) -> Instruction {
    use Instruction::*;
    const STACK_POINTER: u8 = 13;
    const PROGRAM_COUNTER: u8 = 15;
    const LAST_PROGRAM_STATUS_SPECIAL_REGISTER: u8 = 7;
    match bits8!(insn, 11, 15) {
        0b00000 => {
            let rd = bits8!(insn, 0, 2);
            let rm = bits8!(insn, 3, 5);
            let imm5 = bits8!(insn, 6, 10);
            if imm5 == 0 {
                Unsupported
            } else {
                LslRdRmImm5 { rd, rm, imm5 }
            }
        }
        0b00001 => {
            let rd = bits8!(insn, 0, 2);
            let rm = bits8!(insn, 3, 5);
            let imm = bits8!(insn, 6, 10);
            if imm == 0 {
                LsrRdRmImm5 { rd, rm, imm: 32 }
            } else {
                LsrRdRmImm5 { rd, rm, imm }
            }
        }
        0b00011 => {
            let rd = bits8!(insn, 0, 2);
            let rn = bits8!(insn, 3, 5);
            let rm = bits8!(insn, 6, 8);
            match bits8!(insn, 9, 10) {
                0b00 => AddRdRnRm { rd, rn, rm },
                0b01 => SubRdRnRm { rd, rn, rm },
                _ => Unsupported,
            }
        }
        0b00100 => {
            let imm8 = bits8!(insn, 0, 7);
            let rd = bits8!(insn, 8, 10);
            MovRdImm8 { rd, imm8 }
        }
        0b00101 => {
            let imm8 = bits8!(insn, 0, 7);
            let rn = bits8!(insn, 8, 10);
            CmpRnImm8 { rn, imm8 }
        }
        0b00110 => {
            let imm8 = bits8!(insn, 0, 7);
            let rdn = bits8!(insn, 8, 10);
            AddRdnImm8 { rdn, imm8 }
        }
        0b00111 => {
            let imm8 = bits8!(insn, 0, 7);
            let rdn = bits8!(insn, 8, 10);
            SubRdnImm8 { rdn, imm8 }
        }
        0b01000 => match bits16!(insn, 6, 10) {
            0b00000 => {
                let rdn = bits8!(insn, 0, 2);
                let rm = bits8!(insn, 3, 5);
                AndRdnRm { rdn, rm }
            }
            0b00010 => {
                let rdn = bits8!(insn, 0, 2);
                let rm = bits8!(insn, 3, 5);
                LslRdnRm { rdn, rm }
            }
            0b00011 => {
                let rdn = bits8!(insn, 0, 2);
                let rm = bits8!(insn, 3, 5);
                LsrRdnRm { rdn, rm }
            }
            0b01010 => {
                let rn = bits8!(insn, 0, 2);
                let rm = bits8!(insn, 3, 5);
                CmpRnRm { rn, rm }
            }
            0b01100 => {
                let rdn = bits8!(insn, 0, 2);
                let rm = bits8!(insn, 3, 5);
                OrrRdnRm { rdn, rm }
            }
            0b01101 => {
                let rdm = bits8!(insn, 0, 2);
                let rn = bits8!(insn, 3, 5);
                MulRdmRn { rdm, rn }
            }
            0b11000..=0b11011 => {
                let rd = bits8!(insn, 7, 7) << 3 | bits8!(insn, 0, 2);
                let rm = bits8!(insn, 3, 6);
                MovRdRm { rd, rm }
            }
            0b11100..=0b11101 => {
                let rm = bits8!(insn, 3, 6);
                if bits8!(insn, 0, 2) == 0 && rm != PROGRAM_COUNTER {
                    BxRm { rm }
                } else {
                    Unsupported
                }
            }
            0b11110..=0b11111 => {
                let rm = bits8!(insn, 3, 6);
                if bits8!(insn, 0, 2) == 0 {
                    if rm == PROGRAM_COUNTER {
                        Invalid
                    } else {
                        BlxRm { rm }
                    }
                } else {
                    Unsupported
                }
            }
            _ => Unsupported,
        },
        0b01001 => {
            let imm = bits16!(insn, 0, 7) << 2;
            let rt = bits8!(insn, 8, 10);
            LdrRtPcImm8 { rt, imm }
        }
        0b01100 => {
            let rt = bits8!(insn, 0, 2);
            let rn = bits8!(insn, 3, 5);
            let imm = bits8!(insn, 6, 10) << 2;
            StrRtRnImm5 { rt, rn, imm }
        }
        0b01101 => {
            let rt = bits8!(insn, 0, 2);
            let rn = bits8!(insn, 3, 5);
            let imm = bits8!(insn, 6, 10) << 2;
            LdrRtRnImm5 { rt, rn, imm }
        }
        0b01110 => {
            let rt = bits8!(insn, 0, 2);
            let rn = bits8!(insn, 3, 5);
            let imm5 = bits8!(insn, 6, 10);
            StrbRtRnImm5 { rt, rn, imm5 }
        }
        0b01111 => {
            let rt = bits8!(insn, 0, 2);
            let rn = bits8!(insn, 3, 5);
            let imm5 = bits8!(insn, 6, 10);
            LdrbRtRnImm5 { rt, rn, imm5 }
        }
        0b10000 => {
            let rt = bits8!(insn, 0, 2);
            let rn = bits8!(insn, 3, 5);
            let imm = bits8!(insn, 6, 10) << 1;
            StrhRtRnImm5 { rt, rn, imm }
        }
        0b10001 => {
            let rt = bits8!(insn, 0, 2);
            let rn = bits8!(insn, 3, 5);
            let imm = bits8!(insn, 6, 10) << 1;
            LdrhRtRnImm5 { rt, rn, imm }
        }
        0b10010 => {
            let imm = bits16!(insn, 0, 7) << 2;
            let rt = bits8!(insn, 8, 10);
            StrRtSpImm8 { rt, imm }
        }
        0b10011 => {
            let imm = bits16!(insn, 0, 7) << 2;
            let rt = bits8!(insn, 8, 10);
            LdrRtSpImm8 { rt, imm }
        }
        0b10101 => {
            let imm = bits16!(insn, 0, 7) << 2;
            let rd = bits8!(insn, 8, 10);
            AddRdSpImm8 { rd, imm }
        }
        0b10110 => match bits16!(insn, 7, 10) {
            0b0000 => {
                let imm = bits16!(insn, 0, 6) << 2;
                AddSpSpImm7 { imm }
            }
            0b0001 => {
                let imm = bits16!(insn, 0, 6) << 2;
                SubSpSpImm7 { imm }
            }
            0b0010..=0b0011 => {
                let rn = bits8!(insn, 0, 2);
                let imm = bits8!(insn, 3, 7) << 1;
                CbzRnImm6 { rn, imm }
            }
            0b0110..=0b0111 => {
                let rn = bits8!(insn, 0, 2);
                let imm = (bits8!(insn, 3, 7) | (1 << 5)) << 1;
                CbzRnImm6 { rn, imm }
            }
            0b1000..=0b1011 => {
                let registers = bits8!(insn, 0, 7);
                let lr = bits8!(insn, 8, 8) != 0;
                Push { registers, lr }
            }
            _ => Unsupported,
        },
        0b10111 => match bits16!(insn, 8, 10) {
            0b100..=0b101 => {
                let registers = bits8!(insn, 0, 7);
                let pc = bits8!(insn, 8, 8) != 0;
                Pop { registers, pc }
            }
            0b111 => {
                let first_cond_and_mask = bits8!(insn, 0, 7);
                It {
                    first_cond_and_mask,
                }
            }
            _ => Unsupported,
        },
        0b11011 => match bits16!(insn, 8, 10) {
            0b111 => {
                let imm8 = bits8!(insn, 0, 7);
                SvcImm8 { imm8 }
            }
            _ => Unsupported,
        },
        0b11100 => {
            let imm = sign_extend16!(bits16!(insn, 0, 10) << 1, 12);
            BImm11 { imm }
        }
        0b11101 => match bits16!(insn, 4, 10) {
            0b0001101 => {
                if bits16!(insn, 20, 31) == 0b111100000000 {
                    let rn = bits8!(insn, 0, 3);
                    let rm = bits8!(insn, 16, 19);
                    if rn == STACK_POINTER || rm == STACK_POINTER || rm == PROGRAM_COUNTER {
                        Invalid
                    } else {
                        TbbRnRm { rn, rm }
                    }
                } else {
                    Unsupported
                }
            }
            _ => Unsupported,
        },
        0b11110 => {
            if bits8!(insn, 31, 31) == 0 {
                let imm4 = bits16!(insn, 0, 3);
                let i = bits16!(insn, 10, 10);
                let imm8 = bits16!(insn, 16, 23);
                let rd = bits8!(insn, 24, 27);
                let imm3 = bits16!(insn, 28, 30);
                if rd == STACK_POINTER || rd == PROGRAM_COUNTER {
                    Invalid
                } else {
                    match insn & 0b1111101111111111 {
                        0b1111001001000000..=0b1111001001001111 => {
                            let imm16 = (imm4 << 12) | (i << 11) | (imm3 << 8) | imm8;
                            MovwRdImm16 { rd, imm16 }
                        }
                        0b1111001010101111 => {
                            let imm12 = (i << 11) | (imm3 << 8) | imm8;
                            AdrRdMinusImm12 { rd, imm12 }
                        }
                        0b1111001011000000..=0b1111001011001111 => {
                            let imm16 = (imm4 << 12) | (i << 11) | (imm3 << 8) | imm8;
                            MovtRdImm16 { rd, imm16 }
                        }
                        _ => Unsupported,
                    }
                }
            } else {
                let flags = bits32!(insn, 27, 30);
                if flags & 0b1010 == 0b1010 {
                    let imm10 = bits32!(insn, 0, 9);
                    let s = bits32!(insn, 10, 10);
                    let imm11 = bits32!(insn, 16, 26);
                    let j2 = bits32!(insn, 27, 27);
                    let j1 = bits32!(insn, 29, 29);
                    let i1 = (j1 ^ s) ^ 1;
                    let i2 = (j2 ^ s) ^ 1;
                    let mut imm24 = ((i1 << 22) | (i2 << 21) | (imm10 << 11) | imm11) << 1;
                    if s == 0 {
                        let imm16 = imm24 as u16;
                        let imm8 = (imm24 >> 16) as u8;
                        BlPlusImm24 { imm16, imm8 }
                    } else {
                        imm24 = 0xFFFFFF - imm24;
                        let imm16 = imm24 as u16;
                        let imm8 = (imm24 >> 16) as u8;
                        BlMinusImm24 { imm16, imm8 }
                    }
                } else if flags & 0b1110 == 0 {
                    let reg = bits8!(insn, 16, 23);
                    let rd = bits8!(insn, 24, 27);
                    if insn & 0b1111111111111111 == 0b1111001111101111 {
                        if rd == STACK_POINTER || rd == PROGRAM_COUNTER {
                            Invalid
                        } else if reg <= LAST_PROGRAM_STATUS_SPECIAL_REGISTER {
                            Unsupported
                        } else {
                            MrsRdReg { rd, reg }
                        }
                    } else if insn & 0b1111111111110000 == 0b1111001110000000 && rd == 8 {
                        let rn = bits8!(insn, 0, 3);
                        if rn == STACK_POINTER || rn == PROGRAM_COUNTER {
                            Invalid
                        } else if reg <= LAST_PROGRAM_STATUS_SPECIAL_REGISTER {
                            Unsupported
                        } else {
                            MsrRegRn { rn, reg }
                        }
                    } else {
                        Unsupported
                    }
                } else {
                    Unsupported
                }
            }
        }
        0b11111 => match bits16!(insn, 4, 10) {
            0b0111011 => {
                let rn = bits8!(insn, 0, 3);
                let rm = bits8!(insn, 16, 19);
                let rd = bits8!(insn, 24, 27);
                if bits8!(insn, 20, 23) == 0b1111 && bits8!(insn, 28, 31) == 0b1111 {
                    if rn == STACK_POINTER
                        || rn == PROGRAM_COUNTER
                        || rm == STACK_POINTER
                        || rm == PROGRAM_COUNTER
                        || rd == STACK_POINTER
                        || rm == PROGRAM_COUNTER
                    {
                        Invalid
                    } else {
                        UdivRdRnRm { rd, rn, rm }
                    }
                } else {
                    Unsupported
                }
            }
            _ => Unsupported,
        },

        _ => Unsupported,
    }
}

#[cfg(test)]
mod tests {
    use super::decode_insn;
    use super::is32bit_insn;
    use super::Instruction::*;

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
                _ => panic!("Unsupported bit pattern character '{}'", c),
            }
        }
        result
    }

    fn value16(pattern: &BitPattern, values: BitValues) -> u16 {
        value(pattern, values) as u16
    }

    fn value8(pattern: &BitPattern, values: BitValues) -> u8 {
        value(pattern, values) as u8
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
        result.push((1 << num_bits) - 1);
        result
    }

    #[test]
    #[should_panic(expected = "Unsupported bit pattern character ' '")]
    fn value_bad_pattern() {
        value("01AB CD", 0);
    }

    #[test]
    fn add_rdn_imm8() {
        let insn = "00110_abc_defghijk";
        for bit_values in test_cases(insn) {
            assert_eq!(
                decode_insn(value(insn, bit_values)),
                AddRdnImm8 {
                    rdn: value8("abc", bit_values),
                    imm8: value8("defghijk", bit_values)
                }
            );
        }
        assert!(!is32bit_insn(value(insn, 0)));
    }

    #[test]
    fn add_rd_rn_rm() {
        let insn = "0001100_abc_def_ghi";
        for bit_values in test_cases(insn) {
            assert_eq!(
                decode_insn(value(insn, bit_values)),
                AddRdRnRm {
                    rd: value8("ghi", bit_values),
                    rn: value8("def", bit_values),
                    rm: value8("abc", bit_values)
                }
            );
        }
        assert!(!is32bit_insn(value(insn, 0)));
    }

    #[test]
    fn add_rd_sp_imm8() {
        let insn = "10101_abc_defghijk";
        for bit_values in test_cases(insn) {
            assert_eq!(
                decode_insn(value(insn, bit_values)),
                AddRdSpImm8 {
                    rd: value8("abc", bit_values),
                    imm: value16("defghijk_00", bit_values)
                }
            );
        }
        assert!(!is32bit_insn(value(insn, 0)));
    }

    #[test]
    fn add_sp_sp_imm7() {
        let insn = "101100000_abcdefg";
        for bit_values in test_cases(insn) {
            assert_eq!(
                decode_insn(value(insn, bit_values)),
                AddSpSpImm7 {
                    imm: value16("abcdefg_00", bit_values)
                }
            );
        }
        assert!(!is32bit_insn(value(insn, 0)));
    }

    #[test]
    fn adr_rd_minus_imm12() {
        let insn = "0_abc_defg_hijklmno_11110z1010101111";
        for bit_values in test_cases(insn) {
            let rd = value8("defg", bit_values);
            assert_eq!(
                decode_insn(value(insn, bit_values)),
                if rd == 13 || rd == 15 {
                    Invalid
                } else {
                    AdrRdMinusImm12 {
                        rd,
                        imm12: value16("z_abc_hijklmno", bit_values),
                    }
                }
            );
        }
        assert!(is32bit_insn(value(insn, 0)));
    }

    #[test]
    fn and_rdn_rm() {
        let insn = "0100000000_abc_def";
        for bit_values in test_cases(insn) {
            assert_eq!(
                decode_insn(value(insn, bit_values)),
                AndRdnRm {
                    rdn: value8("def", bit_values),
                    rm: value8("abc", bit_values)
                }
            );
        }
        assert!(!is32bit_insn(value(insn, 0)));
    }

    #[test]
    fn b_imm11() {
        let insn = "11100_abcdefghijk";
        for bit_values in test_cases(insn) {
            assert_eq!(
                decode_insn(value(insn, bit_values)),
                BImm11 {
                    imm: sign_extend16!(value16("abcdefghijk_0", bit_values), 12),
                }
            );
        }
        assert!(!is32bit_insn(value(insn, 0)));
    }

    #[test]
    fn bl_plus_imm24() {
        let insn = "11_a_1_b_cdefghijklm_111100_nopqrstuvw";
        for bit_values in test_cases(insn) {
            let imm = value("A_B_nopqrstuvw_cdefghijklm_0", bit_values);
            assert_eq!(
                decode_insn(value(insn, bit_values)),
                BlPlusImm24 {
                    imm16: imm as u16,
                    imm8: (imm >> 16) as u8,
                }
            );
        }
        assert!(is32bit_insn(value(insn, 0)));
    }

    #[test]
    fn bl_minus_imm24() {
        let insn = "11_a_1_b_cdefghijklm_111101_nopqrstuvw";
        for bit_values in test_cases(insn) {
            let signed_imm = value("11111111_a_b_nopqrstuvw_cdefghijklm_0", bit_values) as i32;
            let imm = (-signed_imm - 1) as u32;
            assert_eq!(
                decode_insn(value(insn, bit_values)),
                BlMinusImm24 {
                    imm16: imm as u16,
                    imm8: (imm >> 16) as u8,
                }
            );
        }
        assert!(is32bit_insn(value(insn, 0)));
    }

    #[test]
    fn blx_rm() {
        let insn = "010001111_abcd_000";
        for bit_values in test_cases(insn) {
            let rm = value8("abcd", bit_values);
            assert_eq!(
                decode_insn(value(insn, bit_values)),
                if rm == 15 { Invalid } else { BlxRm { rm } }
            );
        }
        let invalid_insn = "0100011110000abc";
        for bit_values in test_cases(invalid_insn) {
            assert_eq!(decode_insn(value(invalid_insn, bit_values)), Unsupported);
        }
        assert!(!is32bit_insn(value(insn, 0)));
    }

    #[test]
    fn bx_rm() {
        let insn = "010001110_abcd_000";
        for bit_values in test_cases(insn) {
            let rm = value8("abcd", bit_values);
            assert_eq!(
                decode_insn(value(insn, bit_values)),
                if rm == 15 { Unsupported } else { BxRm { rm } }
            );
        }
        let invalid_insn = "0100011100000abc";
        for bit_values in test_cases(invalid_insn) {
            assert_eq!(decode_insn(value(invalid_insn, bit_values)), Unsupported);
        }
        assert!(!is32bit_insn(value(insn, 0)));
    }

    #[test]
    fn cbz_rm_imm6() {
        let insn = "101100_i_1_abcde_fgh";
        for bit_values in test_cases(insn) {
            assert_eq!(
                decode_insn(value(insn, bit_values)),
                CbzRnImm6 {
                    rn: value8("fgh", bit_values),
                    imm: value8("i_abcde_0", bit_values),
                }
            );
        }
        assert!(!is32bit_insn(value(insn, 0)));
    }

    #[test]
    fn cmp_rn_imm8() {
        let insn = "00101_abc_defghijk";
        for bit_values in test_cases(insn) {
            assert_eq!(
                decode_insn(value(insn, bit_values)),
                CmpRnImm8 {
                    rn: value8("abc", bit_values),
                    imm8: value8("defghijk", bit_values),
                }
            );
        }
        assert!(!is32bit_insn(value(insn, 0)));
    }

    #[test]
    fn cmp_rn_rm() {
        let insn = "0100001010_abc_def";
        for bit_values in test_cases(insn) {
            assert_eq!(
                decode_insn(value(insn, bit_values)),
                CmpRnRm {
                    rn: value8("def", bit_values),
                    rm: value8("abc", bit_values),
                }
            );
        }
        assert!(!is32bit_insn(value(insn, 0)));
    }

    #[test]
    fn it() {
        let insn = "10111111_abcd_efgh";
        for bit_values in test_cases(insn) {
            assert_eq!(
                decode_insn(value(insn, bit_values)),
                It {
                    first_cond_and_mask: value8("abcd_efgh", bit_values),
                }
            );
        }
        assert!(!is32bit_insn(value(insn, 0)));
    }

    #[test]
    fn ldrb_rt_rn_imm5() {
        let insn = "01111_abcde_fgh_ijk";
        for bit_values in test_cases(insn) {
            assert_eq!(
                decode_insn(value(insn, bit_values)),
                LdrbRtRnImm5 {
                    rt: value8("ijk", bit_values),
                    rn: value8("fgh", bit_values),
                    imm5: value8("abcde", bit_values),
                }
            );
        }
        assert!(!is32bit_insn(value(insn, 0)));
    }

    #[test]
    fn ldrh_rt_rn_imm5() {
        let insn = "10001_abcde_fgh_ijk";
        for bit_values in test_cases(insn) {
            assert_eq!(
                decode_insn(value(insn, bit_values)),
                LdrhRtRnImm5 {
                    rt: value8("ijk", bit_values),
                    rn: value8("fgh", bit_values),
                    imm: value8("abcde_0", bit_values),
                }
            );
        }
        assert!(!is32bit_insn(value(insn, 0)));
    }

    #[test]
    fn ldr_rt_pc_imm8() {
        let insn = "01001_abc_defghijk";
        for bit_values in test_cases(insn) {
            assert_eq!(
                decode_insn(value(insn, bit_values)),
                LdrRtPcImm8 {
                    rt: value8("abc", bit_values),
                    imm: value16("defghijk_00", bit_values),
                }
            );
        }
        assert!(!is32bit_insn(value(insn, 0)));
    }

    #[test]
    fn ldr_rt_rn_imm5() {
        let insn = "01101_abcde_fgh_ijk";
        for bit_values in test_cases(insn) {
            assert_eq!(
                decode_insn(value(insn, bit_values)),
                LdrRtRnImm5 {
                    rt: value8("ijk", bit_values),
                    rn: value8("fgh", bit_values),
                    imm: value8("abcde_00", bit_values),
                }
            );
        }
        assert!(!is32bit_insn(value(insn, 0)));
    }

    #[test]
    fn ldr_rt_sp_imm8() {
        let insn = "10011_abc_efghijkl";
        for bit_values in test_cases(insn) {
            assert_eq!(
                decode_insn(value(insn, bit_values)),
                LdrRtSpImm8 {
                    rt: value8("abc", bit_values),
                    imm: value16("efghijkl_00", bit_values),
                }
            );
        }
        assert!(!is32bit_insn(value(insn, 0)));
    }

    #[test]
    fn lsl_rdn_rm() {
        let insn = "0100000010_abc_def";
        for bit_values in test_cases(insn) {
            assert_eq!(
                decode_insn(value(insn, bit_values)),
                LslRdnRm {
                    rdn: value8("def", bit_values),
                    rm: value8("abc", bit_values),
                }
            );
        }
        assert!(!is32bit_insn(value(insn, 0)));
    }

    #[test]
    fn lsl_rd_rm_imm5() {
        let insn = "00000_abcde_fgh_ijk";
        for bit_values in test_cases(insn) {
            if value8("abcde", bit_values) == 0 {
                assert_eq!(decode_insn(value(insn, bit_values)), Unsupported);
            } else {
                assert_eq!(
                    decode_insn(value(insn, bit_values)),
                    LslRdRmImm5 {
                        rd: value8("ijk", bit_values),
                        rm: value8("fgh", bit_values),
                        imm5: value8("abcde", bit_values),
                    }
                );
            }
        }
        assert!(!is32bit_insn(value(insn, 0)));
    }

    #[test]
    fn lsr_rdn_rm() {
        let insn = "0100000011_abc_def";
        for bit_values in test_cases(insn) {
            assert_eq!(
                decode_insn(value(insn, bit_values)),
                LsrRdnRm {
                    rdn: value8("def", bit_values),
                    rm: value8("abc", bit_values),
                }
            );
        }
        assert!(!is32bit_insn(value(insn, 0)));
    }

    #[test]
    fn lsr_rd_rm_imm5() {
        let insn = "00001_abcde_fgh_ijk";
        for bit_values in test_cases(insn) {
            let imm5 = value8("abcde", bit_values);
            assert_eq!(
                decode_insn(value(insn, bit_values)),
                LsrRdRmImm5 {
                    rd: value8("ijk", bit_values),
                    rm: value8("fgh", bit_values),
                    imm: if imm5 == 0 { 32 } else { imm5 },
                }
            );
        }
        assert!(!is32bit_insn(value(insn, 0)));
    }

    #[test]
    fn mov_rd_imm8() {
        let insn = "00100_abc_defghijk";
        for bit_values in test_cases(insn) {
            assert_eq!(
                decode_insn(value(insn, bit_values)),
                MovRdImm8 {
                    rd: value8("abc", bit_values),
                    imm8: value8("defghijk", bit_values),
                }
            );
        }
        assert!(!is32bit_insn(value(insn, 0)));
    }

    #[test]
    fn mov_rd_rm() {
        let insn = "01000110_a_bcde_fgh";
        for bit_values in test_cases(insn) {
            assert_eq!(
                decode_insn(value(insn, bit_values)),
                MovRdRm {
                    rd: value8("a_fgh", bit_values),
                    rm: value8("bcde", bit_values),
                }
            );
        }
        assert!(!is32bit_insn(value(insn, 0)));
    }

    #[test]
    fn movt_rd_imm16() {
        let insn = "0_abc_defg_hijklmno_11110_p_101100_qrst";
        for bit_values in test_cases(insn) {
            let rd = value8("defg", bit_values);
            assert_eq!(
                decode_insn(value(insn, bit_values)),
                if rd == 13 || rd == 15 {
                    Invalid
                } else {
                    MovtRdImm16 {
                        rd,
                        imm16: value16("qrst_p_abc_hijklmno", bit_values),
                    }
                }
            );
        }
        assert!(is32bit_insn(value(insn, 0)));
    }

    #[test]
    fn movw_rd_imm16() {
        let insn = "0_abc_defg_hijklmno_11110_p_100100_qrst";
        for bit_values in test_cases(insn) {
            let rd = value8("defg", bit_values);
            assert_eq!(
                decode_insn(value(insn, bit_values)),
                if rd == 13 || rd == 15 {
                    Invalid
                } else {
                    MovwRdImm16 {
                        rd,
                        imm16: value16("qrst_p_abc_hijklmno", bit_values),
                    }
                }
            );
        }
        assert!(is32bit_insn(value(insn, 0)));
    }

    #[test]
    fn mrs_rd_reg() {
        let insn = "1000_abcd_efghijkl_1111001111101111";
        for bit_values in test_cases(insn) {
            let rd = value8("abcd", bit_values);
            let reg = value8("efghijkl", bit_values);
            assert_eq!(
                decode_insn(value(insn, bit_values)),
                if rd == 13 || rd == 15 {
                    Invalid
                } else if reg <= 7 {
                    Unsupported
                } else {
                    MrsRdReg { rd, reg }
                }
            );
        }
        assert!(is32bit_insn(value(insn, 0)));
    }

    #[test]
    fn msr_reg_rn() {
        let insn = "10001000_abcdefgh_111100111000_ijkl";
        for bit_values in test_cases(insn) {
            let rn = value8("ijkl", bit_values);
            let reg = value8("abcdefgh", bit_values);
            assert_eq!(
                decode_insn(value(insn, bit_values)),
                if rn == 13 || rn == 15 {
                    Invalid
                } else if reg <= 7 {
                    Unsupported
                } else {
                    MsrRegRn { rn, reg }
                }
            );
        }
        assert!(is32bit_insn(value(insn, 0)));
    }

    #[test]
    fn mul_rdm_rn() {
        let insn = "0100001101_abc_def";
        for bit_values in test_cases(insn) {
            assert_eq!(
                decode_insn(value(insn, bit_values)),
                MulRdmRn {
                    rdm: value8("def", bit_values),
                    rn: value8("abc", bit_values),
                }
            );
        }
        assert!(!is32bit_insn(value(insn, 0)));
    }

    #[test]
    fn orr_rdn_rm() {
        let insn = "0100001100_abc_def";
        for bit_values in test_cases(insn) {
            assert_eq!(
                decode_insn(value(insn, bit_values)),
                OrrRdnRm {
                    rdn: value8("def", bit_values),
                    rm: value8("abc", bit_values),
                }
            );
        }
        assert!(!is32bit_insn(value(insn, 0)));
    }

    #[test]
    fn pop() {
        let insn = "1011110_a_bcdefghi";
        for bit_values in test_cases(insn) {
            assert_eq!(
                decode_insn(value(insn, bit_values)),
                Pop {
                    registers: value8("bcdefghi", bit_values),
                    pc: value8("a", bit_values) == 1,
                }
            );
        }
        assert!(!is32bit_insn(value(insn, 0)));
    }

    #[test]
    fn push() {
        let insn = "1011010_a_bcdefghi";
        for bit_values in test_cases(insn) {
            assert_eq!(
                decode_insn(value(insn, bit_values)),
                Push {
                    registers: value8("bcdefghi", bit_values),
                    lr: value8("a", bit_values) == 1,
                }
            );
        }
        assert!(!is32bit_insn(value(insn, 0)));
    }

    #[test]
    fn strb_rt_rn_imm5() {
        let insn = "01110_abcde_fgh_ijk";
        for bit_values in test_cases(insn) {
            assert_eq!(
                decode_insn(value(insn, bit_values)),
                StrbRtRnImm5 {
                    rt: value8("ijk", bit_values),
                    rn: value8("fgh", bit_values),
                    imm5: value8("abcde", bit_values),
                }
            );
        }
        assert!(!is32bit_insn(value(insn, 0)));
    }

    #[test]
    fn strh_rt_rn_imm5() {
        let insn = "10000_abcde_fgh_ijk";
        for bit_values in test_cases(insn) {
            assert_eq!(
                decode_insn(value(insn, bit_values)),
                StrhRtRnImm5 {
                    rt: value8("ijk", bit_values),
                    rn: value8("fgh", bit_values),
                    imm: value8("abcde_0", bit_values),
                }
            );
        }
        assert!(!is32bit_insn(value(insn, 0)));
    }

    #[test]
    fn str_rt_rn_imm5() {
        let insn = "01100_abcde_fgh_ijk";
        for bit_values in test_cases(insn) {
            assert_eq!(
                decode_insn(value(insn, bit_values)),
                StrRtRnImm5 {
                    rt: value8("ijk", bit_values),
                    rn: value8("fgh", bit_values),
                    imm: value8("abcde_00", bit_values),
                }
            );
        }
        assert!(!is32bit_insn(value(insn, 0)));
    }

    #[test]
    fn str_rt_sp_imm8() {
        let insn = "10010_abc_efghijkl";
        for bit_values in test_cases(insn) {
            assert_eq!(
                decode_insn(value(insn, bit_values)),
                StrRtSpImm8 {
                    rt: value8("abc", bit_values),
                    imm: value16("efghijkl_00", bit_values),
                }
            );
        }
        assert!(!is32bit_insn(value(insn, 0)));
    }

    #[test]
    fn sub_rdn_imm8() {
        let insn = "00111_abc_defghijk";
        for bit_values in test_cases(insn) {
            assert_eq!(
                decode_insn(value(insn, bit_values)),
                SubRdnImm8 {
                    rdn: value8("abc", bit_values),
                    imm8: value8("defghijk", bit_values)
                }
            );
        }
        assert!(!is32bit_insn(value(insn, 0)));
    }

    #[test]
    fn sub_rd_rn_rm() {
        let insn = "0001101_abc_def_ghi";
        for bit_values in test_cases(insn) {
            assert_eq!(
                decode_insn(value(insn, bit_values)),
                SubRdRnRm {
                    rd: value8("ghi", bit_values),
                    rn: value8("def", bit_values),
                    rm: value8("abc", bit_values)
                }
            );
        }
        assert!(!is32bit_insn(value(insn, 0)));
    }

    #[test]
    fn sub_sp_sp_imm7() {
        let insn = "101100001_abcdefg";
        for bit_values in test_cases(insn) {
            assert_eq!(
                decode_insn(value(insn, bit_values)),
                SubSpSpImm7 {
                    imm: value16("abcdefg_00", bit_values)
                }
            );
        }
        assert!(!is32bit_insn(value(insn, 0)));
    }

    #[test]
    fn svc_imm8() {
        let insn = "11011111_abcdefgh";
        for bit_values in test_cases(insn) {
            assert_eq!(
                decode_insn(value(insn, bit_values)),
                SvcImm8 {
                    imm8: value8("abcdefgh", bit_values)
                }
            );
        }
        assert!(!is32bit_insn(value(insn, 0)));
    }

    #[test]
    fn tbb_rn_rm() {
        let insn = "111100000000_abcd_111010001101_efgh";
        for bit_values in test_cases(insn) {
            let rn = value8("efgh", bit_values);
            let rm = value8("abcd", bit_values);
            assert_eq!(
                decode_insn(value(insn, bit_values)),
                if rn == 13 || rm == 13 || rm == 15 {
                    Invalid
                } else {
                    TbbRnRm { rn, rm }
                }
            );
        }
        let invalid_insn = "ABCDefghijkl_0000_111010001101_0000";
        for bit_values in test_cases(invalid_insn) {
            assert_eq!(decode_insn(value(invalid_insn, bit_values)), Unsupported);
        }
        assert!(is32bit_insn(value(insn, 0)));
    }

    #[test]
    fn udiv_rd_rn_rm() {
        let insn = "1111_abcd_1111_efgh_111110111011_ijkl";
        for bit_values in test_cases(insn) {
            let rd = value8("abcd", bit_values);
            let rn = value8("ijkl", bit_values);
            let rm = value8("efgh", bit_values);
            assert_eq!(
                decode_insn(value(insn, bit_values)),
                if rd == 13 || rd == 15 || rn == 13 || rn == 15 || rm == 13 || rm == 15 {
                    Invalid
                } else {
                    UdivRdRnRm { rd, rn, rm }
                }
            );
        }
        let invalid_insn = "ABCD_0000_1111_0000_111110111011_0000";
        for bit_values in test_cases("abcd_0000_1111_0000_111110111011_0000") {
            assert_eq!(decode_insn(value(invalid_insn, bit_values)), Unsupported);
        }
        assert!(is32bit_insn(value(insn, 0)));
    }

    #[test]
    #[allow(clippy::unusual_byte_groupings)]
    fn unsupported() {
        assert_eq!(decode_insn(0b00011_10_000000000), Unsupported);
        assert_eq!(decode_insn(0b01000_00001_000000), Unsupported);
        assert_eq!(decode_insn(0b10110_0100_0000000), Unsupported);
        assert_eq!(decode_insn(0b10111_000_00000000), Unsupported);
        assert_eq!(
            decode_insn(0b0000000000000000_11110_000_00000000),
            Unsupported
        );
        assert_eq!(
            decode_insn(0b1000000000000000_11110_000_00000000),
            Unsupported
        );
        assert_eq!(
            decode_insn(0b0000000000000000_11111_000_00000000),
            Unsupported
        );
    }
}
