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

#![allow(clippy::unusual_byte_groupings)]

use std::ops::Range;

use crate::arm::Condition;

pub struct Parameter {
    pub name: &'static str,
    pub range: Range<i32>,
    pub multiple: i32,
}

pub struct Field {
    pub name: &'static str,
    pub parameter: u8,
    pub bit_range: Range<u8>,
    pub offset: u8,
}

pub struct InstructionType<const P: usize, const F: usize> {
    pub name: &'static str,
    pub operation: &'static str,
    pub size: u8,
    pub opcode: u32,
    pub parameters: [Parameter; P],
    pub fields: [Field; F],
}

pub trait InstructionFormat {
    fn name(&self) -> &'static str;
    fn definition(&self) -> String;
    fn operation(&self) -> String;
    fn semantics(&self) -> String;
    fn bit_pattern(&self) -> String;
    fn encode(&self, arguments: &[u32]) -> u32;
    fn concrete_semantics(&self, encoding: u32, styled: bool) -> String;
    fn concrete_bit_pattern(&self, encoding: u32) -> String;
    fn concrete_top_bit_pattern(&self, encoding: u32) -> String;
}

const LATEX_STYLES: [&str; 3] = ["\\bm", "\\mathit", "\\mathtt"];

impl<const P: usize, const F: usize> InstructionType<P, F> {
    fn concrete_semantics(&self, arguments: [u32; P]) -> String {
        let mut result = String::new();
        if self.opcode == B_IMM11.opcode {
            result.push_str(&format!(
                r"$\mathrm{{PC}} \leftarrow \mathrm{{PC}} + 2 * {}{{{}}}",
                LATEX_STYLES[0], arguments[0]
            ));
            if arguments[0] < 1024 {
                result.push('$');
            } else {
                result.push_str(" - 4096$");
            }
        } else if self.opcode == BL_IMM22.opcode {
            result.push_str(&format!(
                r"$\mathrm{{PC}} \leftarrow \mathrm{{PC}} + 2 * {}{{{}}}",
                LATEX_STYLES[0], arguments[0]
            ));
            if arguments[0] < 2 * 1024 * 1024 {
                result.push('$');
            } else {
                result.push_str(r" - 8\ \mathrm{MB}$");
            }
        } else if self.opcode == IT.opcode {
            result.push_str("if ");
            let cond = arguments[0];
            if cond == Condition::EQ as u32 {
                result.push_str(&format!("${}{{=}}$", LATEX_STYLES[0]));
            } else if cond == Condition::NE as u32 {
                result.push_str(r"$\ne$");
            } else if cond == Condition::GE as u32 {
                result.push_str(r"$\ge$");
            } else if cond == Condition::LT as u32 {
                result.push_str(r"$<$");
            } else if cond == Condition::GT as u32 {
                result.push_str(r"$>$");
            } else if cond == Condition::LE as u32 {
                result.push_str(r"$\le$");
            }
            result.push_str(" then");
            let mut thenelse = arguments[1];
            while thenelse & 7 != 0 {
                if (thenelse >> 3) == cond & 1 {
                    result.push_str(&format!(" ${}{{then}}$", LATEX_STYLES[1]));
                } else {
                    result.push_str(&format!(" ${}{{else}}$", LATEX_STYLES[1]));
                }
                thenelse <<= 1;
            }
        } else if self.opcode == POP.opcode {
            let ones = arguments[0].count_ones();
            if ones > 2 && arguments[0] == (1 << ones) - 1 {
                result.push_str(&format!(
                    "$\\mathrm{{R}}{{{}0}}..\\mathrm{{R}}{{{}{}}}$ ",
                    LATEX_STYLES[0],
                    LATEX_STYLES[0],
                    ones - 1
                ));
            } else {
                for i in 0..8 {
                    if arguments[0] & (1 << i) != 0 {
                        result.push_str(&format!("$\\mathrm{{R}}{{{}{}}}$ ", LATEX_STYLES[0], i));
                    }
                }
            }
            if arguments[1] != 0 {
                result.push_str(&format!("${}{{PC}}$ ", LATEX_STYLES[1]));
            }
            result.push_str(r"$\leftarrow \mathrm{stack}$");
        } else if self.opcode == PUSH.opcode {
            let ones = arguments[0].count_ones();
            if ones > 2 && arguments[0] == (1 << ones) - 1 {
                result.push_str(&format!(
                    "$\\mathrm{{R}}{{{}0}}..\\mathrm{{R}}{{{}{}}}$ ",
                    LATEX_STYLES[0],
                    LATEX_STYLES[0],
                    ones - 1
                ));
            } else {
                for i in 0..8 {
                    if arguments[0] & (1 << i) != 0 {
                        result.push_str(&format!("$\\mathrm{{R}}{{{}{}}}$ ", LATEX_STYLES[0], i));
                    }
                }
            }
            if arguments[1] != 0 {
                result.push_str(&format!("${}{{LR}}$ ", LATEX_STYLES[1]));
            }
            result.push_str(r"$\rightarrow \mathrm{stack}$");
        }
        result
    }
}

