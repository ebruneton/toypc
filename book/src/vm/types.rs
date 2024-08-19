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

pub struct BytecodeParameter {
    pub name: &'static str,
    pub size: u32,
}

pub struct BytecodeInstructionType {
    pub name: &'static str,
    pub operation: &'static str,
    pub opcode: u8,
    pub parameter: Option<BytecodeParameter>,
}

impl BytecodeInstructionType {
    pub fn size_bytes(&self) -> u32 {
        if let Some(parameter) = &self.parameter {
            parameter.size + 1
        } else {
            1
        }
    }

    pub fn definition(&self) -> String {
        format!(
            "\\makebox[3.5em][l]{{\\sffamily\\bfseries {}}} \\makebox[13em][l]{{{}}} \\hfill {}",
            self.name,
            self.semantics(),
            self.byte_pattern()
        )
    }

    pub fn semantics(&self) -> String {
        let mut result = String::from(self.operation);
        if let Some(parameter) = &self.parameter {
            result = result.replace("#0", parameter.name);
        }
        result
    }

    pub fn byte_pattern(&self) -> String {
        let mut result = String::new();
        let size = self.size_bytes();
        result.push_str("\\begin{tikzpicture}[x=1ex,y=1ex,baseline=0.7ex]\n");
        result.push_str(&format!(
            "\\useasboundingbox (0,0) rectangle +({},3) ;\n",
            size * 4
        ));
        result.push_str(&format!("\\draw (0,0) rectangle +({},3) ;\n", size * 4));
        result.push_str(&format!("\\draw ({},0) -- +(0,3) ;\n", size * 4 - 4));
        for i in 1..(size - 1) as usize {
            result.push_str(&format!("\\draw[gray] ({},0) -- +(0,0.6) ;\n", i * 4));
        }
        result.push_str(&format!(
            "\\node[anchor=base] at ({},0.7) {{{:02X}}} ;\n",
            size * 4 - 2,
            self.opcode
        ));
        if let Some(parameter) = &self.parameter {
            result.push_str(&format!(
                "\\node[anchor=base] at ({},0.8) {{${}$}} ;\n",
                (size - 1) * 2,
                parameter.name
            ));
        }
        result.push_str("\\end{tikzpicture}\n");
        result
    }

    pub fn concrete_semantics(&self, value: u32) -> String {
        if self.parameter.is_some() {
            format!("{{\\tt {value:X}}}")
        } else {
            String::new()
        }
    }

    pub fn concrete_argument(&self, value: u32, decimal: bool) -> String {
        let mut result = String::new();
        if let Some(parameter) = &self.parameter {
            if decimal {
                result.push_str(&format!("{{\\tt {}}}", value));
            } else if parameter.size == 1 {
                result.push_str(&format!("{{\\tt {:02X}}}", value));
            } else if parameter.size == 2 {
                result.push_str(&format!("{{\\tt {:04X}}}", value));
            } else if parameter.size == 4 {
                result.push_str(&format!("{{\\tt {:08X}}}", value));
            }
        }
        result
    }

    pub fn concrete_byte_pattern(&self, value: u32) -> String {
        let mut result = String::new();
        if let Some(parameter) = &self.parameter {
            if parameter.size == 1 {
                result.push_str(&format!("{:02X} ", value));
            } else if parameter.size == 2 {
                result.push_str(&format!("{:04X} ", value));
            } else if parameter.size == 4 {
                result.push_str(&format!("{:08X} ", value));
            }
        }
        result.push_str(&format!("{{\\bfseries {:02X}}}", self.opcode));
        result
    }
}

pub const CST_0: BytecodeInstructionType = BytecodeInstructionType {
    name: r"cst\_0",
    operation: "$push(0)$",
    opcode: 0,
    parameter: Option::None,
};

pub const CST_1: BytecodeInstructionType = BytecodeInstructionType {
    name: r"cst\_1",
    operation: "$push(1)$",
    opcode: 1,
    parameter: Option::None,
};

