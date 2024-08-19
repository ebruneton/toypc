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

use std::{
    collections::HashMap,
    fs::{create_dir_all, File},
    io::Write,
    path::Path,
};

use super::util::{check_line_lengths, DiffProducer, MAX_LINE_LENGTH};

struct Line {
    offset: u32,
    source: String,
    toy0: String,
    toy1: String,
    latex: String,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Token {
    Empty,
    Const,
    ConstName,
    ConstValue,
    Static,
    StaticName,
    StaticValue,
    Fn,
    FnName,
    FnParameter,
    Let,
    LetName,
    Label,
    Opcode0,
    Opcode1,
    Opcode2,
    Opcode4,
    Argument,
    SymbolicConst,
    SymbolicConstArgument,
}

trait Consumer {
    fn space(&mut self, _: &str) {}
    fn token(&mut self, token: Token, value: &str);
}

impl Line {
    fn new(offset: u32, source: &str) -> Self {
        Self {
            offset,
            source: String::from(source),
            toy0: String::new(),
            toy1: String::new(),
            latex: String::new(),
        }
    }

    fn parse(&self, previous_token: Token, consumer: &mut dyn Consumer) -> Token {
        let chars = self.source.as_bytes();
        let len = chars.len();
        let mut start = 0;
        let mut i = 0;
        let mut token = previous_token;
        while i < len {
            while i < len && (chars[i] == b' ' || chars[i] == b'\t') {
                i += 1;
            }
            if i > start {
                consumer.space(&String::from_utf8(chars[start..i].to_vec()).unwrap());
            }
            start = i;
            while i < len && chars[i] != b' ' && chars[i] != b'\t' {
                i += 1;
            }
            if i > start {
                let value = String::from_utf8(chars[start..i].to_vec()).unwrap();
                token = Self::next_token(token, value.as_str());
                consumer.token(token, &value);
                start = i;
            }
        }
        match token {
            Token::FnName | Token::FnParameter => Token::Empty,
            _ => token,
        }
    }