impl<const P: usize, const F: usize> InstructionFormat for InstructionType<P, F> {
    fn name(&self) -> &'static str {
        self.name
    }

    fn definition(&self) -> String {
        let separator = if self.size == 16 {
            ""
        } else {
            r"\\[3pt] \phantom{.}"
        };
        format!(
            "\\makebox[3.5em][l]{{\\sffamily\\bfseries {}}} \\makebox[13em][l]{{{}}} {} \\hfill {}",
            self.name,
            self.semantics(),
            separator,
            self.bit_pattern()
        )
    }

    fn operation(&self) -> String {
        format!("\\arm{{{}}}: {}", self.name, self.semantics())
    }

    fn semantics(&self) -> String {
        let mut parameters = Vec::<String>::new();
        let mut current_field = 0;
        for (i, parameter) in self.parameters.iter().enumerate() {
            let mut value = String::new();
            let mut current_bit = self.fields[current_field].bit_range.end;
            while current_field < self.fields.len()
                && self.fields[current_field].parameter == i as u8
            {
                let field = &self.fields[current_field];
                assert!(field.bit_range.end == current_bit);
                if !value.is_empty() {
                    value.push_str("{:}");
                }
                value.push_str(field.name);
                current_bit = field.bit_range.start;
                current_field += 1;
            }
            assert!(!value.is_empty());
            if parameter.multiple > 1 {
                parameters.push(format!("{}*{}", parameter.multiple, value));
            } else {
                parameters.push(value);
            }
        }
        let mut result = String::from(self.operation);
        for (i, parameter) in parameters.iter().enumerate() {
            result = result.replace(format!("#{}", i).as_str(), parameter.as_str());
        }
        result
    }

    fn bit_pattern(&self) -> String {
        let mut field_number = [0u8; 32];
        for (i, field) in self.fields.iter().enumerate() {
            for j in 0..field.bit_range.len() {
                field_number[field.offset as usize + j] = i as u8 + 1;
            }
        }
        let mut result = String::new();
        result.push_str("\\begin{tikzpicture}[x=0.75ex,y=1ex,baseline=0.7ex]\n");
        result.push_str(&format!(
            "\\useasboundingbox (0,0) rectangle +({},3) ;\n",
            (self.size as usize) * 3
        ));
        for i in 0..self.size as usize {
            if i > 0 {
                if field_number[i] != field_number[i - 1] {
                    result.push_str(&format!(
                        "\\draw ({},0) -- +(0,3) ;\n",
                        (self.size as usize - i) * 3
                    ));
                } else {
                    result.push_str(&format!(
                        "\\draw[gray] ({},0) -- +(0,0.7) ;\n",
                        (self.size as usize - i) * 3
                    ));
                }
            }
            if field_number[i] == 0 {
                result.push_str(&format!(
                    "\\node[anchor=base] at ({}.5,0.7) {{{}}} ;\n",
                    (self.size as usize - i) * 3 - 2,
                    (self.opcode >> i) & 1
                ));
            }
        }
        result.push_str(&format!(
            "\\draw (0,0) rectangle +({},3) ;\n",
            self.size * 3
        ));
        for field in &self.fields {
            result.push_str(&format!(
                "\\node[anchor=base] at ({},0.8) {{${}$}} ;\n",
                ((self.size - field.offset) * 3) as f32 - field.bit_range.len() as f32 * 1.5,
                field.name
            ));
        }
        result.push_str("\\end{tikzpicture}\n");
        result
    }

    fn encode(&self, arguments: &[u32]) -> u32 {
        let mut normalized_arguments = [0; P];
        for (i, parameter) in self.parameters.iter().enumerate() {
            assert!(arguments[i] as i32 % parameter.multiple == 0);
            normalized_arguments[i] = (arguments[i] as i32 / parameter.multiple) as u32;
            assert!(parameter.range.contains(&(normalized_arguments[i] as i32)));
            if parameter.range.start == 1
                && parameter.range.end == 33
                && normalized_arguments[i] == 32
            {
                normalized_arguments[i] = 0;
            }
        }
        let mut result = self.opcode;
        for field in &self.fields {
            let value = normalized_arguments[field.parameter as usize] >> field.bit_range.start
                & ((1 << (field.bit_range.end - field.bit_range.start)) - 1);
            result |= value << field.offset;
        }
        result
    }

    fn concrete_semantics(&self, encoding: u32, styled: bool) -> String {
        let mut arguments = [0; P];
        for field in &self.fields {
            let bits = (encoding >> field.offset) & ((1 << field.bit_range.len()) - 1);
            arguments[field.parameter as usize] |= bits << field.bit_range.start;
        }
        let special_result = self.concrete_semantics(arguments);
        if !special_result.is_empty() {
            return special_result;
        }
        let mut result = String::from(self.operation);
        for (i, argument) in arguments.iter().enumerate() {
            let style = if styled { LATEX_STYLES[i] } else { "" };
            if self.parameters[i].multiple == 1 {
                result = result.replace(
                    format!("#{}", i).as_str(),
                    format!("{}{{{}}}", style, argument).as_str(),
                );
            } else {
                result = result.replace(
                    format!("#{}", i).as_str(),
                    format!("{}*{}{{{}}}", self.parameters[i].multiple, style, argument).as_str(),
                );
            }
        }
        result
    }

    fn concrete_bit_pattern(&self, encoding: u32) -> String {
        let mut field_number = [0u8; 32];
        for (i, field) in self.fields.iter().enumerate() {
            for j in 0..field.bit_range.len() {
                field_number[field.offset as usize + j] = i as u8 + 1;
            }
        }
        let mut result = String::new();
        result.push_str("\\begin{tikzpicture}[x=0.45ex,y=1ex,baseline=0.7ex]\n");
        result.push_str(&format!(
            "\\useasboundingbox (0,0) rectangle +({},3) ;\n",
            16 * 3
        ));
        for i in 0..16 {
            if i > 0 && field_number[i] != field_number[i - 1] {
                result.push_str(&format!("\\draw ({},0) -- +(0,3) ;\n", (16 - i) * 3));
            }
            if field_number[i] > 0 {
                result.push_str(&format!(
                    "\\node[anchor=base] at ({}.5,0.7) {{${}{{{}}}$}} ;\n",
                    (16 - i) * 3 - 2,
                    LATEX_STYLES[self.fields[field_number[i] as usize - 1].parameter as usize],
                    (encoding >> i) & 1
                ));
            } else {
                result.push_str(&format!(
                    "\\node[anchor=base] at ({}.5,0.7) {{{}}} ;\n",
                    (16 - i) * 3 - 2,
                    (encoding >> i) & 1
                ));
            }
        }
        result.push_str(&format!("\\draw (0,0) rectangle +({},3) ;\n", 16 * 3));
        result.push_str("\\end{tikzpicture}\n");
        result
    }

    fn concrete_top_bit_pattern(&self, encoding: u32) -> String {
        assert!(self.size == 32);
        let mut field_number = [0u8; 32];
        for (i, field) in self.fields.iter().enumerate() {
            for j in 0..field.bit_range.len() {
                field_number[field.offset as usize + j] = i as u8 + 1;
            }
        }
        let mut result = String::new();
        result.push_str("\\begin{tikzpicture}[x=0.45ex,y=1ex,baseline=0.7ex]\n");
        result.push_str(&format!(
            "\\useasboundingbox (0,0) rectangle +({},3) ;\n",
            16 * 3
        ));
        for i in 16..32 {
            if i > 16 && field_number[i] != field_number[i - 1] {
                result.push_str(&format!("\\draw ({},0) -- +(0,3) ;\n", (32 - i) * 3));
            }
            if field_number[i] > 0 {
                result.push_str(&format!(
                    "\\node[anchor=base] at ({}.5,0.7) {{${}{{{}}}$}} ;\n",
                    (32 - i) * 3 - 2,
                    LATEX_STYLES[self.fields[field_number[i] as usize - 1].parameter as usize],
                    (encoding >> i) & 1
                ));
            } else {
                result.push_str(&format!(
                    "\\node[anchor=base] at ({}.5,0.7) {{{}}} ;\n",
                    (32 - i) * 3 - 2,
                    (encoding >> i) & 1
                ));
            }
        }
        result.push_str(&format!("\\draw (0,0) rectangle +({},3) ;\n", 16 * 3));
        result.push_str("\\end{tikzpicture}\n");
        result
    }
}