pub const CST8: BytecodeInstructionType = BytecodeInstructionType {
    name: "cst8",
    operation: "$push(#0)$",
    opcode: 2,
    parameter: Option::Some(BytecodeParameter { name: "c", size: 1 }),
};

pub const CST: BytecodeInstructionType = BytecodeInstructionType {
    name: "cst",
    operation: "$push(#0)$",
    opcode: 3,
    parameter: Option::Some(BytecodeParameter { name: "c", size: 4 }),
};

pub const ADD: BytecodeInstructionType = BytecodeInstructionType {
    name: "add",
    operation: r"$y \leftarrow pop(), x \leftarrow pop(), push(x+y)$",
    opcode: 4,
    parameter: Option::None,
};

pub const SUB: BytecodeInstructionType = BytecodeInstructionType {
    name: "sub",
    operation: r"$y \leftarrow pop(), x \leftarrow pop(), push(x-y)$",
    opcode: 5,
    parameter: Option::None,
};

pub const MUL: BytecodeInstructionType = BytecodeInstructionType {
    name: "mul",
    operation: r"$y \leftarrow pop(), x \leftarrow pop(), push(x*y)$",
    opcode: 6,
    parameter: Option::None,
};

pub const DIV: BytecodeInstructionType = BytecodeInstructionType {
    name: "div",
    operation: r"$y \leftarrow pop(), x \leftarrow pop(), push(x/y)$",
    opcode: 7,
    parameter: Option::None,
};

pub const AND: BytecodeInstructionType = BytecodeInstructionType {
    name: "and",
    operation: r"$y \leftarrow pop(), x \leftarrow pop(), push(x \wedge y)$",
    opcode: 8,
    parameter: Option::None,
};

pub const OR: BytecodeInstructionType = BytecodeInstructionType {
    name: "or",
    operation: r"$y \leftarrow pop(), x \leftarrow pop(), push(x \vee y)$",
    opcode: 9,
    parameter: Option::None,
};

pub const LSL: BytecodeInstructionType = BytecodeInstructionType {
    name: "lsl",
    operation: r"$y \leftarrow pop(), x \leftarrow pop(), push(x \ll y)$",
    opcode: 10,
    parameter: Option::None,
};

pub const LSR: BytecodeInstructionType = BytecodeInstructionType {
    name: "lsr",
    operation: r"$y \leftarrow pop(), x \leftarrow pop(), push(x \gg y)$",
    opcode: 11,
    parameter: Option::None,
};

pub const IFLT: BytecodeInstructionType = BytecodeInstructionType {
    name: "iflt",
    operation: r"$y \leftarrow pop(), x \leftarrow pop(),$ jump to $#0$ if $x < y$",
    opcode: 12,
    parameter: Option::Some(BytecodeParameter { name: "c", size: 2 }),
};

pub const IFEQ: BytecodeInstructionType = BytecodeInstructionType {
    name: "ifeq",
    operation: r"$y \leftarrow pop(), x \leftarrow pop(),$ jump to $#0$ if $x = y$",
    opcode: 13,
    parameter: Option::Some(BytecodeParameter { name: "c", size: 2 }),
};

pub const IFGT: BytecodeInstructionType = BytecodeInstructionType {
    name: "ifgt",
    operation: r"$y \leftarrow pop(), x \leftarrow pop(),$ jump to $#0$ if $x > y$",
    opcode: 14,
    parameter: Option::Some(BytecodeParameter { name: "c", size: 2 }),
};

pub const IFLE: BytecodeInstructionType = BytecodeInstructionType {
    name: "ifle",
    operation: r"$y \leftarrow pop(), x \leftarrow pop(),$ jump to $#0$ if $x \le y$",
    opcode: 15,
    parameter: Option::Some(BytecodeParameter { name: "c", size: 2 }),
};

pub const IFNE: BytecodeInstructionType = BytecodeInstructionType {
    name: "ifne",
    operation: r"$y \leftarrow pop(), x \leftarrow pop(),$ jump to $#0$ if $x \ne y$",
    opcode: 16,
    parameter: Option::Some(BytecodeParameter { name: "c", size: 2 }),
};

