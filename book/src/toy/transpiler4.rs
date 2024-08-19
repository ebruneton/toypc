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
    1, 32, 10, 1, 1, 1, 1, 11, 39, 40, 41, 6, 4, 44, 12, 46, 7, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 58,
    59, 13, 14, 15, 1, 1, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
    3, 3, 91, 1, 93, 1, 3, 1, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
    3, 3, 3, 123, 16, 125, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
];

static TC_OPERATORS: [u8; 28] = [
    1, 1, 16, 1, 8, 18, 1, 1, 5, 1, 1, 20, 12, 10, 15, 1, 61, 13, 13, 1, 14, 11, 17, 11, 9, 19, 1,
    1,
];

const TC_INTEGER: u8 = 2;
const TC_QUOTED_CHAR: u8 = 30;
const TC_IDENTIFIER: u8 = 3;
const TC_ADD: u8 = 4;
const TC_SUB: u8 = 5;
const TC_MUL: u8 = 6;
const TC_DIV: u8 = 7;
const TC_BIT_AND: u8 = 8;
const TC_BIT_OR: u8 = 9;
const TC_SHIFT_LEFT: u8 = 10;
const TC_SHIFT_RIGHT: u8 = 11;
const TC_LT: u8 = 12;
const TC_EQ: u8 = 13;
const TC_GT: u8 = 14;
const TC_LE: u8 = 15;
const TC_NE: u8 = 16;
const TC_GE: u8 = 17;
const TC_AND: u8 = 18;
const TC_OR: u8 = 19;
const TC_ARROW: u8 = 20;
const TC_AS: u8 = 128;
const TC_BREAK: u8 = 129;
const TC_CONST: u8 = 130;
const TC_ELSE: u8 = 131;
const TC_FN: u8 = 132;
const TC_IF: u8 = 133;
const TC_LET: u8 = 134;
const TC_LOOP: u8 = 135;
const TC_NULL: u8 = 136;
const TC_RETURN: u8 = 137;
const TC_SIZEOF: u8 = 138;
const TC_STATIC: u8 = 139;
const TC_STRUCT: u8 = 140;
const TC_U32: u8 = 141;
const TC_WHILE: u8 = 142;

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

#[derive(Clone)]
struct BaseType {
    struct_name: String,
}

impl BaseType {
    fn new(struct_name: &str) -> Self {
        Self {
            struct_name: String::from(struct_name),
        }
    }

    fn int() -> Self {
        Self {
            struct_name: String::new(),
        }
    }
}

#[derive(Clone)]
struct Struct {
    prefix: String,
    sizeof_name: String,
    fields: HashMap<String, BaseType>,
}

pub struct Transpiler4 {
    keywords: HashMap<&'static str, u8>,
    source: Vec<u8>,
    src: usize,
    next_char: u8,
    next_char_type: u8,
    next_token: u8,
    next_token_data: u32,
    next_token_length: u32,
    symbols: HashMap<String, BaseType>,
    structs: HashMap<String, Struct>,
    void_fn: bool,
    old_spaces: String,
    new_spaces: String,
    old: Vec<String>,
    new: Vec<String>,
    parts: Vec<String>,
    diff_producer: DiffProducer,
    closed: bool,
}

static DIFF: &str = "<diff>";