    fn next_token(token: Token, value: &str) -> Token {
        match token {
            Token::Empty
            | Token::ConstValue
            | Token::LetName
            | Token::Label
            | Token::Opcode0
            | Token::Argument
            | Token::SymbolicConstArgument => match value {
                "const" => Token::Const,
                "static" => Token::Static,
                "fn" => Token::Fn,
                "let" => Token::Let,
                "cst_0" | "cst_1" | "add" | "sub" | "mul" | "div" | "and" | "or" | "lsl"
                | "lsr" | "load" | "store" | "pop" | "calld" | "ret" | "retv" | "blx" => {
                    Token::Opcode0
                }
                "cst8" | "ptr" | "get" | "set" => Token::Opcode1,
                "iflt" | "ifeq" | "ifgt" | "ifle" | "ifne" | "ifge" | "goto" | "call" | "callr" => {
                    Token::Opcode2
                }
                "cst" => Token::Opcode4,
                "scst" => Token::SymbolicConst,
                _ => {
                    if value.starts_with(':') {
                        Token::Label
                    } else {
                        panic!("Syntax error: '{value}'")
                    }
                }
            },
            Token::Const => Token::ConstName,
            Token::ConstName => Token::ConstValue,
            Token::Static => Token::StaticName,
            Token::StaticName => Token::StaticValue,
            Token::StaticValue => match value {
                "const" => Token::Const,
                "static" => Token::Static,
                "fn" => Token::Fn,
                _ => Token::StaticValue,
            },
            Token::Fn => Token::FnName,
            Token::FnName | Token::FnParameter => Token::FnParameter,
            Token::Let => Token::LetName,
            Token::Opcode1 | Token::Opcode2 | Token::Opcode4 => Token::Argument,
            Token::SymbolicConst => Token::SymbolicConstArgument,
        }
    }
}

static ARITY: &str = "_arity_";

enum Kind {
    Const,
    Static,
    Fn,
    Label,
    Variable,
}

struct Value {
    kind: Kind,
    value: u32,
}

impl Value {
    fn new(kind: Kind, value: u32) -> Self {
        Self { kind, value }
    }
}

struct Resolver {
    code_offset: u32,
    current_const: String,
    current_function: String,
    current_function_offset: u32,
    next_variable: u32,
    globals: HashMap<String, Value>,
    locals: HashMap<String, HashMap<String, Value>>,
}

impl Resolver {
    fn new(flash_dst: u32) -> Self {
        Self {
            code_offset: flash_dst + 4,
            current_const: String::new(),
            current_function: String::new(),
            current_function_offset: 0,
            next_variable: 0,
            globals: HashMap::new(),
            locals: HashMap::new(),
        }
    }
}

impl Consumer for Resolver {
    fn token(&mut self, state: Token, value: &str) {
        match state {
            Token::ConstName => {
                self.current_const = String::from(value);
            }
            Token::ConstValue => {
                self.globals.insert(
                    self.current_const.clone(),
                    Value::new(Kind::Const, value.parse().unwrap()),
                );
            }
            Token::StaticName => {
                self.globals.insert(
                    String::from(value),
                    Value::new(Kind::Static, self.code_offset + 0xC0000),
                );
            }
            Token::FnName => {
                if !value.ends_with(';') {
                    let mut locals = HashMap::new();
                    locals.insert(String::from(ARITY), Value::new(Kind::Variable, 0));
                    self.current_function = String::from(value);
                    self.current_function_offset = self.code_offset;
                    self.next_variable = 4;
                    self.globals
                        .insert(String::from(value), Value::new(Kind::Fn, self.code_offset));
                    self.locals.insert(String::from(value), locals);
                }
            }
            Token::FnParameter => {
                let locals = self.locals.get_mut(&self.current_function).unwrap();
                let arity = locals.get(&String::from(ARITY)).unwrap().value;
                locals.insert(String::from(value), Value::new(Kind::Variable, arity));
                locals.insert(String::from(ARITY), Value::new(Kind::Variable, arity + 1));
                self.next_variable += 1;
            }
            Token::LetName => {
                let locals = self.locals.get_mut(&self.current_function).unwrap();
                locals.insert(
                    String::from(value),
                    Value::new(Kind::Variable, self.next_variable),
                );
                self.next_variable += 1;
            }
            Token::Label => {
                self.locals.get_mut(&self.current_function).unwrap().insert(
                    String::from(value.strip_prefix(':').unwrap()),
                    Value::new(Kind::Label, self.code_offset - self.current_function_offset),
                );
            }
            _ => (),
        }
        let size = match state {
            Token::StaticValue | Token::Opcode0 => 1,
            Token::Opcode1 => 2,
            Token::Opcode2 => 3,
            Token::Opcode4 => 5,
            Token::SymbolicConstArgument => {
                let x = self.globals.get(value).unwrap().value;
                match x {
                    0 | 1 => 1,
                    _ => panic!("Not yet implemented"),
                }
            }
            Token::FnName => {
                if value.ends_with(';') {
                    0
                } else {
                    2
                }
            }
            _ => 0,
        };
        self.code_offset += size;
    }
}

struct Converter<'a> {
    resolver: &'a Resolver,
    current_function: String,
    current_arity: u32,
    current_parameter: u32,
    toy0: String,
    toy1: String,
    latex: String,
}

impl<'a> Converter<'a> {
    fn new(resolver: &'a Resolver) -> Self {
        Self {
            resolver,
            current_function: String::new(),
            current_arity: 0,
            current_parameter: 0,
            toy0: String::new(),
            toy1: String::new(),
            latex: String::new(),
        }
    }

    fn clear(&mut self) {
        self.toy0.clear();
        self.toy1.clear();
        self.latex.clear();
    }
}