pub const GENERIC_16BIT: InstructionType<1, 2> = InstructionType::<1, 2> {
    name: "",
    operation: "",
    size: 16,
    opcode: 0,
    parameters: [Parameter {
        name: "",
        range: 0..i32::MAX,
        multiple: 1,
    }],
    fields: [
        Field {
            name: "byte_1",
            parameter: 0,
            bit_range: 8..16,
            offset: 8,
        },
        Field {
            name: "byte_0",
            parameter: 0,
            bit_range: 0..8,
            offset: 0,
        },
    ],
};

pub const GENERIC_32BIT: InstructionType<1, 4> = InstructionType::<1, 4> {
    name: "",
    operation: "",
    size: 32,
    opcode: 0,
    parameters: [Parameter {
        name: "",
        range: 0..i32::MAX,
        multiple: 1,
    }],
    fields: [
        Field {
            name: "byte_3",
            parameter: 0,
            bit_range: 24..32,
            offset: 24,
        },
        Field {
            name: "byte_2",
            parameter: 0,
            bit_range: 16..24,
            offset: 16,
        },
        Field {
            name: "byte_1",
            parameter: 0,
            bit_range: 8..16,
            offset: 8,
        },
        Field {
            name: "byte_0",
            parameter: 0,
            bit_range: 0..8,
            offset: 0,
        },
    ],
};

pub const ADD_RDN_IMM8: InstructionType<2, 2> = InstructionType::<2, 2> {
    name: "ADD",
    operation: r"$\mathrm{R}#0 \leftarrow \mathrm{R}#0 + #1$",
    size: 16,
    opcode: 0b00110_000_00000000,
    parameters: [
        Parameter {
            name: "z",
            range: 0..8,
            multiple: 1,
        },
        Parameter {
            name: "c",
            range: 0..256,
            multiple: 1,
        },
    ],
    fields: [
        Field {
            name: "z",
            parameter: 0,
            bit_range: 0..3,
            offset: 8,
        },
        Field {
            name: "c",
            parameter: 1,
            bit_range: 0..8,
            offset: 0,
        },
    ],
};

pub const ADD_RD_RN_RM: InstructionType<3, 3> = InstructionType::<3, 3> {
    name: "ADD",
    operation: r"$\mathrm{R}#0 \leftarrow \mathrm{R}#1 + \mathrm{R}#2$",
    size: 16,
    opcode: 0b0001100_000_000_000,
    parameters: [
        Parameter {
            name: "z",
            range: 0..8,
            multiple: 1,
        },
        Parameter {
            name: "x",
            range: 0..8,
            multiple: 1,
        },
        Parameter {
            name: "y",
            range: 0..8,
            multiple: 1,
        },
    ],
    fields: [
        Field {
            name: "z",
            parameter: 0,
            bit_range: 0..3,
            offset: 0,
        },
        Field {
            name: "x",
            parameter: 1,
            bit_range: 0..3,
            offset: 3,
        },
        Field {
            name: "y",
            parameter: 2,
            bit_range: 0..3,
            offset: 6,
        },
    ],
};

pub const ADD_RD_SP_IMM8: InstructionType<2, 2> = InstructionType::<2, 2> {
    name: "ADD",
    operation: r"$\mathrm{R}#0 \leftarrow \mathrm{SP} + #1$",
    size: 16,
    opcode: 0b10101_000_00000000,
    parameters: [
        Parameter {
            name: "z",
            range: 0..8,
            multiple: 1,
        },
        Parameter {
            name: "c",
            range: 0..256,
            multiple: 4,
        },
    ],
    fields: [
        Field {
            name: "z",
            parameter: 0,
            bit_range: 0..3,
            offset: 8,
        },
        Field {
            name: "c",
            parameter: 1,
            bit_range: 0..8,
            offset: 0,
        },
    ],
};

pub const ADD_SP_SP_IMM7: InstructionType<1, 1> = InstructionType::<1, 1> {
    name: "ADD",
    operation: r"$\mathrm{SP} \leftarrow \mathrm{SP} + #0$",
    size: 16,
    opcode: 0b101100000_0000000,
    parameters: [Parameter {
        name: "c",
        range: 0..128,
        multiple: 4,
    }],
    fields: [Field {
        name: "c",
        parameter: 0,
        bit_range: 0..7,
        offset: 0,
    }],
};

pub const ADR_RD_MINUS_IMM12: InstructionType<2, 4> = InstructionType::<2, 4> {
    name: "ADR",
    operation: r"$\mathrm{R}#0 \leftarrow \lfloor\mathrm{PC}\rfloor_4 - #1$",
    size: 32,
    opcode: 0b0_000_0000_00000000_11110_0_1010101111,
    parameters: [
        Parameter {
            name: "z",
            range: 0..16,
            multiple: 1,
        },
        Parameter {
            name: "c",
            range: 0..4096,
            multiple: 1,
        },
    ],
    fields: [
        Field {
            name: "z",
            parameter: 0,
            bit_range: 0..4,
            offset: 24,
        },
        Field {
            name: "c_2",
            parameter: 1,
            bit_range: 11..12,
            offset: 10,
        },
        Field {
            name: "c_1",
            parameter: 1,
            bit_range: 8..11,
            offset: 28,
        },
        Field {
            name: "c_0",
            parameter: 1,
            bit_range: 0..8,
            offset: 16,
        },
    ],
};