impl Transpiler4 {
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
            structs: HashMap::new(),
            void_fn: false,
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
        keywords.insert("as", TC_AS);
        keywords.insert("break", TC_BREAK);
        keywords.insert("const", TC_CONST);
        keywords.insert("else", TC_ELSE);
        keywords.insert("fn", TC_FN);
        keywords.insert("if", TC_IF);
        keywords.insert("let", TC_LET);
        keywords.insert("loop", TC_LOOP);
        keywords.insert("null", TC_NULL);
        keywords.insert("return", TC_RETURN);
        keywords.insert("sizeof", TC_SIZEOF);
        keywords.insert("static", TC_STATIC);
        keywords.insert("struct", TC_STRUCT);
        keywords.insert("u32", TC_U32);
        keywords.insert("while", TC_WHILE);
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
        let mut index = 4 * (first_char_type - 10);
        if second_char_type == first_char_type {
            self.read_char();
            index += 1;
        } else if self.next_char == b'=' {
            self.read_char();
            index += 2;
        } else if self.next_char == b'>' {
            self.read_char();
            index += 3;
        }
        TC_OPERATORS[index as usize]
    }

    fn read_comment(&mut self, mut src: usize, spaces: &mut String) -> bool {
        if self.source[src + 1] != b'*' {
            return false;
        }
        self.old_spaces.push_str(&spaces.replace("  ", "\t"));
        self.new_spaces.push_str(&spaces.replace("  ", "\t"));
        spaces.clear();
        self.add_diff();
        let mut comment = String::new();
        comment.push(self.source[src] as char);
        comment.push(self.source[src + 1] as char);
        while self.source[src + 2] != b'*' || self.source[src + 3] != b'/' {
            comment.push(self.source[src + 2] as char);
            src += 1;
        }
        comment.push(self.source[src + 2] as char);
        comment.push(self.source[src + 3] as char);
        self.add_new_str(&comment);
        self.add_diff();
        self.src = src + 3;
        true
    }

    fn read_token(&mut self) {
        let mut char_type = self.next_char_type;
        let mut spaces = String::new();
        while char_type == b' '
            || char_type == TC_DIV
            || self.next_char == b'@'
            || self.next_char == b'#'
        {
            if self.next_char == b'#' {
                self.read_char();
                self.read_integer();
                char_type = self.next_char_type;
                self.add_diff();
                self.add_both_str(&format!("#{}", self.parts[self.next_token_data as usize]));
                self.add_diff();
            } else {
                if char_type == TC_DIV {
                    if !self.read_comment(self.src, &mut spaces) {
                        break;
                    }
                } else {
                    spaces.push(self.next_char as char);
                }
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

    fn next_identifier(&mut self) -> String {
        if self.next_token != TC_IDENTIFIER {
            panic!("Expected identifier, got {}", self.next_token);
        }
        let name = self.next_token_data;
        let length = self.next_token_length;
        String::from(from_utf8(&self.source[name as usize..(name + length) as usize]).unwrap())
    }

    fn parse_type(&mut self) -> BaseType {
        while self.next_token == TC_BIT_AND || self.next_token == TC_AND {
            self.add_new('&');
            if self.next_token == TC_AND {
                self.add_new('&');
            }
            self.read_token();
        }
        if self.next_token == TC_U32 {
            self.add_new_str("u32");
            self.read_token();
            return BaseType::int();
        }
        let name = self.next_identifier();
        self.add_new_str(&name);
        self.read_token();
        BaseType::new(&name)
    }

    fn parse_const(&mut self) {
        self.add_both_str("const");
        self.parse_token(TC_CONST);
        let identifier = self.next_identifier();
        self.add_both_str(&identifier);
        self.read_token();
        self.add_diff();
        self.add_new(':');
        self.parse_token(b':');
        let base_type = self.parse_type();
        self.add_diff();
        self.add_both('=');
        self.parse_token(b'=');
        self.parse_integer();
        self.symbols.insert(identifier, base_type);
        self.add_both(';');
        self.parse_token(b';');
    }

    fn parse_static(&mut self) {
        self.add_both_str("static");
        self.parse_token(TC_STATIC);
        let identifier = self.next_identifier();
        self.add_both_str(&identifier);
        self.read_token();
        self.add_both('=');
        self.parse_token(b'=');
        self.add_both('[');
        self.parse_token(b'[');
        self.symbols.insert(identifier, BaseType::int());
        self.parse_integer();
        while self.next_token == b',' {
            self.add_both(',');
            self.read_token();
            self.parse_integer();
        }
        self.add_both(']');
        self.parse_token(b']');
        self.add_both(';');
        self.parse_token(b';');
    }

    fn parse_struct(&mut self) {
        self.add_new_str("struct");
        self.add_diff();
        self.parse_token(TC_STRUCT);
        let name = self.next_identifier();
        self.add_new_str(&name);
        self.read_token();
        self.parse_token(b',');
        let prefix = self.next_identifier();
        self.read_token();
        let mut sizeof_name = String::new();
        if self.next_token == b',' {
            self.read_token();
            sizeof_name = self.next_identifier();
            self.read_token();
        }
        self.add_new('{');
        self.parse_token(b'{');
        let mut fields = HashMap::new();
        let mut n = 0;
        while self.next_token != b'}' {
            if n > 0 {
                self.add_new(',');
                self.parse_token(b',');
            }
            let field = self.next_identifier();
            self.old_spaces.clear();
            self.add_old('\n');
            self.add_old_str("const ");
            self.add_old_str(&prefix);
            self.add_diff();
            self.add_both_str(&field);
            self.add_diff();
            self.add_old_str(&format!(" = {};", n * 4));
            self.add_diff();
            self.read_token();
            self.add_new(':');
            self.parse_token(b':');
            self.old_spaces.clear();
            fields.insert(field, self.parse_type());
            self.add_diff();
            n += 1;
        }
        self.structs.insert(
            name,
            Struct {
                prefix,
                sizeof_name: sizeof_name.clone(),
                fields,
            },
        );
        self.add_new('}');
        self.add_diff();
        if !sizeof_name.is_empty() {
            self.add_old_str(&format!("const sizeof_{} = {};\n", sizeof_name, n * 4));
        }
        self.read_token();
        self.add_diff();
    }

    fn parse_fn_arguments(&mut self, function: &str) -> BaseType {
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
        self.symbols.get(function).unwrap().clone()
    }

    fn parse_sizeof_expr(&mut self) -> BaseType {
        self.add_new_str("sizeof");
        self.parse_token(TC_SIZEOF);
        self.add_new('(');
        self.parse_token(b'(');
        let base_type = &self.next_identifier();
        let sizeof_name;
        let sizeof;
        {
            let struct_type = self.structs.get(base_type).unwrap();
            sizeof_name = struct_type.sizeof_name.clone();
            sizeof = struct_type.fields.len() * 4;
        }
        if sizeof_name.is_empty() {
            self.add_old_str(&format!("{sizeof}"));
        } else {
            self.add_old_str("sizeof_");
            self.add_old_str(&sizeof_name);
        }
        self.add_new_str(base_type);
        self.read_token();
        self.add_new(')');
        self.parse_token(b')');
        BaseType::int()
    }

    fn parse_primitive_expr(&mut self) -> BaseType {
        if self.next_token == TC_INTEGER || self.next_token == TC_QUOTED_CHAR {
            self.parse_integer();
            BaseType::int()
        } else if self.next_token == TC_IDENTIFIER {
            let identifier = self.next_identifier();
            self.add_both_str(&identifier);
            self.read_token();
            if self.next_token == b'(' {
                self.parse_fn_arguments(&identifier)
            } else {
                self.symbols.get(&identifier).unwrap().clone()
            }
        } else if self.next_token == TC_SIZEOF {
            self.parse_sizeof_expr()
        } else if self.next_token == TC_NULL {
            self.add_diff();
            self.add_old('0');
            self.add_new_str("null");
            self.add_diff();
            self.read_token();
            BaseType::int()
        } else {
            self.add_both('(');
            self.parse_token(b'(');
            let base_type = self.parse_expr();
            self.add_both(')');
            self.parse_token(b')');
            base_type
        }
    }

    fn parse_path_expr(&mut self, mut address_of: bool) -> BaseType {
        self.add_diff();
        let old_index = self.old.len();
        let old_spaces = self.old_spaces.clone();
        let mut base_type = self.parse_primitive_expr();
        while self.next_token == b'.' {
            if address_of {
                if self.old[old_index - 2] != "&" {
                    panic!("Internal error");
                }
                self.old.remove(old_index - 2);
            } else {
                while self.old[old_index].trim().is_empty() {
                    self.old.remove(old_index);
                }
                self.old.insert(old_index, String::from("("));
                self.old.insert(old_index, String::from("*"));
                self.old.insert(old_index, old_spaces.clone());
            }
            self.add_diff();
            self.add_old('+');
            self.add_new('.');
            self.add_diff();
            self.read_token();
            let field = self.next_identifier();
            let prefix;
            {
                let struct_type = self.structs.get(&base_type.struct_name).unwrap();
                base_type = struct_type.fields.get(&field).unwrap().clone();
                prefix = struct_type.prefix.clone();
            }
            self.add_old_str(&prefix);
            self.add_diff();
            self.add_both_str(&field);
            if !address_of {
                self.old_spaces.clear();
                self.add_diff();
                self.add_old(')');
            }
            self.read_token();
            address_of = false;
        }
        if self.next_token != TC_AS {
            self.add_diff();
        }
        base_type
    }

    fn parse_pointer_expr(&mut self) -> BaseType {
        if self.next_token == TC_MUL {
            self.add_both('*');
            self.read_token();
            self.parse_pointer_expr()
        } else if self.next_token == TC_BIT_AND {
            self.add_both('&');
            self.read_token();
            self.parse_path_expr(true)
        } else {
            self.parse_path_expr(false)
        }
    }

    fn parse_cast_expr(&mut self) -> BaseType {
        let base_type = self.parse_pointer_expr();
        if self.next_token != TC_AS {
            return base_type;
        }
        self.add_new_str("as");
        self.read_token();
        self.old_spaces.clear();
        let cast_type = self.parse_type();
        self.add_diff();
        cast_type
    }

    fn parse_mult_expr(&mut self) -> BaseType {
        let base_type = self.parse_cast_expr();
        let mut next_token = self.next_token;
        while next_token == TC_MUL || next_token == TC_DIV {
            self.add_both(if next_token == TC_MUL { '*' } else { '/' });
            self.read_token();
            self.parse_cast_expr();
            next_token = self.next_token;
        }
        base_type
    }

    fn parse_add_expr(&mut self) -> BaseType {
        let base_type = self.parse_mult_expr();
        let mut next_token = self.next_token;
        while next_token == TC_ADD || next_token == TC_SUB {
            self.add_both(if next_token == TC_ADD { '+' } else { '-' });
            self.read_token();
            self.parse_mult_expr();
            next_token = self.next_token;
        }
        base_type
    }

    fn parse_shift_expr(&mut self) -> BaseType {
        let base_type = self.parse_add_expr();
        let mut next_token = self.next_token;
        while next_token == TC_SHIFT_LEFT || next_token == TC_SHIFT_RIGHT {
            self.add_both_str(if next_token == TC_SHIFT_LEFT {
                "<<"
            } else {
                ">>"
            });
            self.read_token();
            self.parse_add_expr();
            next_token = self.next_token;
        }
        base_type
    }

    fn parse_bit_and_expr(&mut self) -> BaseType {
        let base_type = self.parse_shift_expr();
        while self.next_token == TC_BIT_AND {
            self.add_both('&');
            self.read_token();
            self.parse_shift_expr();
        }
        base_type
    }

    fn parse_expr(&mut self) -> BaseType {
        let base_type = self.parse_bit_and_expr();
        while self.next_token == TC_BIT_OR {
            self.add_both('|');
            self.read_token();
            self.parse_bit_and_expr();
        }
        base_type
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

    fn parse_comparison_expr(&mut self) {
        self.parse_expr();
        let token = self.next_token;
        self.add_both_str(Self::relational_operator(token));
        self.read_token();
        self.parse_expr();
    }

    fn parse_and_expr(&mut self) {
        self.parse_comparison_expr();
        while self.next_token == TC_AND {
            self.add_both_str("&&");
            self.read_token();
            self.parse_comparison_expr();
        }
    }

    fn parse_boolean_expr(&mut self) {
        self.parse_and_expr();
        while self.next_token == TC_OR {
            self.add_both_str("||");
            self.read_token();
            self.parse_and_expr();
        }
    }

    fn parse_block_stmt(&mut self, has_break: &mut bool) -> Reachability {
        let mut state = Reachability::EndReachable;
        self.add_both('{');
        self.parse_token(b'{');
        while self.next_token != b'}' {
            if state == Reachability::EndUnreachable {
                panic!("Unreachable code!");
            }
            state = self.parse_stmt(has_break);
        }
        self.add_both('}');
        self.read_token();
        state
    }

    fn parse_assignment(&mut self) -> Reachability {
        self.add_both('=');
        self.parse_token(b'=');
        self.parse_expr();
        Reachability::EndReachable
    }

    fn parse_expr_or_assign_stmt(&mut self) -> Reachability {
        self.parse_expr();
        if self.next_token == b'=' {
            self.parse_assignment();
        }
        self.add_both(';');
        self.parse_token(b';');
        Reachability::EndReachable
    }

    fn parse_return_stmt(&mut self) -> Reachability {
        self.add_both_str("return");
        self.parse_token(TC_RETURN);
        if self.next_token != b';' {
            self.parse_expr();
        }
        self.add_both(';');
        self.parse_token(b';');
        Reachability::EndUnreachable
    }

    fn parse_break_stmt(&mut self, has_break: &mut bool) -> Reachability {
        self.add_both_str("break");
        self.read_token();
        self.add_both(';');
        self.parse_token(b';');
        *has_break = true;
        Reachability::EndUnreachable
    }

    fn parse_while_or_loop_stmt(&mut self, expr: bool, _has_break: &mut bool) -> Reachability {
        self.add_both_str(if expr { "while" } else { "loop" });
        self.read_token();
        if expr {
            self.parse_boolean_expr();
        }
        let mut loop_has_break = false;
        self.parse_block_stmt(&mut loop_has_break);
        if !expr && !loop_has_break {
            return Reachability::EndUnreachable;
        }
        Reachability::EndReachable
    }

    fn parse_if_stmt(&mut self, has_break: &mut bool) -> Reachability {
        self.add_both_str("if");
        self.parse_token(TC_IF);
        self.parse_boolean_expr();
        let mut state = self.parse_block_stmt(has_break);
        if self.next_token == TC_ELSE {
            self.add_both_str("else");
            self.read_token();
            if self.next_token == b'{' {
                state = state.or(&self.parse_block_stmt(has_break));
            } else if self.next_token == TC_IF {
                state = state.or(&self.parse_if_stmt(has_break));
            }
        } else {
            state = Reachability::EndReachable;
        }
        state
    }

    fn parse_stmt(&mut self, has_break: &mut bool) -> Reachability {
        if self.next_token == TC_IF {
            self.parse_if_stmt(has_break)
        } else if self.next_token == TC_WHILE {
            self.parse_while_or_loop_stmt(true, has_break)
        } else if self.next_token == TC_LOOP {
            self.parse_while_or_loop_stmt(false, has_break)
        } else if self.next_token == TC_BREAK {
            self.parse_break_stmt(has_break)
        } else if self.next_token == TC_RETURN {
            self.parse_return_stmt()
        } else {
            self.parse_expr_or_assign_stmt()
        }
    }

    fn parse_let_stmt(&mut self) {
        self.add_both_str("let");
        self.parse_token(TC_LET);
        let variable = self.next_identifier();
        self.add_both_str(&variable);
        self.read_token();
        let separator = self.next_token;
        let mut base_type = BaseType::int();
        if separator == b':' {
            self.add_diff();
            self.add_new(':');
            self.read_token();
            base_type = self.parse_type();
            self.add_diff();
        }
        self.add_both('=');
        self.parse_token(b'=');
        let expr_typ = self.parse_expr();
        if separator == b':' {
            self.symbols.insert(variable, base_type);
        } else {
            self.symbols.insert(variable, expr_typ);
        }
        self.add_both(';');
        self.parse_token(b';');
    }

    fn parse_fn_name(&mut self) -> String {
        let name = self.next_identifier();
        self.add_both_str(&name);
        self.read_token();
        name
    }

    fn parse_fn_parameters(&mut self) -> BaseType {
        let mut i = 0;
        self.add_both('(');
        self.parse_token(b'(');
        while self.next_token != b')' {
            if i > 0 {
                self.add_both(',');
                self.parse_token(b',');
            }
            let variable = self.next_identifier();
            self.add_both_str(&variable);
            self.read_token();
            self.add_diff();
            self.add_new(':');
            self.parse_token(b':');
            let base_type = self.parse_type();
            self.old_spaces.clear();
            self.add_diff();
            self.symbols.insert(variable, base_type);
            i += 1;
        }
        self.add_both(')');
        self.add_diff();
        self.parse_token(b')');
        if self.next_token == TC_ARROW {
            self.add_new_str("->");
            self.read_token();
            self.old_spaces.clear();
            let base_type = self.parse_type();
            self.add_diff();
            self.void_fn = false;
            base_type
        } else {
            self.void_fn = true;
            BaseType::int()
        }
    }

    fn parse_fn_body(&mut self) {
        if self.next_token == b';' {
            self.add_both(';');
            self.read_token();
            return;
        }
        self.add_both('{');
        self.parse_token(b'{');
        let mut state = Reachability::EndReachable;
        while self.next_token != b'}' {
            if self.next_token == TC_LET {
                self.parse_let_stmt();
            } else {
                let mut has_break = false;
                state = self.parse_stmt(&mut has_break);
            }
        }
        if self.void_fn && state == Reachability::EndReachable {
            let unchanged = self.old_spaces.ends_with('@');
            self.add_diff();
            if self.is_single_line_fn() {
                self.add_old_str("return; ");
            } else if unchanged {
                self.add_old_str("\treturn;\n@");
            } else {
                self.add_old_str("\treturn;\n");
            }
            self.add_diff();
        }
        self.add_both('}');
        self.parse_token(b'}');
    }

    fn is_single_line_fn(&self) -> bool {
        for old in self.old.iter().rev() {
            if old == "\n" {
                break;
            }
            if old == "fn" {
                return true;
            }
        }
        false
    }

    fn parse_fn(&mut self) {
        self.add_both_str("fn");
        self.read_token();
        let name = self.parse_fn_name();
        let mut old_symbols = self.symbols.clone();
        let base_type = self.parse_fn_parameters();
        old_symbols.insert(name.clone(), base_type.clone());
        self.symbols.insert(name.clone(), base_type);
        self.parse_fn_body();
        self.symbols = old_symbols;
    }

    fn parse_program(&mut self) {
        self.get_next_char();
        self.read_token();
        loop {
            self.add_diff();
            if self.next_token == TC_FN {
                self.parse_fn();
            } else if self.next_token == TC_STRUCT {
                self.parse_struct();
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

    pub fn write_toy3(&mut self, filename: &str) -> std::io::Result<()> {
        self.close()?;
        write_toy_file(self.diff_producer.get_old(), filename, false, false)
    }

    pub fn write_toy4(&mut self, filename: &str) -> std::io::Result<()> {
        self.close()?;
        write_toy_file(self.diff_producer.get_new(), filename, false, false)
    }

    pub fn check_changes(&self, filename: &str) -> std::io::Result<()> {
        self.diff_producer.check_changes(filename)
    }
}
