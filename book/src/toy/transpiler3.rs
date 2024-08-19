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
    collections::{HashMap, HashSet},
    fs::File,
    io::Write,
    str::from_utf8,
};

use super::util::{write_toy_file, DiffProducer};

static TC_CHAR_TYPES: [u8; 256] = [
    1, 1, 1, 1, 1, 1, 1, 1, 1, 32, 32, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 32, 10, 1, 1, 1, 1, 11, 39, 40, 41, 6, 4, 44, 5, 1, 7, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 58, 59,
    12, 13, 14, 1, 1, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
    91, 1, 93, 1, 3, 1, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
    3, 123, 15, 125, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1,
];

static TC_OPERATORS: [u8; 18] = [
    1, 1, 16, 8, 18, 1, 12, 10, 15, 61, 13, 13, 14, 11, 17, 9, 19, 1,
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
const TC_LT: u8 = 12;
const TC_EQ: u8 = 13;
const TC_GT: u8 = 14;
const TC_LE: u8 = 15;
const TC_NE: u8 = 16;
const TC_GE: u8 = 17;
const TC_AND: u8 = 18;
const TC_OR: u8 = 19;
const TC_BREAK: u8 = 128;
const TC_CONST: u8 = 129;
const TC_ELSE: u8 = 130;
const TC_FN: u8 = 131;
const TC_IF: u8 = 132;
const TC_LET: u8 = 133;
const TC_LOOP: u8 = 134;
const TC_RETURN: u8 = 135;
const TC_STATIC: u8 = 136;
const TC_WHILE: u8 = 137;
const TC_VOID_FN: u8 = 138;

#[derive(Clone, PartialEq)]
enum Symbol {
    Constant,
    Static,
    Variable,
    Function,
    VoidFunction,
    Label,
}

#[derive(Clone, PartialEq)]
enum Origin {
    Address,
    Variable,
    Other,
    Void,
    Unreachable,
}

struct Label {
    name: String,
    reachable: bool,
}

impl Label {
    fn new(name: String) -> Self {
        Self {
            name,
            reachable: false,
        }
    }
}

#[derive(Clone, PartialEq)]
enum Reachability {
    EndReachable,
    EndUnreachable,
}

impl Reachability {
    fn or(&self, other: &Reachability) -> Reachability {
        if self == &Reachability::EndReachable || other == &Reachability::EndReachable {
            Reachability::EndReachable
        } else {
            Reachability::EndUnreachable
        }
    }
}

pub struct Transpiler3 {
    keywords: HashMap<&'static str, u8>,
    source: Vec<u8>,
    src: usize,
    next_char: u8,
    next_char_type: u8,
    next_token: u8,
    next_token_data: u32,
    next_token_length: u32,
    symbols: HashMap<String, Symbol>,
    next_label: u32,
    old_spaces: String,
    new_spaces: String,
    old: Vec<String>,
    new: Vec<String>,
    parts: Vec<String>,
    diff_producer: DiffProducer,
    closed: bool,
}

static DIFF: &str = "<diff>";

impl Transpiler3 {
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
            next_label: 0,
            old_spaces: String::new(),
            new_spaces: String::new(),
            old: Vec::new(),
            new: Vec::new(),
            parts: Vec::new(),
            diff_producer: DiffProducer::default(),
            closed: false,
        }
    }

    fn fill_keywords() -> HashMap<&'static str, u8> {
        let mut keywords = HashMap::new();
        keywords.insert("break", TC_BREAK);
        keywords.insert("const", TC_CONST);
        keywords.insert("else", TC_ELSE);
        keywords.insert("fn", TC_FN);
        keywords.insert("if", TC_IF);
        keywords.insert("let", TC_LET);
        keywords.insert("loop", TC_LOOP);
        keywords.insert("return", TC_RETURN);
        keywords.insert("static", TC_STATIC);
        keywords.insert("while", TC_WHILE);
        keywords.insert("voidfn", TC_VOID_FN);
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

    fn read_operator(&mut self, first_char_type: u8) -> u8 {
        let second_char_type = self.read_char();
        let mut index = 3 * (first_char_type - 10);
        if second_char_type == first_char_type {
            self.read_char();
            index += 1;
        } else if self.next_char == b'=' {
            self.read_char();
            index += 2;
        }
        TC_OPERATORS[index as usize]
    }

    fn read_token(&mut self) {
        let mut char_type = self.next_char_type;
        let mut spaces = String::new();
        while char_type == b' ' || self.next_char == b'@' || self.next_char == b'#' {
            if self.next_char == b'#' {
                self.read_char();
                self.read_integer();
                char_type = self.next_char_type;
                self.add_diff();
                self.add_old_str(&format!("#{}", self.parts[self.next_token_data as usize]));
                self.add_new_str(&format!("#{}", self.parts[self.next_token_data as usize]));
                self.add_diff();
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
        } else if (10..20).contains(&char_type) {
            token = self.read_operator(char_type);
        } else if char_type != 0 {
            self.read_char();
        }
        self.next_token = token;
    }

    fn flush_old_spaces(&mut self) {
        for c in self.old_spaces.chars() {
            if c != ' ' || self.old.is_empty() || !self.old.last().unwrap().ends_with([' ', '\t']) {
                self.old.push(format!("{c}"));
            }
        }
        self.old_spaces.clear();
    }

    fn add_old(&mut self, c: char) {
        if c == ',' && self.old_spaces.ends_with(' ') {
            self.old_spaces.pop();
        }
        self.flush_old_spaces();
        self.old.push(format!("{c}"));
    }

    fn add_old_str(&mut self, s: &str) {
        self.flush_old_spaces();
        self.old.push(String::from(s));
    }

    fn flush_new_spaces(&mut self) {
        if !self.new_spaces.is_empty() {
            self.new.push(self.new_spaces.clone());
            self.new_spaces.clear();
        }
    }

    fn add_new(&mut self, c: char) {
        self.flush_new_spaces();
        self.new.push(format!("{c}"));
    }

    fn add_new_str(&mut self, s: &str) {
        self.flush_new_spaces();
        self.new.push(String::from(s));
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
        self.flush_old_spaces();
        self.flush_new_spaces();
        self.old.push(String::from(DIFF));
        self.new.push(String::from(DIFF));
    }

    fn parse_token(&mut self, token: u8) {
        if self.next_token != token {
            panic!("Unexpected token {}, expected {token}", self.next_token);
        }
        self.read_token();
    }

    fn parse_integer(&mut self) {
        if self.next_token != TC_INTEGER && self.next_token != TC_QUOTED_CHAR {
            panic!("Expected integer, got {}", self.next_token);
        }
        let value = self.next_token_data;
        if self.next_token == TC_INTEGER {
            self.add_both_str(&format!("{value}"));
        } else {
            self.add_both('\'');
            self.add_both((value as u8) as char);
            self.add_both('\'');
        }
        self.read_token();
    }

    fn parse_identifier(&mut self) -> String {
        if self.next_token != TC_IDENTIFIER {
            panic!("Expected identifier, got {}", self.next_token);
        }
        let name = self.next_token_data;
        let length = self.next_token_length;
        let result =
            String::from(from_utf8(&self.source[name as usize..(name + length) as usize]).unwrap());
        self.add_both_str(&result);
        self.read_token();
        result
    }

    fn parse_const(&mut self) {
        self.add_both_str("const");
        self.parse_token(TC_CONST);
        let identifier = self.parse_identifier();
        self.old_spaces.clear();
        self.new_spaces.clear();
        self.add_diff();
        self.add_new(' ');
        self.add_new('=');
        self.parse_token(b'=');
        self.add_diff();
        self.parse_integer();
        self.symbols.insert(identifier, Symbol::Constant);
        self.add_diff();
        self.add_new(';');
        self.parse_token(b';');
    }

    fn parse_static(&mut self) {
        self.add_both_str("static");
        self.parse_token(TC_STATIC);
        let identifier = self.parse_identifier();
        self.add_diff();
        self.add_new('=');
        self.parse_token(b'=');
        self.old_spaces.clear();
        self.add_new('[');
        self.parse_token(b'[');
        self.add_diff();
        self.symbols.insert(identifier, Symbol::Static);
        self.parse_integer();
        while self.next_token == b',' {
            self.add_diff();
            self.add_old(' ');
            self.add_new(',');
            self.add_diff();
            self.read_token();
            self.parse_integer();
        }
        self.add_diff();
        self.add_new(']');
        self.parse_token(b']');
        self.add_new(';');
        self.parse_token(b';');
    }

    fn parse_fn_arguments(&mut self, function: &str) -> Origin {
        self.add_both('(');
        self.parse_token(b'(');
        while self.next_token != b')' {
            self.parse_expr();
            if self.next_token != b')' {
                self.add_both(',');
                self.parse_token(b',');
            }
        }
        self.add_both(')');
        self.read_token();
        if function == "panic" {
            Origin::Unreachable
        } else if self.symbols.get(function).unwrap() == &Symbol::VoidFunction {
            Origin::Void
        } else {
            Origin::Other
        }
    }

    fn parse_primitive_expr(&mut self) -> Origin {
        if self.next_token == TC_INTEGER || self.next_token == TC_QUOTED_CHAR {
            self.parse_integer();
            Origin::Other
        } else if self.next_token == TC_IDENTIFIER {
            let identifier = self.parse_identifier();
            if self.next_token == b'(' {
                self.parse_fn_arguments(&identifier)
            } else if self.symbols.get(&identifier).unwrap() == &Symbol::Variable {
                Origin::Variable
            } else {
                Origin::Other
            }
        } else {
            self.add_both('(');
            self.parse_token(b'(');
            let origin = self.parse_expr();
            self.add_both(')');
            self.parse_token(b')');
            origin
        }
    }

    fn parse_pointer_expr(&mut self) -> Origin {
        if self.next_token == TC_MUL {
            self.add_both('*');
            self.add_diff();
            self.read_token();
            self.parse_pointer_expr();
            Origin::Address
        } else if self.next_token == TC_BIT_AND {
            self.add_both('&');
            self.read_token();
            self.parse_identifier();
            Origin::Other
        } else {
            self.parse_primitive_expr()
        }
    }

    fn parse_mult_expr(&mut self) -> Origin {
        let mut origin = self.parse_pointer_expr();
        let mut next_token = self.next_token;
        while next_token == TC_MUL || next_token == TC_DIV {
            self.add_both(if next_token == TC_MUL { '*' } else { '/' });
            self.read_token();
            self.parse_pointer_expr();
            origin = Origin::Other;
            next_token = self.next_token;
        }
        origin
    }

    fn parse_add_expr(&mut self) -> Origin {
        let mut origin = self.parse_mult_expr();
        let mut next_token = self.next_token;
        while next_token == TC_ADD || next_token == TC_SUB {
            self.add_both(if next_token == TC_ADD { '+' } else { '-' });
            self.read_token();
            self.parse_mult_expr();
            origin = Origin::Other;
            next_token = self.next_token;
        }
        origin
    }

    fn parse_bit_and_expr(&mut self) -> Origin {
        let mut origin = self.parse_add_expr();
        while self.next_token == TC_BIT_AND {
            self.add_both('&');
            self.read_token();
            self.parse_add_expr();
            origin = Origin::Other;
        }
        origin
    }

    fn parse_expr(&mut self) -> Origin {
        let mut origin = self.parse_bit_and_expr();
        while self.next_token == TC_BIT_OR {
            self.add_both('|');
            self.read_token();
            self.parse_bit_and_expr();
            origin = Origin::Other
        }
        origin
    }

    fn relational_opcode(token: u8) -> &'static str {
        match token {
            TC_LT => "iflt",
            TC_EQ => "ifeq",
            TC_GT => "ifgt",
            TC_LE => "ifle",
            TC_NE => "ifne",
            TC_GE => "ifge",
            _ => panic!("Unexpected token {token}"),
        }
    }

    fn relational_operator(token: u8) -> &'static str {
        match token {
            TC_LT => "<",
            TC_EQ => "==",
            TC_GT => ">",
            TC_LE => "<=",
            TC_NE => "!=",
            TC_GE => ">=",
            _ => panic!("Unexpected token {token}"),
        }
    }

    fn new_label(&mut self) -> Label {
        let result = Label::new(format!("_l{}", self.next_label));
        self.next_label += 1;
        result
    }

    fn add_label(&mut self, label: &Label) {
        if self.old_spaces.ends_with([' ', '\t']) {
            self.old_spaces.pop();
        }
        self.add_diff();
        self.add_old_str(&format!(":{}", label.name));
        self.add_diff();
    }

    fn add_token_with_label(&mut self, token: u8, label: &mut Label) {
        self.add_diff();
        self.add_old_str(Self::relational_opcode(token));
        self.add_old(' ');
        self.add_old_str(&label.name);
        self.add_old(';');
        self.add_diff();
        label.reachable = true;
    }

    fn insert_goto(&mut self, index: usize, label: &mut Label) {
        self.old.insert(index, String::from(";"));
        self.old.insert(index, label.name.clone());
        self.old.insert(index, String::from(" "));
        self.old.insert(index, String::from("goto"));
        label.reachable = true;
    }

    fn parse_comparison_expr(&mut self) -> u8 {
        self.parse_expr();
        let token = self.next_token;
        self.old_spaces.clear();
        self.new_spaces.clear();
        self.add_diff();
        self.add_old(',');
        self.add_new(' ');
        self.add_new_str(Self::relational_operator(token));
        self.add_diff();
        self.read_token();
        self.parse_expr();
        token
    }

    fn parse_and_expr(&mut self, else_label: &mut Label) -> u8 {
        let mut token = self.parse_comparison_expr();
        while self.next_token == TC_AND {
            self.add_token_with_label(TC_LT + TC_GE - token, else_label);
            self.add_new_str("&&");
            self.add_diff();
            self.read_token();
            token = self.parse_comparison_expr();
        }
        token
    }

    fn parse_boolean_expr(&mut self, then_label: &mut Label) -> Label {
        let mut else_label = self.new_label();
        let mut token = self.parse_and_expr(&mut else_label);
        while self.next_token == TC_OR {
            self.add_token_with_label(token, then_label);
            self.add_label(&else_label);
            self.add_new_str("||");
            self.add_diff();
            self.read_token();
            else_label = self.new_label();
            token = self.parse_and_expr(&mut else_label);
        }
        self.add_token_with_label(TC_LT + TC_GE - token, &mut else_label);
        else_label
    }

    fn parse_label(&mut self) {
        self.add_diff();
        self.parse_token(b':');
        let name = self.next_token_data;
        let length = self.next_token_length;
        let label =
            String::from(from_utf8(&self.source[name as usize..(name + length) as usize]).unwrap());
        if self.symbols.contains_key(&label) {
            panic!("Label '{label}' already defined as another symbol");
        }
        self.symbols.insert(label.clone(), Symbol::Label);
        self.add_label(&Label::new(label));
        self.read_token();
        self.parse_token(b':');
    }

    fn parse_block_stmt(&mut self, break_label: &mut Label, end_index: &mut usize) -> Reachability {
        let mut state = Reachability::EndReachable;
        self.add_new('{');
        self.add_diff();
        self.parse_token(b'{');
        while self.next_token != b'}' {
            if state == Reachability::EndUnreachable {
                panic!("Unreachable code!");
            }
            if self.next_token == b':' {
                self.parse_label();
            }
            state = self.parse_stmt(break_label);
        }
        self.add_diff();
        *end_index = self.old.len();
        self.add_new('}');
        self.read_token();
        self.add_diff();
        state
    }

    fn parse_assignment(&mut self, origin: &Origin, old_lhs: usize) -> Reachability {
        let variable = self.old.remove(old_lhs);
        self.old_spaces.clear();
        self.new_spaces.clear();
        self.add_diff();
        if *origin == Origin::Address {
            if variable != "*" {
                panic!("Internal error");
            }
            self.add_old(',');
        }
        self.add_new(' ');
        self.add_new('=');
        self.parse_token(b'=');
        if *origin != Origin::Address {
            self.old_spaces.clear();
        }
        self.add_diff();
        self.parse_expr();
        self.add_diff();
        if *origin == Origin::Address {
            self.add_old_str(" store");
        } else {
            self.add_old_str(" set ");
            self.add_old_str(&variable);
        }
        Reachability::EndReachable
    }

    fn parse_expr_or_assign_stmt(&mut self) -> Reachability {
        self.flush_old_spaces();
        let old_size = self.old.len();
        let origin = self.parse_expr();
        if self.next_token == b'=' {
            self.parse_assignment(&origin, old_size);
        } else if origin != Origin::Void && origin != Origin::Unreachable {
            self.add_diff();
            self.add_old_str(" pop");
        }
        self.add_diff();
        self.add_both(';');
        self.parse_token(b';');
        if origin == Origin::Unreachable {
            Reachability::EndUnreachable
        } else {
            Reachability::EndReachable
        }
    }

    fn parse_return_stmt(&mut self) -> Reachability {
        self.add_new_str("return");
        self.parse_token(TC_RETURN);
        if self.next_token != b';' {
            self.old_spaces.clear();
            self.add_diff();
            self.parse_expr();
            self.add_diff();
            self.add_old(' ');
            self.add_old_str("retv");
        } else {
            self.add_old_str("ret");
        }
        self.add_diff();
        self.add_both(';');
        self.parse_token(b';');
        Reachability::EndUnreachable
    }

    fn parse_break_stmt(&mut self, label: &mut Label) -> Reachability {
        self.add_new_str("break");
        self.read_token();
        self.flush_old_spaces();
        self.insert_goto(self.old.len(), label);
        self.add_new(';');
        self.add_diff();
        self.parse_token(b';');
        Reachability::EndUnreachable
    }

    fn parse_while_or_loop_stmt(&mut self, expr: bool) -> Reachability {
        let mut loop_label = self.new_label();
        let mut start_label = self.new_label();
        let mut end_label = self.new_label();
        let mut end_loop_index = 0;
        self.add_label(&loop_label);
        self.add_new_str(if expr { "while" } else { "loop" });
        self.read_token();
        self.old_spaces.clear();
        self.add_diff();
        if expr {
            end_label = self.parse_boolean_expr(&mut start_label);
        }
        self.add_label(&start_label);
        let state = self.parse_block_stmt(&mut end_label, &mut end_loop_index);
        if state == Reachability::EndReachable {
            self.insert_goto(end_loop_index, &mut loop_label);
        }
        self.add_label(&end_label);
        if !expr && !end_label.reachable {
            return Reachability::EndUnreachable;
        }
        Reachability::EndReachable
    }

    fn parse_if_stmt(&mut self, break_label: &mut Label) -> Reachability {
        self.add_new_str("if");
        self.parse_token(TC_IF);
        self.old_spaces.clear();
        self.add_diff();
        let mut then_label = self.new_label();
        let else_label = self.parse_boolean_expr(&mut then_label);
        self.add_label(&then_label);
        let mut end_if_index = 0;
        let mut state = self.parse_block_stmt(break_label, &mut end_if_index);
        let mut end_if_label = self.new_label();
        if self.next_token == TC_ELSE {
            self.add_diff();
            if state == Reachability::EndReachable {
                self.insert_goto(end_if_index, &mut end_if_label);
            }
            self.add_label(&else_label);
            self.add_new_str("else");
            self.read_token();
            self.old_spaces.clear();
            self.add_diff();
            if self.next_token == b':' {
                self.parse_label();
            }
            if self.next_token == b'{' {
                state = state.or(&self.parse_block_stmt(break_label, &mut end_if_index));
            } else if self.next_token == TC_IF {
                state = state.or(&self.parse_if_stmt(break_label));
            }
            self.add_label(&end_if_label);
        } else {
            self.add_label(&else_label);
            state = Reachability::EndReachable;
        }
        state
    }

    fn parse_stmt(&mut self, break_label: &mut Label) -> Reachability {
        self.flush_old_spaces();
        self.flush_new_spaces();
        self.add_diff();
        if self.next_token == TC_IF {
            self.parse_if_stmt(break_label)
        } else if self.next_token == TC_WHILE {
            self.parse_while_or_loop_stmt(true)
        } else if self.next_token == TC_LOOP {
            self.parse_while_or_loop_stmt(false)
        } else if self.next_token == TC_BREAK {
            self.parse_break_stmt(break_label)
        } else if self.next_token == TC_RETURN {
            self.parse_return_stmt()
        } else {
            self.parse_expr_or_assign_stmt()
        }
    }

    fn parse_let_stmt(&mut self) {
        self.add_both_str("let");
        self.parse_token(TC_LET);
        let variable = self.parse_identifier();
        self.old_spaces.clear();
        self.new_spaces.clear();
        self.add_diff();
        self.add_new_str(" =");
        self.parse_token(b'=');
        self.add_diff();
        self.parse_expr();
        self.symbols.insert(variable, Symbol::Variable);
        self.add_both(';');
        self.parse_token(b';');
    }

    fn parse_fn_name(&mut self) -> String {
        self.parse_identifier()
    }

    fn parse_fn_parameters(&mut self) {
        self.add_both('(');
        self.parse_token(b'(');
        if self.next_token == TC_IDENTIFIER {
            let variable = self.parse_identifier();
            self.symbols.insert(variable, Symbol::Variable);
            while self.next_token == b',' {
                self.add_both(',');
                self.read_token();
                let variable = self.parse_identifier();
                self.symbols.insert(variable, Symbol::Variable);
            }
        }
        self.add_both(')');
        self.parse_token(b')');
    }

    fn parse_fn_body(&mut self) {
        if self.next_token == b';' {
            self.add_diff();
            self.add_both(';');
            self.read_token();
            return;
        }
        let fn_start = self.old.len();
        self.add_both('{');
        self.parse_token(b'{');
        self.add_diff();
        while self.next_token != b'}' {
            if self.next_token == b':' {
                self.parse_label();
            }
            if self.next_token == TC_LET {
                self.parse_let_stmt();
            } else {
                self.parse_stmt(&mut Label::new(String::new()));
            }
            self.add_diff();
        }
        self.add_diff();
        self.add_both('}');
        self.parse_token(b'}');
        self.normalize_labels(fn_start);
        self.optimize_gotos(fn_start);
        self.delete_unreachable_gotos(fn_start);
    }

    fn normalize_labels(&mut self, fn_start: usize) {
        let mut canonical_names = HashMap::new();
        let mut same_labels = Vec::new();
        let mut i = fn_start;
        while i < self.old.len() {
            if self.old[i].starts_with(':') {
                same_labels.push(&self.old[i]);
            } else if !same_labels.is_empty() && !Self::is_blank(&self.old[i]) {
                let mut canonical_name = same_labels[0];
                for label in &same_labels {
                    if !label.starts_with(":_") {
                        canonical_name = label;
                        break;
                    }
                }
                let canonical_name = String::from(&canonical_name[1..]);
                for label in &same_labels {
                    canonical_names.insert(String::from(&label[1..]), canonical_name.clone());
                }
                same_labels.clear();
            }
            i += 1;
        }
        let mut used_labels = HashSet::new();
        for i in fn_start..self.old.len() {
            if self.old[i].starts_with('_') {
                let canonical_name = canonical_names.get(&self.old[i]).unwrap();
                used_labels.insert(canonical_name.clone());
                self.old[i] = canonical_name.clone();
            }
        }
        i = fn_start;
        while i < self.old.len() {
            if self.old[i].starts_with(":_") {
                if !used_labels.contains(&self.old[i][1..]) {
                    assert!(self.old[i] != DIFF);
                    self.old.remove(i);
                    i -= 1;
                } else {
                    let label = String::from(&self.old[i][1..]);
                    self.old[i] = format!(":{}", canonical_names.get(&label).unwrap());
                }
            }
            i += 1;
        }
    }

    fn optimize_gotos(&mut self, fn_start: usize) {
        let mut final_targets = HashMap::new();
        let mut previous_label: Option<usize> = Option::None;
        let mut i = fn_start;
        while i < self.old.len() {
            if self.old[i] == "goto" && previous_label.is_some() {
                let j = previous_label.unwrap();
                final_targets.insert(
                    String::from(&self.old[j][1..]),
                    String::from(&self.old[i + 2]),
                );
            } else if self.old[i] == "ret" && previous_label.is_some() {
                let j = previous_label.unwrap();
                final_targets.insert(String::from(&self.old[j][1..]), String::from("ret"));
            }
            if self.old[i].starts_with(':') {
                previous_label = Option::Some(i);
            } else if !Self::is_blank(&self.old[i]) {
                previous_label = Option::None;
            }
            i += 1;
        }
        let mut used_labels = HashSet::new();
        for i in fn_start..self.old.len() {
            if i > 2 && self.old[i - 2] == "goto" && self.old[i - 1] == " " {
                let mut label = &self.old[i];
                while let Some(target) = final_targets.get(label) {
                    if target == "ret" && self.old[i - 2] != "goto" {
                        break;
                    }
                    label = target;
                }
                used_labels.insert(label.clone());
                self.old[i] = label.clone();
            }
        }
        i = fn_start;
        while i < self.old.len() {
            if self.old[i].starts_with(":_") && !used_labels.contains(&self.old[i][1..]) {
                assert!(self.old[i] != DIFF);
                self.old.remove(i);
                i -= 1;
            }
            if self.old[i] == "goto" && self.old[i + 2] == "ret" {
                assert!(self.old[i] != DIFF);
                self.old.remove(i);
                assert!(self.old[i] != DIFF);
                self.old.remove(i);
                i -= 2;
            }
            i += 1;
        }
    }

    fn delete_unreachable_gotos(&mut self, fn_start: usize) {
        let mut i = fn_start;
        while i < self.old.len() {
            if self.old[i] != "goto" || !self.is_unreachable(i - 1) {
                i += 1;
                continue;
            }
            assert_eq!(self.old[i + 1], " ");
            assert_eq!(self.old[i + 3], ";");
            self.old.remove(i);
            self.old.remove(i);
            self.old.remove(i);
            self.old.remove(i);
            i -= 4;
        }
    }

    fn is_unreachable(&mut self, mut index: usize) -> bool {
        while index > 0 && (Self::is_blank(&self.old[index]) || self.old[index] == ";") {
            index -= 1;
        }
        if self.old[index] == "ret" {
            return true;
        }
        if index > 3
            && self.old[index - 3] == "panic"
            && self.old[index - 2] == "("
            && self.old[index] == ")"
        {
            return true;
        }
        false
    }

    fn is_blank(s: &str) -> bool {
        s == "@" || s == DIFF || s.trim().is_empty()
    }

    fn parse_fn(&mut self, symbol: Symbol) {
        self.add_both_str("fn");
        self.read_token();
        let name = self.parse_fn_name();
        self.next_label = 1;
        self.symbols.insert(name, symbol);
        self.add_diff();
        let old_symbols = self.symbols.clone();
        self.parse_fn_parameters();
        self.parse_fn_body();
        self.symbols = old_symbols;
    }

    fn parse_program(&mut self) {
        self.get_next_char();
        self.read_token();
        loop {
            self.add_diff();
            if self.next_token == TC_FN {
                self.parse_fn(Symbol::Function);
            } else if self.next_token == TC_VOID_FN {
                self.parse_fn(Symbol::VoidFunction);
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

    fn close(&mut self) -> std::io::Result<()> {
        if self.closed {
            return Ok(());
        }
        self.parse_program();
        self.write_latex()?;
        self.closed = true;
        Ok(())
    }

    fn write_latex(&mut self) -> std::io::Result<()> {
        let old_slices = self.old.split(|x| x == DIFF);
        let mut new_slices = self.new.split(|x| x == DIFF);
        for old_slice in old_slices {
            let new_slice = new_slices.next().unwrap();
            if old_slice.len() == 1 && old_slice.first().unwrap().starts_with('#') {
                let mut output = File::create(&old_slice.first().unwrap()[1..])?;
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
            } else {
                if !(old_slice == [" "] && new_slice == [","]) {
                    for old in old_slice {
                        self.diff_producer.add_old_str(old);
                    }
                }
                for new in new_slice {
                    self.diff_producer.add_new_str(new);
                }
                self.diff_producer.add_diff();
            }
        }
        Ok(())
    }

    pub fn write_toy2(&mut self, filename: &str) -> std::io::Result<()> {
        self.close()?;
        let mut source = String::new();
        for old in &self.old {
            if old != DIFF && !old.starts_with('#') {
                if old.starts_with(':') && !source.ends_with('\n') {
                    source.push('\n');
                }
                source.push_str(&old.replace('@', ""));
                if old.starts_with(':') {
                    source.push('\n');
                }
            }
        }
        write_toy_file(&source, filename, true, false)
    }

    pub fn write_toy3(&mut self, filename: &str) -> std::io::Result<()> {
        self.close()?;
        write_toy_file(self.diff_producer.get_new(), filename, false, false)
    }

    pub fn check_changes(&self, filename: &str) -> std::io::Result<()> {
        self.diff_producer.check_changes(filename)
    }
}