pub const AND_RDN_RM: InstructionType<2, 2> = InstructionType::<2, 2> {
    name: "AND",
    operation: r"$\mathrm{R}#0 \leftarrow \mathrm{R}#0 \wedge \mathrm{R}#1$",
    size: 16,
    opcode: 0b0100000000_000_000,
    parameters: [
        Parameter {
            name: "z",
            range: 0..8,
            multiple: 1,
        },
        Parameter {
            name: "x",
            range: 0..8,
            multiple: 1,
        },
    ],
    fields: [
        Field {
            name: "z",
            parameter: 0,
            bit_range: 0..3,
            offset: 0,
        },
        Field {
            name: "x",
            parameter: 1,
            bit_range: 0..3,
            offset: 3,
        },
    ],
};

pub const B_IMM11: InstructionType<1, 1> = InstructionType::<1, 1> {
    name: "B",
    operation: r"$\mathrm{PC} \leftarrow \mathrm{PC} + signed_{12}(#0)$",
    size: 16,
    opcode: 0b11100_00000000000,
    parameters: [Parameter {
        name: "c",
        range: -1024..1024,
        multiple: 2,
    }],
    fields: [Field {
        name: "c",
        parameter: 0,
        bit_range: 0..11,
        offset: 0,
    }],
};

pub const BL_IMM22: InstructionType<1, 2> = InstructionType::<1, 2> {
    name: "BL",
    operation: r"$\mathrm{PC} \leftarrow \mathrm{PC} + signed_{23}(#0), \mathrm{LR} \leftarrow a+5$",
    size: 32,
    opcode: 0b11_1_1_1_00000000000_11110_0_0000000000,
    parameters: [Parameter {
        name: "c",
        range: -2097152..2097152,
        multiple: 2,
    }],
    fields: [
        Field {
            name: "c_1",
            parameter: 0,
            bit_range: 11..22,
            offset: 0,
        },
        Field {
            name: "c_0",
            parameter: 0,
            bit_range: 0..11,
            offset: 16,
        },
    ],
};

pub const BLX_RM: InstructionType<1, 1> = InstructionType::<1, 1> {
    name: "BLX",
    operation: r"$\mathrm{PC} \leftarrow \mathrm{R}#0-1, \mathrm{LR} \leftarrow a+3$",
    size: 16,
    opcode: 0b010001111_0000_000,
    parameters: [Parameter {
        name: "x",
        range: 0..16,
        multiple: 1,
    }],
    fields: [Field {
        name: "x",
        parameter: 0,
        bit_range: 0..4,
        offset: 3,
    }],
};

pub const BX_RM: InstructionType<1, 1> = InstructionType::<1, 1> {
    name: "BX",
    operation: r"$\mathrm{PC} \leftarrow \mathrm{R}#0-1$",
    size: 16,
    opcode: 0b010001110_0000_000,
    parameters: [Parameter {
        name: "x",
        range: 0..16,
        multiple: 1,
    }],
    fields: [Field {
        name: "x",
        parameter: 0,
        bit_range: 0..4,
        offset: 3,
    }],
};

pub const CBZ_RN_IMM6: InstructionType<2, 3> = InstructionType::<2, 3> {
    name: "CBZ",
    operation: r"$\mathrm{PC} \leftarrow \mathrm{PC} + #1, if \mathrm{R}#0=0$",
    size: 16,
    opcode: 0b101100_0_1_00000_000,
    parameters: [
        Parameter {
            name: "x",
            range: 0..8,
            multiple: 1,
        },
        Parameter {
            name: "c",
            range: 0..64,
            multiple: 2,
        },
    ],
    fields: [
        Field {
            name: "x",
            parameter: 0,
            bit_range: 0..3,
            offset: 0,
        },
        Field {
            name: "c_1",
            parameter: 1,
            bit_range: 5..6,
            offset: 9,
        },
        Field {
            name: "c_0",
            parameter: 1,
            bit_range: 0..5,
            offset: 3,
        },
    ],
};

pub const CMP_RN_IMM8: InstructionType<2, 2> = InstructionType::<2, 2> {
    name: "CMP",
    operation: r"$compare(\mathrm{R}#0, #1)$",
    size: 16,
    opcode: 0b00101_000_00000000,
    parameters: [
        Parameter {
            name: "x",
            range: 0..8,
            multiple: 1,
        },
        Parameter {
            name: "c",
            range: 0..256,
            multiple: 1,
        },
    ],
    fields: [
        Field {
            name: "x",
            parameter: 0,
            bit_range: 0..3,
            offset: 8,
        },
        Field {
            name: "c",
            parameter: 1,
            bit_range: 0..8,
            offset: 0,
        },
    ],
};

pub const CMP_RN_RM: InstructionType<2, 2> = InstructionType::<2, 2> {
    name: "CMP",
    operation: r"$compare(\mathrm{R}#0, \mathrm{R}#1)$",
    size: 16,
    opcode: 0b0100001010_000_000,
    parameters: [
        Parameter {
            name: "x",
            range: 0..8,
            multiple: 1,
        },
        Parameter {
            name: "y",
            range: 0..8,
            multiple: 1,
        },
    ],
    fields: [
        Field {
            name: "x",
            parameter: 0,
            bit_range: 0..3,
            offset: 0,
        },
        Field {
            name: "y",
            parameter: 1,
            bit_range: 0..3,
            offset: 3,
        },
    ],
};

pub const IT: InstructionType<2, 6> = InstructionType::<2, 6> {
    name: "IT",
    operation: r"if $c_0{:}c_n$ then I$_n$",
    size: 16,
    opcode: 0b10111111_0000_0000,
    parameters: [
        Parameter {
            name: "cond",
            range: 0..16,
            multiple: 1,
        },
        Parameter {
            name: "thenelse",
            range: 0..16,
            multiple: 1,
        },
    ],
    fields: [
        Field {
            name: "c_0",
            parameter: 0,
            bit_range: 1..4,
            offset: 5,
        },
        Field {
            name: "c_1",
            parameter: 0,
            bit_range: 0..1,
            offset: 4,
        },
        Field {
            name: "c_2",
            parameter: 1,
            bit_range: 3..4,
            offset: 3,
        },
        Field {
            name: "c_3",
            parameter: 1,
            bit_range: 2..3,
            offset: 2,
        },
        Field {
            name: "c_4",
            parameter: 1,
            bit_range: 1..2,
            offset: 1,
        },
        Field {
            name: "c_5",
            parameter: 1,
            bit_range: 0..1,
            offset: 0,
        },
    ],
};