pub const IFGE: BytecodeInstructionType = BytecodeInstructionType {
    name: "ifge",
    operation: r"$y \leftarrow pop(), x \leftarrow pop(),$ jump to $#0$ if $x \ge y$",
    opcode: 17,
    parameter: Option::Some(BytecodeParameter { name: "c", size: 2 }),
};

pub const GOTO: BytecodeInstructionType = BytecodeInstructionType {
    name: "goto",
    operation: r"jump to $#0$",
    opcode: 18,
    parameter: Option::Some(BytecodeParameter { name: "c", size: 2 }),
};

pub const LOAD: BytecodeInstructionType = BytecodeInstructionType {
    name: "load",
    operation: r"$x \leftarrow pop(), push(\mathrm{mem32}[x])$",
    opcode: 19,
    parameter: Option::None,
};

pub const STORE: BytecodeInstructionType = BytecodeInstructionType {
    name: "store",
    operation: r"$v \leftarrow pop(), x \leftarrow pop(), \mathrm{mem32}[x] \leftarrow v$",
    opcode: 20,
    parameter: Option::None,
};

pub const PTR: BytecodeInstructionType = BytecodeInstructionType {
    name: "ptr",
    operation: r"$push(\mathrm{address\ of\ frame}[#0])$",
    opcode: 21,
    parameter: Option::Some(BytecodeParameter { name: "i", size: 1 }),
};

pub const GET: BytecodeInstructionType = BytecodeInstructionType {
    name: "get",
    operation: r"$push(\mathrm{frame}[#0])$",
    opcode: 22,
    parameter: Option::Some(BytecodeParameter { name: "i", size: 1 }),
};

pub const SET: BytecodeInstructionType = BytecodeInstructionType {
    name: "set",
    operation: r"$\mathrm{frame}[#0] \leftarrow pop()$",
    opcode: 23,
    parameter: Option::Some(BytecodeParameter { name: "i", size: 1 }),
};

pub const POPI: BytecodeInstructionType = BytecodeInstructionType {
    name: "pop",
    operation: r"$pop()$",
    opcode: 24,
    parameter: Option::None,
};

pub const FN: BytecodeInstructionType = BytecodeInstructionType {
    name: "fn",
    operation: r"$push\_frame(#0)$",
    opcode: 25,
    parameter: Option::Some(BytecodeParameter { name: "n", size: 1 }),
};

pub const CALL: BytecodeInstructionType = BytecodeInstructionType {
    name: "call",
    operation: r"call function at $C0000_{16}+#0$",
    opcode: 26,
    parameter: Option::Some(BytecodeParameter { name: "c", size: 2 }),
};

pub const CALLR: BytecodeInstructionType = BytecodeInstructionType {
    name: "callr",
    operation: r"call function at $a-#0$",
    opcode: 27,
    parameter: Option::Some(BytecodeParameter { name: "c", size: 2 }),
};

pub const CALLD: BytecodeInstructionType = BytecodeInstructionType {
    name: "calld",
    operation: r"$x \leftarrow pop(),$ call function at $x$",
    opcode: 28,
    parameter: Option::None,
};

pub const RETURN: BytecodeInstructionType = BytecodeInstructionType {
    name: "ret",
    operation: r"$pop\_frame()$",
    opcode: 29,
    parameter: Option::None,
};

pub const RETURN_VALUE: BytecodeInstructionType = BytecodeInstructionType {
    name: "retv",
    operation: r"$x \leftarrow pop(), pop\_frame(), push(x)$",
    opcode: 30,
    parameter: Option::None,
};

pub const BLX: BytecodeInstructionType = BytecodeInstructionType {
    name: "blx",
    operation: r"$x \leftarrow pop(), \mathrm{BLX}\ x$",
    opcode: 31,
    parameter: Option::None,
};

pub const U8_DATA: BytecodeInstructionType = BytecodeInstructionType {
    name: "u8",
    operation: "",
    opcode: 255,
    parameter: Option::None,
};
