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

use std::{collections::HashMap, fs::File, io::Write, str::from_utf8};

use super::util::{write_toy_file, DiffProducer};

static TC_CHAR_TYPES: [u8; 256] = [
    1, 1, 1, 1, 1, 1, 1, 1, 1, 32, 32, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 32, 1, 1, 1, 1, 1, 8, 39, 40, 41, 6, 4, 44, 5, 1, 7, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 58, 59,
    1, 1, 1, 1, 1, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 1,
    1, 1, 1, 3, 1, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
    123, 9, 125, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1,
];

static ARG_SIZES: [u8; 32] = [
    0, 0, 1, 4, 0, 0, 0, 0, 0, 0, 0, 0, 2, 2, 2, 2, 2, 2, 2, 0, 0, 1, 1, 1, 0, 0, 2, 0, 0, 0, 0, 0,
];

const TC_INTEGER: u8 = 2;
const TC_QUOTED_CHAR: u8 = 10;
const TC_IDENTIFIER: u8 = 3;
const TC_ADD: u8 = 4;
const TC_SUB: u8 = 5;
const TC_MUL: u8 = 6;
const TC_DIV: u8 = 7;
const TC_BIT_AND: u8 = 8;
const TC_BIT_OR: u8 = 9;
const TC_FN: u8 = b'f';
const TC_LET: u8 = b'l';
const TC_CONST: u8 = b'c';
const TC_STATIC: u8 = b's';

#[derive(Clone, PartialEq)]
enum IntegerKind {
    Const,
    Static,
    Expression,
}

#[derive(Clone, PartialEq)]
enum SymbolKind {
    Constant,
    Static,
    Variable,
}

#[derive(Clone)]
struct Symbol {
    kind: SymbolKind,
    value: u32,
}

pub struct Transpiler2 {
    keywords: HashMap<&'static str, u8>,
    source: Vec<u8>,
    src: usize,
    next_char: u8,
    next_char_type: u8,
    next_token: u8,
    next_token_data: u32,
    next_token_length: u32,
    symbols: HashMap<String, Symbol>,
    old_spaces: String,
    new_spaces: String,
    diff_producer: DiffProducer,
    parts: Vec<String>,
    closed: bool,
}

impl Transpiler2 {
    pub fn new() -> Self {
        Self {
            keywords: Self::fill_keywords(),
            source: Vec::new(),
            src: 0,
            next_char: 0,
            next_char_type: 0,
            next_token: 0,
            next_token_data: 0,
            next_token_length: 0,
            symbols: HashMap::new(),
            old_spaces: String::new(),
            new_spaces: String::new(),
            diff_producer: DiffProducer::default(),
            parts: Vec::new(),
            closed: false,
        }
    }