pub const LDRB_RT_RN_IMM5: InstructionType<3, 3> = InstructionType::<3, 3> {
    name: "LDRB",
    operation: r"$\mathrm{R}#0 \leftarrow \mathrm{mem8}[\mathrm{R}#1 + #2]$",
    size: 16,
    opcode: 0b01111_00000_000_000,
    parameters: [
        Parameter {
            name: "z",
            range: 0..8,
            multiple: 1,
        },
        Parameter {
            name: "x",
            range: 0..8,
            multiple: 1,
        },
        Parameter {
            name: "c",
            range: 0..32,
            multiple: 1,
        },
    ],
    fields: [
        Field {
            name: "z",
            parameter: 0,
            bit_range: 0..3,
            offset: 0,
        },
        Field {
            name: "x",
            parameter: 1,
            bit_range: 0..3,
            offset: 3,
        },
        Field {
            name: "c",
            parameter: 2,
            bit_range: 0..5,
            offset: 6,
        },
    ],
};

pub const LDRH_RT_RN_IMM5: InstructionType<3, 3> = InstructionType::<3, 3> {
    name: "LDRH",
    operation: r"$\mathrm{R}#0 \leftarrow \mathrm{mem16}[\mathrm{R}#1 + #2]$",
    size: 16,
    opcode: 0b10001_00000_000_000,
    parameters: [
        Parameter {
            name: "z",
            range: 0..8,
            multiple: 1,
        },
        Parameter {
            name: "x",
            range: 0..8,
            multiple: 1,
        },
        Parameter {
            name: "c",
            range: 0..32,
            multiple: 2,
        },
    ],
    fields: [
        Field {
            name: "z",
            parameter: 0,
            bit_range: 0..3,
            offset: 0,
        },
        Field {
            name: "x",
            parameter: 1,
            bit_range: 0..3,
            offset: 3,
        },
        Field {
            name: "c",
            parameter: 2,
            bit_range: 0..5,
            offset: 6,
        },
    ],
};

pub const LDR_RT_PC_IMM8: InstructionType<2, 2> = InstructionType::<2, 2> {
    name: "LDR",
    operation: r"$\mathrm{R}#0 \leftarrow \mathrm{mem32}[\lfloor\mathrm{PC}\rfloor_4 + #1]$",
    size: 16,
    opcode: 0b01001_000_00000000,
    parameters: [
        Parameter {
            name: "z",
            range: 0..8,
            multiple: 1,
        },
        Parameter {
            name: "c",
            range: 0..256,
            multiple: 4,
        },
    ],
    fields: [
        Field {
            name: "z",
            parameter: 0,
            bit_range: 0..3,
            offset: 8,
        },
        Field {
            name: "c",
            parameter: 1,
            bit_range: 0..8,
            offset: 0,
        },
    ],
};

pub const LDR_RT_RN_IMM5: InstructionType<3, 3> = InstructionType::<3, 3> {
    name: "LDR",
    operation: r"$\mathrm{R}#0 \leftarrow \mathrm{mem32}[\mathrm{R}#1 + #2]$",
    size: 16,
    opcode: 0b01101_00000_000_000,
    parameters: [
        Parameter {
            name: "z",
            range: 0..8,
            multiple: 1,
        },
        Parameter {
            name: "x",
            range: 0..8,
            multiple: 1,
        },
        Parameter {
            name: "c",
            range: 0..32,
            multiple: 4,
        },
    ],
    fields: [
        Field {
            name: "z",
            parameter: 0,
            bit_range: 0..3,
            offset: 0,
        },
        Field {
            name: "x",
            parameter: 1,
            bit_range: 0..3,
            offset: 3,
        },
        Field {
            name: "c",
            parameter: 2,
            bit_range: 0..5,
            offset: 6,
        },
    ],
};

pub const LDR_RT_SP_IMM8: InstructionType<2, 2> = InstructionType::<2, 2> {
    name: "LDR",
    operation: r"$\mathrm{R}#0 \leftarrow \mathrm{mem32}[\mathrm{SP} + #1]$",
    size: 16,
    opcode: 0b10011_000_00000000,
    parameters: [
        Parameter {
            name: "z",
            range: 0..8,
            multiple: 1,
        },
        Parameter {
            name: "c",
            range: 0..256,
            multiple: 4,
        },
    ],
    fields: [
        Field {
            name: "z",
            parameter: 0,
            bit_range: 0..3,
            offset: 8,
        },
        Field {
            name: "c",
            parameter: 1,
            bit_range: 0..8,
            offset: 0,
        },
    ],
};

pub const LSL_RDN_RM: InstructionType<2, 2> = InstructionType::<2, 2> {
    name: "LSL",
    operation: r"$\mathrm{R}#0 \leftarrow \mathrm{R}#0 \ll (\mathrm{R}#1\ mod\ 32)$",
    size: 16,
    opcode: 0b0100000010_000_000,
    parameters: [
        Parameter {
            name: "z",
            range: 0..8,
            multiple: 1,
        },
        Parameter {
            name: "x",
            range: 0..8,
            multiple: 1,
        },
    ],
    fields: [
        Field {
            name: "z",
            parameter: 0,
            bit_range: 0..3,
            offset: 0,
        },
        Field {
            name: "x",
            parameter: 1,
            bit_range: 0..3,
            offset: 3,
        },
    ],
};

pub const LSL_RD_RM_IMM5: InstructionType<3, 3> = InstructionType::<3, 3> {
    name: "LSL",
    operation: r"$\mathrm{R}#0 \leftarrow \mathrm{R}#1 \ll #2$",
    size: 16,
    opcode: 0b00000_00000_000_000,
    parameters: [
        Parameter {
            name: "z",
            range: 0..8,
            multiple: 1,
        },
        Parameter {
            name: "x",
            range: 0..8,
            multiple: 1,
        },
        Parameter {
            name: "c",
            range: 0..32,
            multiple: 1,
        },
    ],
    fields: [
        Field {
            name: "z",
            parameter: 0,
            bit_range: 0..3,
            offset: 0,
        },
        Field {
            name: "x",
            parameter: 1,
            bit_range: 0..3,
            offset: 3,
        },
        Field {
            name: "c",
            parameter: 2,
            bit_range: 0..5,
            offset: 6,
        },
    ],
};