impl<'a> Consumer for Converter<'a> {
    fn space(&mut self, value: &str) {
        if !self.toy0.ends_with('\t') && !self.toy0.ends_with(' ') {
            self.toy0.push_str(&value.replace("  ", "\t"));
        }
        if !self.toy1.ends_with('\t') && !self.toy1.ends_with(' ') {
            self.toy1.push_str(&value.replace("  ", "\t"));
        }
        self.latex.push_str(&value.replace('\t', "  "));
    }

    fn token(&mut self, token: Token, value: &str) {
        match token {
            Token::Empty => panic!("Internal error"),
            Token::Const | Token::ConstName | Token::ConstValue | Token::SymbolicConst => (),
            Token::Static => {
                self.toy1.push_str(value);
            }
            Token::StaticName => {
                self.toy1.push_str(value);
                self.latex.pop();
                self.latex
                    .push_str(&format!("\\ToyInsert{{static {}}}", value));
            }
            Token::StaticValue => {
                if self.toy0 == " " {
                    self.toy0.clear();
                }
                if value.len() == 3 && value.starts_with('\'') && value.ends_with('\'') {
                    let v = format!("{}", value.chars().nth(1).unwrap() as u32);
                    self.toy0.push_str("d ");
                    self.toy0.push_str(&v);
                    self.toy1.push_str(&v);
                    self.latex.push_str("\\ToyDelete{d }");
                    self.latex
                        .push_str(&format!("\\ToyComment{{{}}}{}", value, v));
                } else {
                    self.toy0.push_str("d ");
                    self.toy0.push_str(value);
                    self.toy1.push_str(value);
                    self.latex.push_str("\\ToyDelete{d }");
                    self.latex.push_str(value);
                }
            }
            Token::FnName => {
                if value.ends_with(';') {
                    self.toy1.pop();
                    self.toy1.push_str("fn ");
                    self.toy1.push_str(value);
                    self.latex.pop();
                    self.latex.push_str(&format!("\\ToyInsert{{fn {}}}", value));
                } else {
                    self.current_function = String::from(value);
                    self.current_arity = self
                        .resolver
                        .locals
                        .get(value)
                        .unwrap()
                        .get(ARITY)
                        .unwrap()
                        .value;
                    self.current_parameter = 0;
                    self.toy0.pop();
                    self.toy0.push_str("fn ");
                    self.toy0.push_str(&format!("{}", self.current_arity));
                    self.toy1.pop();
                    self.toy1.push_str("fn ");
                    self.toy1.push_str(value);
                    self.toy1.push(' ');
                    self.toy1.push_str(&format!("{}", self.current_arity));
                    self.latex.pop();
                    self.latex.push_str("fn ");
                    self.latex.push_str(&format!("\\ToyInsert{{{} }}", value));
                    self.latex.push_str(&format!("{}", self.current_arity));
                }
            }
            Token::FnParameter => {
                if self.current_parameter == 0 {
                    self.latex.push_str("\\ToyComment{(}");
                }
                if self.current_parameter > 0 {
                    self.latex.pop();
                    self.latex.push_str("\\ToyComment{,} ");
                }
                self.latex.push_str(&format!("\\ToyParam{{{}}}", value));
                self.current_parameter += 1;
                if self.current_parameter == self.current_arity {
                    self.latex.push_str("\\ToyComment{)}");
                }
            }
            Token::LetName => {
                self.latex.pop();
                self.latex.push_str(&format!("\\ToyLet{{{}}}", value));
            }
            Token::Label => {
                self.toy0.push_str("<skip>");
                self.toy1.push_str(value);
                self.latex.push_str(&format!("\\ToyInsert{{{}}}", value));
            }
            Token::Opcode0 | Token::Opcode1 | Token::Opcode2 | Token::Opcode4 => {
                self.toy0.push_str(value);
                self.toy1.push_str(value);
                self.latex.push_str(value);
            }
            Token::Argument => {
                if value.parse::<u32>().is_ok() {
                    self.toy0.push_str(value);
                    self.toy1.push_str(value);
                    self.latex.push_str(value);
                } else {
                    let locals = self.resolver.locals.get(&self.current_function).unwrap();
                    let v = locals
                        .get(value)
                        .unwrap_or_else(|| self.resolver.globals.get(value).unwrap());
                    self.toy0.push_str(&format!("{}", v.value));
                    match v.kind {
                        Kind::Static | Kind::Fn | Kind::Label => {
                            self.toy1.push_str(value);
                            self.latex.push_str(&format!(
                                "\\ToyDelete{{{}}}\\ToyInsert{{{}}}",
                                v.value, value
                            ));
                        }
                        Kind::Const => {
                            self.toy1.push_str(&format!("{}", v.value));
                            self.latex
                                .push_str(&format!("\\ToyConst{{{}=}}{}", value, v.value));
                        }
                        Kind::Variable => {
                            self.toy1.push_str(&format!("{}", v.value));
                            self.latex
                                .push_str(&format!("\\ToyVar{{{}:}}{}", value, v.value));
                        }
                    }
                }
            }
            Token::SymbolicConstArgument => {
                let x = self.resolver.globals.get(value).unwrap().value;
                match x {
                    0 => {
                        self.toy0.push_str("cst_0");
                        self.toy1.push_str("cst_0");
                        self.latex.pop();
                        self.latex
                            .push_str(&format!("\\ToyComment{{{}=}}cst_0", value));
                    }
                    1 => {
                        self.toy0.push_str("cst_1");
                        self.toy1.push_str("cst_1");
                        self.latex.pop();
                        self.latex
                            .push_str(&format!("\\ToyComment{{{}=}}cst_1", value));
                    }
                    _ => panic!("Not yet implemented"),
                }
            }
            Token::Fn | Token::Let => (),
        }
    }
}

