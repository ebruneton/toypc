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

pub enum T8Instruction {
    Ldr(u8),
    Str(u8),
    Add(u8),
    Sub(u8),
    Jump(u8),
    IfZ(u8),
    IfC(u8),
    In,
    Out,
    Data(u8, &'static str),
}

impl T8Instruction {
    pub fn name(&self) -> &'static str {
        match &self {
            T8Instruction::Ldr(_address) => "LDR",
            T8Instruction::Str(_address) => "STR",
            T8Instruction::Add(_address) => "ADD",
            T8Instruction::Sub(_address) => "SUB",
            T8Instruction::Jump(_address) => "JMP",
            T8Instruction::IfZ(_address) => "IFZ",
            T8Instruction::IfC(_address) => "IFC",
            T8Instruction::In => "IN",
            T8Instruction::Out => "OUT",
            T8Instruction::Data(_value, _comment) => "(data)",
        }
    }

    pub fn definition(&self) -> String {
        let semantics = match &self {
            T8Instruction::Ldr(_address) => "$\\mathrm{R0} \\leftarrow \\mathrm{mem8}[a]$",
            T8Instruction::Str(_address) => "$\\mathrm{R0} \\rightarrow \\mathrm{mem8}[a]$",
            T8Instruction::Add(_address) => {
                "$\\mathrm{R0} \\leftarrow \\mathrm{R0} + \\mathrm{mem8}[a]$"
            }
            T8Instruction::Sub(_address) => {
                "$\\mathrm{R0} \\leftarrow \\mathrm{R0} - \\mathrm{mem8}[a]$"
            }
            T8Instruction::Jump(_address) => "jump to $a$",
            T8Instruction::IfZ(_address) => "if $\\mathrm{R0} = 0$ then jump to $a$",
            T8Instruction::IfC(_address) => "if carry $\\ne 0$ then jump to $a$",
            T8Instruction::In => "$\\mathrm{R0} \\leftarrow$ input",
            T8Instruction::Out => "$\\mathrm{R0} \\rightarrow$ output",
            T8Instruction::Data(_value, comment) => comment,
        };
        format!(
            "\\makebox[3.5em][l]{{\\sffamily\\bfseries {}}} \\makebox[13em][l]{{{}}} \\hfill {}",
            self.name(),
            semantics,
            self.bit_pattern()
        )
    }

    pub fn encoding(&self) -> u8 {
        match &self {
            T8Instruction::Ldr(address) => 0b001_00000 | address,
            T8Instruction::Str(address) => 0b000_00000 | address,
            T8Instruction::Add(address) => 0b010_00000 | address,
            T8Instruction::Sub(address) => 0b011_00000 | address,
            T8Instruction::Jump(address) => 0b100_00000 | address,
            T8Instruction::IfZ(address) => 0b101_00000 | address,
            T8Instruction::IfC(address) => 0b110_00000 | address,
            T8Instruction::In => 0b1110_0000,
            T8Instruction::Out => 0b1111_0000,
            T8Instruction::Data(value, _comment) => *value,
        }
    }

    pub fn bit_pattern(&self) -> String {
        let mut result = String::new();
        result.push_str("\\begin{tikzpicture}[x=0.75ex,y=1ex,baseline=0.7ex]\n");
        result.push_str("\\useasboundingbox (0,0) rectangle +(24,3) ;\n");
        let encoding = self.encoding();
        let is_in_out = (encoding & 0b111_00000) == 0b111_00000;
        let opcode_size = if is_in_out { 4 } else { 3 };
        for i in 0..8 {
            if i == opcode_size {
                result.push_str(&format!("\\draw ({},0) -- +(0,3) ;\n", i * 3));
            } else if i > 0 {
                result.push_str(&format!("\\draw[gray] ({},0) -- +(0,0.7) ;\n", i * 3));
            }
            if i < opcode_size {
                result.push_str(&format!(
                    "\\node[anchor=base] at ({}.5,0.7) {{{}}} ;\n",
                    i * 3 + 1,
                    (encoding >> (7 - i)) & 1
                ));
            }
        }
        result.push_str("\\draw (0,0) rectangle +(24,3) ;\n");
        if !is_in_out {
            result.push_str("\\node[anchor=base] at (16.5,0.8) {$a$} ;\n");
        }
        result.push_str("\\end{tikzpicture}\n");
        result
    }