    fn fill_keywords() -> HashMap<&'static str, u8> {
        let mut keywords = HashMap::new();
        keywords.insert("iflt", 140);
        keywords.insert("ifeq", 141);
        keywords.insert("ifgt", 142);
        keywords.insert("ifle", 143);
        keywords.insert("ifne", 144);
        keywords.insert("ifge", 145);
        keywords.insert("goto", 146);
        keywords.insert("load", 147);
        keywords.insert("store", 148);
        keywords.insert("set", 151);
        keywords.insert("pop", 152);
        keywords.insert("ret", 157);
        keywords.insert("retv", 158);
        keywords.insert("fn", b'f');
        keywords.insert("let", b'l');
        keywords.insert("const", b'c');
        keywords.insert("static", b's');
        keywords
    }

    pub fn add(&mut self, line: &str) {
        self.source.extend(line.as_bytes());
        self.source.push(b'\n');
    }

    pub fn write(&mut self, filename: &str) {
        self.source
            .extend(format!("#{}", self.parts.len()).as_bytes());
        self.parts.push(String::from(filename));
    }

    fn get_next_char(&mut self) {
        self.next_char = self.source[self.src];
        self.next_char_type = TC_CHAR_TYPES[self.next_char as usize];
    }

    fn read_char(&mut self) -> u8 {
        if self.src >= self.source.len() {
            panic!("Unexpected end of input");
        }
        self.src += 1;
        self.next_char = 0;
        self.next_char_type = 0;
        if self.src < self.source.len() {
            self.get_next_char();
        }
        self.next_char_type
    }

    fn read_integer(&mut self) -> u8 {
        let mut char_type = self.next_char_type;
        let mut v = 0;
        while char_type == TC_INTEGER {
            v = v * 10 + (self.next_char - b'0') as u32;
            char_type = self.read_char();
        }
        self.next_token_data = v;
        TC_INTEGER
    }

    fn read_quoted_char(&mut self) -> u8 {
        self.read_char();
        let value = self.next_char;
        self.read_char();
        self.read_char();
        self.next_token_data = value as u32;
        TC_QUOTED_CHAR
    }

    fn read_identifier(&mut self) -> u8 {
        let start = self.src;
        let mut char_type = self.next_char_type;
        while char_type == TC_IDENTIFIER || char_type == TC_INTEGER {
            char_type = self.read_char();
        }
        let length = self.src - start;
        self.next_token_data = start as u32;
        self.next_token_length = length as u32;
        let range =
            self.next_token_data as usize..(self.next_token_data + self.next_token_length) as usize;
        *self
            .keywords
            .get(&from_utf8(&self.source[range.start..range.end]).unwrap())
            .unwrap_or(&TC_IDENTIFIER)
    }

    fn read_token(&mut self) {
        let mut char_type = self.next_char_type;
        let mut spaces = String::new();
        while char_type == b' ' || self.next_char == b'@' || self.next_char == b'#' {
            if self.next_char == b'#' {
                self.read_char();
                self.read_integer();
                char_type = self.next_char_type;
                let filename = self.parts[self.next_token_data as usize].clone();
                self.write_latex(&filename).unwrap();
            } else {
                spaces.push(self.next_char as char);
                char_type = self.read_char();
            }
        }
        self.old_spaces.push_str(&spaces.replace("  ", "\t"));
        self.new_spaces.push_str(&spaces.replace("  ", "\t"));
        let mut token = char_type;
        if char_type == TC_INTEGER {
            token = self.read_integer();
        } else if char_type == b'\'' {
            token = self.read_quoted_char();
        } else if char_type == TC_IDENTIFIER {
            token = self.read_identifier();
        } else if char_type != 0 {
            self.read_char();
        }
        self.next_token = token;
    }

    fn flush_old_spaces(&mut self) {
        for c in self.old_spaces.replace(' ', "").chars() {
            self.diff_producer.add_old(c);
        }
        self.old_spaces.clear();
    }

    fn add_old(&mut self, c: char) {
        self.flush_old_spaces();
        self.diff_producer.add_old(c);
    }

    fn add_old_str(&mut self, s: &str) {
        self.flush_old_spaces();
        let old = self.diff_producer.get_old();
        if !old.is_empty() && !old.ends_with([' ', '\t', '\n', ':']) {
            self.diff_producer.add_old(' ');
        }
        self.diff_producer.add_old_str(s);
    }

    fn add_old_value(&mut self, value: u32) {
        if value == 0 {
            self.add_old_str("cst_0");
        } else if value == 1 {
            self.add_old_str("cst_1");
        } else if value < 256 {
            self.add_old_str(&format!("cst8 {}", value));
        } else {
            self.add_old_str(&format!("cst {}", value));
        }
    }

    fn flush_new_spaces(&mut self) {
        self.diff_producer.add_new_str(&self.new_spaces);
        self.new_spaces.clear();
    }

    fn add_new(&mut self, c: char) {
        self.flush_new_spaces();
        self.diff_producer.add_new(c);
    }

    fn add_new_str(&mut self, s: &str) {
        self.flush_new_spaces();
        self.diff_producer.add_new_str(s);
    }

    fn add_both(&mut self, c: char) {
        self.add_old(c);
        self.add_new(c);
    }

    fn add_both_str(&mut self, s: &str) {
        self.add_old_str(s);
        self.add_new_str(s);
    }

    fn add_diff(&mut self) {
        self.diff_producer.add_diff();
    }

    fn parse_token(&mut self, token: u8) {
        if self.next_token != token {
            panic!("Unexpected token {}, expected {token}", self.next_token);
        }
        self.read_token();
    }

    fn parse_integer(&mut self, kind: IntegerKind) -> u32 {
        if self.next_token != TC_INTEGER && self.next_token != TC_QUOTED_CHAR {
            panic!("Expected integer, got {}", self.next_token);
        }
        let value = self.next_token_data;
        if self.next_token == TC_INTEGER {
            self.add_new_str(&format!("{value}"));
        } else {
            self.add_new('\'');
            self.add_new((value as u8) as char);
            self.add_new('\'');
        }
        match kind {
            IntegerKind::Expression => self.add_old_value(value),
            IntegerKind::Static => self.add_old_str(&format!("{value}")),
            IntegerKind::Const => (),
        }
        self.read_token();
        value
    }

    fn next_identifier(&mut self) -> String {
        if self.next_token != TC_IDENTIFIER {
            panic!("Expected identifier, got {}", self.next_token);
        }
        let name = self.next_token_data;
        let length = self.next_token_length;
        let result =
            String::from(from_utf8(&self.source[name as usize..(name + length) as usize]).unwrap());
        result
    }

    fn parse_const(&mut self) {
        self.add_old('\n');
        self.add_new_str("const");
        self.parse_token(TC_CONST);
        let identifier = self.next_identifier();
        self.add_new_str(&identifier);
        self.read_token();
        let value = self.parse_integer(IntegerKind::Const);
        self.symbols.insert(
            identifier,
            Symbol {
                kind: SymbolKind::Constant,
                value,
            },
        );
    }

    fn parse_static(&mut self) {
        self.add_both_str("static");
        self.parse_token(TC_STATIC);
        let identifier = self.next_identifier();
        self.add_both_str(&identifier);
        self.read_token();
        self.symbols.insert(
            identifier,
            Symbol {
                kind: SymbolKind::Static,
                value: 0,
            },
        );
        while self.next_token == TC_INTEGER || self.next_token == TC_QUOTED_CHAR {
            self.add_diff();
            self.parse_integer(IntegerKind::Static);
        }
    }

    fn parse_argument(&mut self) {
        let identifier = self.next_identifier();
        if let Some(symbol) = self.symbols.get(&identifier) {
            if symbol.kind == SymbolKind::Variable {
                self.add_old_str(&format!("{}", symbol.value));
            } else {
                self.add_old_str(&identifier);
            }
        } else {
            self.add_old_str(&identifier);
        }
        self.add_new_str(&identifier);
        self.read_token();
    }

    fn parse_label(&mut self) {
        self.add_old(':');
        self.add_new(':');
        self.parse_token(b':');
        let identifier = self.next_identifier();
        self.add_both_str(&identifier);
        self.read_token();
    }

    fn parse_fn_arguments(&mut self) {
        self.add_new('(');
        self.parse_token(b'(');
        while self.next_token != b')' {
            self.parse_expr();
            if self.next_token != b')' {
                self.add_new(',');
                self.parse_token(b',');
            }
        }
        self.add_new(')');
        self.read_token();
    }

    fn parse_primitive_expr(&mut self) {
        if self.next_token == TC_INTEGER || self.next_token == TC_QUOTED_CHAR {
            self.parse_integer(IntegerKind::Expression);
        } else if self.next_token == TC_IDENTIFIER {
            let identifier = self.next_identifier();
            self.add_new_str(&identifier);
            self.read_token();
            if self.next_token == b'(' {
                self.parse_fn_arguments();
                self.add_old_str("call ");
                self.add_old_str(&identifier);
            } else {
                let symbol = self.symbols.get(&identifier).unwrap();
                if symbol.kind == SymbolKind::Variable {
                    self.add_old_str(&format!("get {}", symbol.value));
                } else if symbol.kind == SymbolKind::Static {
                    self.add_old_str(&format!("cst {identifier}"));
                } else {
                    self.add_old_value(symbol.value);
                }
            }
        } else {
            self.add_new('(');
            self.parse_token(b'(');
            self.parse_expr();
            self.add_new(')');
            self.parse_token(b')');
        }
    }

    fn parse_pointer_expr(&mut self) {
        if self.next_token == TC_MUL {
            self.add_new('*');
            self.read_token();
            self.parse_pointer_expr();
            self.add_old_str("load");
        } else if self.next_token == TC_BIT_AND {
            self.add_new('&');
            self.add_old_str("ptr");
            self.read_token();
            let variable = self.next_identifier();
            self.add_new_str(&variable);
            let symbol = self.symbols.get(&variable).unwrap();
            self.add_old_str(&format!("{}", symbol.value));
            self.read_token();
        } else {
            self.parse_primitive_expr();
        }
    }

    fn parse_mult_expr(&mut self) {
        self.parse_pointer_expr();
        let mut next_token = self.next_token;
        while next_token == TC_MUL || next_token == TC_DIV {
            self.add_new(if next_token == TC_MUL { '*' } else { '/' });
            self.read_token();
            self.parse_pointer_expr();
            self.add_old_str(if next_token == TC_MUL { "mul" } else { "div" });
            next_token = self.next_token;
        }
    }

    fn parse_add_expr(&mut self) {
        self.parse_mult_expr();
        let mut next_token = self.next_token;
        while next_token == TC_ADD || next_token == TC_SUB {
            self.add_new(if next_token == TC_ADD { '+' } else { '-' });
            self.read_token();
            self.parse_mult_expr();
            self.add_old_str(if next_token == TC_ADD { "add" } else { "sub" });
            next_token = self.next_token;
        }
    }

    fn parse_bit_and_expr(&mut self) {
        self.parse_add_expr();
        while self.next_token == TC_BIT_AND {
            self.add_new('&');
            self.read_token();
            self.parse_add_expr();
            self.add_old_str("and");
        }
    }

    fn parse_expr(&mut self) {
        self.parse_bit_and_expr();
        while self.next_token == TC_BIT_OR {
            self.add_new('|');
            self.read_token();
            self.parse_bit_and_expr();
            self.add_old_str("or");
        }
    }

    fn parse_instruction(&mut self) {
        let opcode = self.next_token - 128;
        let range =
            self.next_token_data as usize..(self.next_token_data + self.next_token_length) as usize;
        let keyword = String::from(from_utf8(&self.source[range.start..range.end]).unwrap());
        self.add_diff();
        self.add_both_str(&keyword);
        self.add_diff();
        self.read_token();
        if ARG_SIZES[opcode as usize] > 0 {
            self.parse_argument();
        }
    }

    fn parse_let_stmt(&mut self, next_variable: u32) -> u32 {
        self.flush_old_spaces();
        self.add_new_str("let");
        self.parse_token(TC_LET);
        let variable = self.next_identifier();
        self.add_new_str(&variable);
        self.read_token();
        self.flush_new_spaces();
        self.add_diff();
        self.parse_expr();
        self.symbols.insert(
            variable,
            Symbol {
                kind: SymbolKind::Variable,
                value: next_variable,
            },
        );
        self.add_new(';');
        self.parse_token(b';');
        next_variable + 1
    }

    fn parse_statement(&mut self, next_variable: u32) -> u32 {
        let mut result = next_variable;
        if self.next_token == b':' {
            self.parse_label();
        } else if self.next_token == TC_LET {
            result = self.parse_let_stmt(result);
        } else {
            if self.next_token != b';' && self.next_token < 128 {
                self.add_diff();
                self.parse_expr();
                while self.next_token == b',' {
                    self.add_new(',');
                    self.parse_token(b',');
                    self.add_diff();
                    self.parse_expr();
                }
            }
            if self.next_token != b';' {
                self.parse_instruction();
            }
            self.add_diff();
            self.add_new(';');
            self.parse_token(b';');
        }
        result
    }

    fn parse_fn_name(&mut self) {
        let name = self.next_identifier();
        self.add_both_str(&name);
        self.read_token();
    }

    fn parse_fn_parameters(&mut self) -> u32 {
        let mut count = 0;
        self.add_new('(');
        self.parse_token(b'(');
        if self.next_token == TC_IDENTIFIER {
            let variable = self.next_identifier();
            self.add_new_str(&variable);
            self.read_token();
            self.symbols.insert(
                variable,
                Symbol {
                    kind: SymbolKind::Variable,
                    value: count,
                },
            );
            count += 1;
            while self.next_token == b',' {
                self.add_new(',');
                self.read_token();
                let variable = self.next_identifier();
                self.add_new_str(&variable);
                self.read_token();
                self.symbols.insert(
                    variable,
                    Symbol {
                        kind: SymbolKind::Variable,
                        value: count,
                    },
                );
                count += 1;
            }
        }
        self.add_new(')');
        self.parse_token(b')');
        count
    }

    fn parse_fn_body(&mut self, arity: u32) {
        if self.next_token == b';' {
            self.add_diff();
            self.add_both(';');
            self.read_token();
            return;
        }
        self.add_old_str(&format!("{arity}"));
        self.add_new('{');
        self.parse_token(b'{');
        self.add_diff();
        let mut next_variable = arity + 4;
        while self.next_token != b'}' {
            next_variable = self.parse_statement(next_variable);
            self.add_diff();
        }
        self.add_diff();
        self.add_old('\n');
        self.add_new('}');
        self.parse_token(b'}');
    }

    fn parse_fn(&mut self) {
        self.add_both_str("fn");
        self.parse_token(TC_FN);
        self.parse_fn_name();
        self.add_diff();
        let old_symbols = self.symbols.clone();
        let arity = self.parse_fn_parameters();
        self.parse_fn_body(arity);
        self.symbols = old_symbols;
    }

    fn parse_program(&mut self) {
        self.get_next_char();
        self.read_token();
        loop {
            self.add_diff();
            if self.next_token == TC_FN {
                self.parse_fn();
            } else if self.next_token == TC_STATIC {
                self.parse_static();
            } else if self.next_token == TC_CONST {
                self.parse_const();
            } else if self.next_token == 0 {
                break;
            } else {
                panic!("Unexpected token {}", self.next_token);
            }
        }
    }

    fn write_latex(&mut self, filename: &str) -> std::io::Result<()> {
        self.flush_old_spaces();
        self.flush_new_spaces();
        let mut output = File::create(filename)?;
        writeln!(&mut output, "\\begin{{Code}}")?;
        let mut previous_line_empty = true;
        for line in self.diff_producer.get_diff().lines() {
            if line.trim().is_empty() {
                if !previous_line_empty {
                    output.write_all(line.as_bytes())?;
                    writeln!(&mut output)?;
                }
                previous_line_empty = true;
            } else {
                output.write_all(line.as_bytes())?;
                writeln!(&mut output)?;
                previous_line_empty = false;
            }
        }
        writeln!(&mut output, "\\end{{Code}}")?;
        Ok(())
    }

    pub fn write_toy1(&mut self, filename: &str) -> std::io::Result<()> {
        self.close()?;
        write_toy_file(self.diff_producer.get_old(), filename, false, false)
    }

    pub fn write_toy2(&mut self, filename: &str) -> std::io::Result<()> {
        self.close()?;
        write_toy_file(self.diff_producer.get_new(), filename, false, false)
    }

    pub fn check_changes(&self, filename: &str) -> std::io::Result<()> {
        self.diff_producer.check_changes(filename)
    }

    fn close(&mut self) -> std::io::Result<()> {
        if self.closed {
            return Ok(());
        }
        self.parse_program();
        self.closed = true;
        Ok(())
    }
}