struct Part {
    filename: String,
    start_line: usize,
    end_line: usize,
}

pub struct Transpiler1 {
    last_token: Token,
    resolver: Resolver,
    lines: Vec<Line>,
    parts: Vec<Part>,
    closed: bool,
}

impl Transpiler1 {
    pub fn new(base: u32) -> Self {
        Self {
            last_token: Token::Empty,
            resolver: Resolver::new(base),
            lines: Vec::new(),
            parts: Vec::new(),
            closed: false,
        }
    }

    pub fn add(&mut self, line: &str) {
        let l = Line::new(self.resolver.code_offset, line);
        self.last_token = l.parse(self.last_token, &mut self.resolver);
        self.lines.push(l);
    }

    pub fn write(&mut self, filename: &str) {
        let start_line = self.parts.last().map_or(0, |p| p.end_line);
        let end_line = self.lines.len();
        self.parts.push(Part {
            filename: String::from(filename),
            start_line,
            end_line,
        });
    }

    fn write_latex(
        &self,
        filename: &str,
        start: usize,
        end: usize,
        current_function_offset: &mut u32,
    ) -> std::io::Result<()> {
        let mut output = File::create(filename)?;
        writeln!(&mut output, "\\begin{{Code}}")?;
        let mut previous_line_empty = true;
        for line in &self.lines[start..end] {
            if line.latex.trim().is_empty() {
                if !previous_line_empty {
                    output.write_all(
                        Self::wrapped_latex_line(line, current_function_offset).as_bytes(),
                    )?;
                    writeln!(&mut output)?;
                }
                previous_line_empty = true;
            } else {
                output.write_all(
                    Self::wrapped_latex_line(line, current_function_offset).as_bytes(),
                )?;
                writeln!(&mut output)?;
                previous_line_empty = false;
            }
        }
        writeln!(&mut output, "\\end{{Code}}")?;
        Ok(())
    }