    pub fn concrete_semantics(&self) -> String {
        match &self {
            T8Instruction::Ldr(address) => {
                format!("$\\mathrm{{R0}} \\leftarrow \\mathrm{{mem8}}[\\mathit{{{address}}}]$")
            }
            T8Instruction::Str(address) => {
                format!("$\\mathrm{{R0}} \\rightarrow \\mathrm{{mem8}}[\\mathit{{{address}}}]$")
            }
            T8Instruction::Add(address) => {
                format!(
                    "$\\mathrm{{R0}} \\leftarrow \\mathrm{{R0}} +\
                 \\mathrm{{mem8}}[\\mathit{{{address}}}]$"
                )
            }
            T8Instruction::Sub(address) => {
                format!(
                    "$\\mathrm{{R0}} \\leftarrow \\mathrm{{R0}} -\
                 \\mathrm{{mem8}}[\\mathit{{{address}}}]$"
                )
            }
            T8Instruction::Jump(address) => format!("jump to $\\mathit{{{address}}}$"),
            T8Instruction::IfZ(address) => {
                format!("if $\\mathrm{{R0}} = 0$ then jump to $\\mathit{{{address}}}$")
            }
            T8Instruction::IfC(address) => {
                format!("if carry $\\ne 0$ then jump to $\\mathit{{{address}}}$")
            }
            T8Instruction::In => String::from("$\\mathrm{R0} \\leftarrow$ input"),
            T8Instruction::Out => String::from("$\\mathrm{R0} \\rightarrow$ output"),
            T8Instruction::Data(_value, comment) => String::from(*comment),
        }
    }

    pub fn concrete_bit_pattern(&self) -> String {
        let mut result = String::new();
        result.push_str("\\begin{tikzpicture}[x=0.75ex,y=1ex,baseline=0.7ex]\n");
        result.push_str("\\useasboundingbox (0,0) rectangle +(24,3) ;\n");
        let encoding = self.encoding();
        let is_in_out = (encoding & 0b111_00000) == 0b111_00000;
        let opcode_size = if let T8Instruction::Data(_value, _comment) = self {
            8
        } else if is_in_out {
            4
        } else {
            3
        };
        for i in 0..8 {
            if i == opcode_size {
                result.push_str(&format!("\\draw ({},0) -- +(0,3) ;\n", i * 3));
            } else if i > 0 {
                result.push_str(&format!("\\draw[gray] ({},0) -- +(0,0.7) ;\n", i * 3));
            }
            if i < opcode_size {
                result.push_str(&format!(
                    "\\node[anchor=base] at ({}.5,0.7) {{{}}} ;\n",
                    i * 3 + 1,
                    (encoding >> (7 - i)) & 1
                ));
            } else {
                result.push_str(&format!(
                    "\\node[anchor=base] at ({}.5,0.7) {{$\\mathit{{{}}}$}} ;\n",
                    i * 3 + 1,
                    (encoding >> (7 - i)) & 1
                ));
            }
        }
        result.push_str("\\draw (0,0) rectangle +(24,3) ;\n");
        result.push_str("\\end{tikzpicture}\n");
        result
    }
}

#[derive(Default)]
pub struct T8Program {
    instructions: Vec<T8Instruction>,
}

impl T8Program {
    pub fn ldr(&mut self, address: u8) {
        self.instructions.push(T8Instruction::Ldr(address));
    }
    pub fn str(&mut self, address: u8) {
        self.instructions.push(T8Instruction::Str(address));
    }
    pub fn add(&mut self, address: u8) {
        self.instructions.push(T8Instruction::Add(address));
    }
    pub fn sub(&mut self, address: u8) {
        self.instructions.push(T8Instruction::Sub(address));
    }
    pub fn jump(&mut self, address: u8) {
        self.instructions.push(T8Instruction::Jump(address));
    }
    pub fn if_zero(&mut self, address: u8) {
        self.instructions.push(T8Instruction::IfZ(address));
    }
    pub fn if_carry(&mut self, address: u8) {
        self.instructions.push(T8Instruction::IfC(address));
    }
    pub fn input(&mut self) {
        self.instructions.push(T8Instruction::In);
    }
    pub fn output(&mut self) {
        self.instructions.push(T8Instruction::Out);
    }
    pub fn data(&mut self, value: u8, comment: &'static str) {
        self.instructions.push(T8Instruction::Data(value, comment));
    }

    pub fn get_listing(&self) -> String {
        self.get_listing_with_offset(0)
    }

    pub fn get_listing_with_offset(&self, offset: usize) -> String {
        let mut result = String::new();
        result.push_str(r"\vspace{0.4\baselineskip}\noindent ");
        for (i, insn) in self.instructions.iter().enumerate() {
            result.push_str(&format!("\\makebox[3em][l]{{\\arm{{{}}}}}", insn.name()));
            result.push_str(insn.concrete_semantics().as_str());
            result.push_str("\\hfill");
            result.push_str(insn.concrete_bit_pattern().as_str());
            result.push_str(&format!(
                "\\makebox[2.5em][r]{{\\tt\\color{{gray}} {}}}",
                i + offset
            ));
            result.push_str("\\\\\n");
        }
        result.pop();
        result.push_str("[-0.5\\baselineskip]\n");
        result
    }

    pub fn get_machine_code(&self) -> Vec<u8> {
        let mut result = Vec::new();
        for insn in &self.instructions {
            result.push(insn.encoding());
        }
        result
    }
}