pub const LSR_RDN_RM: InstructionType<2, 2> = InstructionType::<2, 2> {
    name: "LSR",
    operation: r"$\mathrm{R}#0 \leftarrow \mathrm{R}#0 \gg (\mathrm{R}#1\ mod\ 32)$",
    size: 16,
    opcode: 0b0100000011_000_000,
    parameters: [
        Parameter {
            name: "z",
            range: 0..8,
            multiple: 1,
        },
        Parameter {
            name: "x",
            range: 0..8,
            multiple: 1,
        },
    ],
    fields: [
        Field {
            name: "z",
            parameter: 0,
            bit_range: 0..3,
            offset: 0,
        },
        Field {
            name: "x",
            parameter: 1,
            bit_range: 0..3,
            offset: 3,
        },
    ],
};

pub const LSR_RD_RM_IMM5: InstructionType<3, 3> = InstructionType::<3, 3> {
    name: "LSR",
    operation: r"$\mathrm{R}#0 \leftarrow \mathrm{R}#1 \gg #2$",
    size: 16,
    opcode: 0b00001_00000_000_000,
    parameters: [
        Parameter {
            name: "z",
            range: 0..8,
            multiple: 1,
        },
        Parameter {
            name: "x",
            range: 0..8,
            multiple: 1,
        },
        Parameter {
            name: "c",
            range: 1..33,
            multiple: 1,
        },
    ],
    fields: [
        Field {
            name: "z",
            parameter: 0,
            bit_range: 0..3,
            offset: 0,
        },
        Field {
            name: "x",
            parameter: 1,
            bit_range: 0..3,
            offset: 3,
        },
        Field {
            name: "c",
            parameter: 2,
            bit_range: 0..5,
            offset: 6,
        },
    ],
};

pub const MOV_RD_IMM8: InstructionType<2, 2> = InstructionType::<2, 2> {
    name: "MOV",
    operation: r"$\mathrm{R}#0 \leftarrow #1$",
    size: 16,
    opcode: 0b00100_000_00000000,
    parameters: [
        Parameter {
            name: "z",
            range: 0..8,
            multiple: 1,
        },
        Parameter {
            name: "c",
            range: 0..256,
            multiple: 1,
        },
    ],
    fields: [
        Field {
            name: "z",
            parameter: 0,
            bit_range: 0..3,
            offset: 8,
        },
        Field {
            name: "c",
            parameter: 1,
            bit_range: 0..8,
            offset: 0,
        },
    ],
};

pub const MOV_RD_RM: InstructionType<2, 3> = InstructionType::<2, 3> {
    name: "MOV",
    operation: r"$\mathrm{R}#0 \leftarrow \mathrm{R}#1$",
    size: 16,
    opcode: 0b01000110_0_0000_000,
    parameters: [
        Parameter {
            name: "z",
            range: 0..16,
            multiple: 1,
        },
        Parameter {
            name: "x",
            range: 0..16,
            multiple: 1,
        },
    ],
    fields: [
        Field {
            name: "z_1",
            parameter: 0,
            bit_range: 3..4,
            offset: 7,
        },
        Field {
            name: "z_0",
            parameter: 0,
            bit_range: 0..3,
            offset: 0,
        },
        Field {
            name: "x",
            parameter: 1,
            bit_range: 0..4,
            offset: 3,
        },
    ],
};

pub const MOVT_RD_IMM16: InstructionType<2, 5> = InstructionType::<2, 5> {
    name: "MOVT",
    operation: r"$\mathrm{R}#0[31..16] \leftarrow #1$",
    size: 32,
    opcode: 0b0_000_0000_00000000_11110_0_101100_0000,
    parameters: [
        Parameter {
            name: "z",
            range: 0..16,
            multiple: 1,
        },
        Parameter {
            name: "c",
            range: 0..65536,
            multiple: 1,
        },
    ],
    fields: [
        Field {
            name: "z",
            parameter: 0,
            bit_range: 0..4,
            offset: 24,
        },
        Field {
            name: "c_3",
            parameter: 1,
            bit_range: 12..16,
            offset: 0,
        },
        Field {
            name: "c_2",
            parameter: 1,
            bit_range: 11..12,
            offset: 10,
        },
        Field {
            name: "c_1",
            parameter: 1,
            bit_range: 8..11,
            offset: 28,
        },
        Field {
            name: "c_0",
            parameter: 1,
            bit_range: 0..8,
            offset: 16,
        },
    ],
};

pub const MOVW_RD_IMM16: InstructionType<2, 5> = InstructionType::<2, 5> {
    name: "MOVW",
    operation: r"$\mathrm{R}#0 \leftarrow #1$",
    size: 32,
    opcode: 0b0_000_0000_00000000_11110_0_100100_0000,
    parameters: [
        Parameter {
            name: "z",
            range: 0..16,
            multiple: 1,
        },
        Parameter {
            name: "c",
            range: 0..65536,
            multiple: 1,
        },
    ],
    fields: [
        Field {
            name: "z",
            parameter: 0,
            bit_range: 0..4,
            offset: 24,
        },
        Field {
            name: "c_3",
            parameter: 1,
            bit_range: 12..16,
            offset: 0,
        },
        Field {
            name: "c_2",
            parameter: 1,
            bit_range: 11..12,
            offset: 10,
        },
        Field {
            name: "c_1",
            parameter: 1,
            bit_range: 8..11,
            offset: 28,
        },
        Field {
            name: "c_0",
            parameter: 1,
            bit_range: 0..8,
            offset: 16,
        },
    ],
};

pub const MRS_RD_SYSM: InstructionType<2, 2> = InstructionType::<2, 2> {
    name: "MRS",
    operation: r"$\mathrm{R}#0 \leftarrow \mathrm{SYSR}#1$",
    size: 32,
    opcode: 0b1000_0000_00000000_1111001111101111,
    parameters: [
        Parameter {
            name: "z",
            range: 0..16,
            multiple: 1,
        },
        Parameter {
            name: "x",
            range: 0..256,
            multiple: 1,
        },
    ],
    fields: [
        Field {
            name: "z",
            parameter: 0,
            bit_range: 0..4,
            offset: 24,
        },
        Field {
            name: "x",
            parameter: 1,
            bit_range: 0..8,
            offset: 16,
        },
    ],
};