    fn wrapped_latex_line(line: &Line, current_function_offset: &mut u32) -> String {
        DiffProducer::wrap_line(
            &Self::latex_line(line, current_function_offset),
            MAX_LINE_LENGTH,
        )
    }

    fn latex_line(line: &Line, current_function_offset: &mut u32) -> String {
        if line.latex.is_empty() {
            String::new()
        } else if line.toy0.starts_with("fn") {
            *current_function_offset = line.offset;
            format!("\\ToyComment{{{:4}}} {}", line.offset, line.latex)
        } else if line.toy1.starts_with("static") {
            *current_function_offset = 0;
            format!("\\ToyComment{{{:4}}} {}", line.offset, line.latex)
        } else if line.toy0.trim_end().is_empty() || *current_function_offset == 0 {
            format!("     {}", line.latex)
        } else {
            format!(
                "\\ToyComment{{{:+4}}} {}",
                line.offset - *current_function_offset,
                line.latex
            )
        }
    }

    pub fn write_toy0(&mut self, filename: &str) -> std::io::Result<()> {
        self.close()?;
        let mut previous_line_empty = true;
        let mut out = String::new();
        for line in &self.lines {
            if line.toy0.trim().is_empty() || line.toy0 == "<skip>" {
                if !previous_line_empty && line.toy0 != "<skip>" {
                    Self::append(line.toy0.trim_end(), &mut out);
                }
                previous_line_empty = true;
            } else {
                if (line.toy0.starts_with("fn") || line.toy1.starts_with("static"))
                    && !previous_line_empty
                {
                    Self::append("", &mut out);
                }
                Self::append(line.toy0.trim_end(), &mut out);
                previous_line_empty = false;
            }
        }
        check_line_lengths(&out);
        create_dir_all(Path::new(filename).parent().unwrap())?;
        let mut output = File::create(filename)?;
        output.write_all(out.as_bytes())
    }

    pub fn write_toy1(&mut self, filename: &str) -> std::io::Result<()> {
        self.close()?;
        let mut previous_line_empty = true;
        let mut out = String::new();
        for line in &self.lines {
            if line.toy1.trim().is_empty() {
                if !previous_line_empty {
                    Self::append(line.toy1.trim_end(), &mut out);
                }
                previous_line_empty = true;
            } else {
                if (line.toy1.starts_with("fn") || line.toy1.starts_with("static"))
                    && !previous_line_empty
                {
                    Self::append("", &mut out);
                }
                Self::append(line.toy1.trim_end(), &mut out);
                previous_line_empty = false;
            }
        }
        check_line_lengths(&out);
        create_dir_all(Path::new(filename).parent().unwrap())?;
        let mut output = File::create(filename)?;
        output.write_all(out.as_bytes())
    }

    fn append(line: &str, out: &mut String) {
        if !line.is_empty()
            && !line.starts_with("fn")
            && !line.starts_with("static")
            && !line.starts_with([' ', '\t', ':'])
            && !out.is_empty()
        {
            out.pop();
            out.push(' ');
        }
        out.push_str(line);
        out.push('\n');
    }

    fn close(&mut self) -> std::io::Result<()> {
        if self.closed {
            return Ok(());
        }
        let mut last_token = Token::Empty;
        let mut converter = Converter::new(&self.resolver);
        for line in &mut self.lines {
            last_token = line.parse(last_token, &mut converter);
            line.toy0 = converter.toy0.clone();
            line.toy1 = converter.toy1.clone();
            line.latex = converter.latex.clone();
            converter.clear();
        }
        let mut current_function_offset = 0;
        for part in &self.parts {
            self.write_latex(
                &part.filename,
                part.start_line,
                part.end_line,
                &mut current_function_offset,
            )?;
        }
        self.closed = true;
        Ok(())
    }
}

impl Drop for Transpiler1 {
    fn drop(&mut self) {
        self.close().unwrap();
    }
}