pub const MSR_SYSM_RD: InstructionType<2, 2> = InstructionType::<2, 2> {
    name: "MSR",
    operation: r"$\mathrm{SYSR}#0 \leftarrow \mathrm{R}#1$",
    size: 32,
    opcode: 0b10001000_00000000_111100111000_0000,
    parameters: [
        Parameter {
            name: "z",
            range: 0..256,
            multiple: 1,
        },
        Parameter {
            name: "x",
            range: 0..16,
            multiple: 1,
        },
    ],
    fields: [
        Field {
            name: "z",
            parameter: 0,
            bit_range: 0..8,
            offset: 16,
        },
        Field {
            name: "x",
            parameter: 1,
            bit_range: 0..4,
            offset: 0,
        },
    ],
};

pub const MUL_RDM_RN: InstructionType<2, 2> = InstructionType::<2, 2> {
    name: "MUL",
    operation: r"$\mathrm{R}#0 \leftarrow \mathrm{R}#0 * \mathrm{R}#1$",
    size: 16,
    opcode: 0b0100001101_000_000,
    parameters: [
        Parameter {
            name: "z",
            range: 0..8,
            multiple: 1,
        },
        Parameter {
            name: "x",
            range: 0..8,
            multiple: 1,
        },
    ],
    fields: [
        Field {
            name: "z",
            parameter: 0,
            bit_range: 0..3,
            offset: 0,
        },
        Field {
            name: "x",
            parameter: 1,
            bit_range: 0..3,
            offset: 3,
        },
    ],
};

pub const ORR_RDN_RM: InstructionType<2, 2> = InstructionType::<2, 2> {
    name: "ORR",
    operation: r"$\mathrm{R}#0 \leftarrow \mathrm{R}#0 \vee \mathrm{R}#1$",
    size: 16,
    opcode: 0b0100001100_000_000,
    parameters: [
        Parameter {
            name: "z",
            range: 0..8,
            multiple: 1,
        },
        Parameter {
            name: "x",
            range: 0..8,
            multiple: 1,
        },
    ],
    fields: [
        Field {
            name: "z",
            parameter: 0,
            bit_range: 0..3,
            offset: 0,
        },
        Field {
            name: "x",
            parameter: 1,
            bit_range: 0..3,
            offset: 3,
        },
    ],
};

pub const POP: InstructionType<2, 2> = InstructionType::<2, 2> {
    name: "POP",
    operation: r"$#0, #1 \leftarrow \mathrm{stack}$",
    size: 16,
    opcode: 0b1011110_0_00000000,
    parameters: [
        Parameter {
            name: "registers",
            range: 0..256,
            multiple: 1,
        },
        Parameter {
            name: "p",
            range: 0..2,
            multiple: 1,
        },
    ],
    fields: [
        Field {
            name: "registers",
            parameter: 0,
            bit_range: 0..8,
            offset: 0,
        },
        Field {
            name: "p",
            parameter: 1,
            bit_range: 0..1,
            offset: 8,
        },
    ],
};

pub const PUSH: InstructionType<2, 2> = InstructionType::<2, 2> {
    name: "PUSH",
    operation: r"$#0, #1 \rightarrow \mathrm{stack}$",
    size: 16,
    opcode: 0b1011010_0_00000000,
    parameters: [
        Parameter {
            name: "registers",
            range: 0..256,
            multiple: 1,
        },
        Parameter {
            name: "l",
            range: 0..2,
            multiple: 1,
        },
    ],
    fields: [
        Field {
            name: "registers",
            parameter: 0,
            bit_range: 0..8,
            offset: 0,
        },
        Field {
            name: "l",
            parameter: 1,
            bit_range: 0..1,
            offset: 8,
        },
    ],
};

pub const STRB_RT_RN_IMM5: InstructionType<3, 3> = InstructionType::<3, 3> {
    name: "STRB",
    operation: r"$\mathrm{R}#0 \rightarrow \mathrm{mem8}[\mathrm{R}#1 + #2]$",
    size: 16,
    opcode: 0b01110_00000_000_000,
    parameters: [
        Parameter {
            name: "z",
            range: 0..8,
            multiple: 1,
        },
        Parameter {
            name: "x",
            range: 0..8,
            multiple: 1,
        },
        Parameter {
            name: "c",
            range: 0..32,
            multiple: 1,
        },
    ],
    fields: [
        Field {
            name: "z",
            parameter: 0,
            bit_range: 0..3,
            offset: 0,
        },
        Field {
            name: "x",
            parameter: 1,
            bit_range: 0..3,
            offset: 3,
        },
        Field {
            name: "c",
            parameter: 2,
            bit_range: 0..5,
            offset: 6,
        },
    ],
};

pub const STRH_RT_RN_IMM5: InstructionType<3, 3> = InstructionType::<3, 3> {
    name: "STRH",
    operation: r"$\mathrm{R}#0 \rightarrow \mathrm{mem16}[\mathrm{R}#1 + #2]$",
    size: 16,
    opcode: 0b10000_00000_000_000,
    parameters: [
        Parameter {
            name: "z",
            range: 0..8,
            multiple: 1,
        },
        Parameter {
            name: "x",
            range: 0..8,
            multiple: 1,
        },
        Parameter {
            name: "c",
            range: 0..32,
            multiple: 2,
        },
    ],
    fields: [
        Field {
            name: "z",
            parameter: 0,
            bit_range: 0..3,
            offset: 0,
        },
        Field {
            name: "x",
            parameter: 1,
            bit_range: 0..3,
            offset: 3,
        },
        Field {
            name: "c",
            parameter: 2,
            bit_range: 0..5,
            offset: 6,
        },
    ],
};

pub const STR_RT_RN_IMM5: InstructionType<3, 3> = InstructionType::<3, 3> {
    name: "STR",
    operation: r"$\mathrm{R}#0 \rightarrow \mathrm{mem32}[\mathrm{R}#1 + #2]$",
    size: 16,
    opcode: 0b01100_00000_000_000,
    parameters: [
        Parameter {
            name: "z",
            range: 0..8,
            multiple: 1,
        },
        Parameter {
            name: "x",
            range: 0..8,
            multiple: 1,
        },
        Parameter {
            name: "c",
            range: 0..32,
            multiple: 4,
        },
    ],
    fields: [
        Field {
            name: "z",
            parameter: 0,
            bit_range: 0..3,
            offset: 0,
        },
        Field {
            name: "x",
            parameter: 1,
            bit_range: 0..3,
            offset: 3,
        },
        Field {
            name: "c",
            parameter: 2,
            bit_range: 0..5,
            offset: 6,
        },
    ],
};

pub const STR_RT_SP_IMM8: InstructionType<2, 2> = InstructionType::<2, 2> {
    name: "STR",
    operation: r"$\mathrm{R}#0 \rightarrow \mathrm{mem32}[\mathrm{SP} + #1]$",
    size: 16,
    opcode: 0b10010_000_00000000,
    parameters: [
        Parameter {
            name: "z",
            range: 0..8,
            multiple: 1,
        },
        Parameter {
            name: "c",
            range: 0..256,
            multiple: 4,
        },
    ],
    fields: [
        Field {
            name: "z",
            parameter: 0,
            bit_range: 0..3,
            offset: 8,
        },
        Field {
            name: "c",
            parameter: 1,
            bit_range: 0..8,
            offset: 0,
        },
    ],
};

pub const SUB_RDN_IMM8: InstructionType<2, 2> = InstructionType::<2, 2> {
    name: "SUB",
    operation: r"$\mathrm{R}#0 \leftarrow \mathrm{R}#0 - #1$",
    size: 16,
    opcode: 0b00111_000_00000000,
    parameters: [
        Parameter {
            name: "z",
            range: 0..8,
            multiple: 1,
        },
        Parameter {
            name: "c",
            range: 0..256,
            multiple: 1,
        },
    ],
    fields: [
        Field {
            name: "z",
            parameter: 0,
            bit_range: 0..3,
            offset: 8,
        },
        Field {
            name: "c",
            parameter: 1,
            bit_range: 0..8,
            offset: 0,
        },
    ],
};

pub const SUB_RD_RN_RM: InstructionType<3, 3> = InstructionType::<3, 3> {
    name: "SUB",
    operation: r"$\mathrm{R}#0 \leftarrow \mathrm{R}#1 - \mathrm{R}#2$",
    size: 16,
    opcode: 0b0001101_000_000_000,
    parameters: [
        Parameter {
            name: "z",
            range: 0..8,
            multiple: 1,
        },
        Parameter {
            name: "x",
            range: 0..8,
            multiple: 1,
        },
        Parameter {
            name: "y",
            range: 0..8,
            multiple: 1,
        },
    ],
    fields: [
        Field {
            name: "z",
            parameter: 0,
            bit_range: 0..3,
            offset: 0,
        },
        Field {
            name: "x",
            parameter: 1,
            bit_range: 0..3,
            offset: 3,
        },
        Field {
            name: "y",
            parameter: 2,
            bit_range: 0..3,
            offset: 6,
        },
    ],
};

pub const SUB_SP_SP_IMM7: InstructionType<1, 1> = InstructionType::<1, 1> {
    name: "SUB",
    operation: r"$\mathrm{SP} \leftarrow \mathrm{SP} - #0$",
    size: 16,
    opcode: 0b101100001_0000000,
    parameters: [Parameter {
        name: "c",
        range: 0..128,
        multiple: 4,
    }],
    fields: [Field {
        name: "c",
        parameter: 0,
        bit_range: 0..7,
        offset: 0,
    }],
};

pub const SVC_IMM8: InstructionType<1, 1> = InstructionType::<1, 1> {
    name: "SVC",
    operation: r"$\mathrm{SVC\ interrupt}$",
    size: 16,
    opcode: 0b11011111_00000000,
    parameters: [Parameter {
        name: "c",
        range: 0..256,
        multiple: 1,
    }],
    fields: [Field {
        name: "c",
        parameter: 0,
        bit_range: 0..8,
        offset: 0,
    }],
};

pub const TBB_RN_RM: InstructionType<2, 2> = InstructionType::<2, 2> {
    name: "TBB",
    operation: r"$\mathrm{PC} \leftarrow \mathrm{PC} + 2*\mathrm{mem8}[\mathrm{R}#0 + \mathrm{R}#1]$",
    size: 32,
    opcode: 0b111100000000_0000_111010001101_0000,
    parameters: [
        Parameter {
            name: "x",
            range: 0..16,
            multiple: 1,
        },
        Parameter {
            name: "y",
            range: 0..16,
            multiple: 1,
        },
    ],
    fields: [
        Field {
            name: "x",
            parameter: 0,
            bit_range: 0..4,
            offset: 0,
        },
        Field {
            name: "y",
            parameter: 1,
            bit_range: 0..4,
            offset: 16,
        },
    ],
};

pub const UDF: InstructionType<1, 1> = InstructionType::<1, 1> {
    name: "UDF",
    operation: r"Undefined Instruction exception",
    size: 16,
    opcode: 0b11011110_00000000,
    parameters: [Parameter {
        name: "c",
        range: 0..256,
        multiple: 1,
    }],
    fields: [Field {
        name: "c",
        parameter: 0,
        bit_range: 0..8,
        offset: 0,
    }],
};

pub const UDIV_RD_RN_RM: InstructionType<3, 3> = InstructionType::<3, 3> {
    name: "UDIV",
    operation: r"$\mathrm{R}#0 \leftarrow \mathrm{R}#1\,/\,\mathrm{R}#2$",
    size: 32,
    opcode: 0b1111_0000_1111_0000_111110111011_0000,
    parameters: [
        Parameter {
            name: "z",
            range: 0..16,
            multiple: 1,
        },
        Parameter {
            name: "x",
            range: 0..16,
            multiple: 1,
        },
        Parameter {
            name: "y",
            range: 0..16,
            multiple: 1,
        },
    ],
    fields: [
        Field {
            name: "z",
            parameter: 0,
            bit_range: 0..4,
            offset: 24,
        },
        Field {
            name: "x",
            parameter: 1,
            bit_range: 0..4,
            offset: 0,
        },
        Field {
            name: "y",
            parameter: 2,
            bit_range: 0..4,
            offset: 16,
        },
    ],
};
