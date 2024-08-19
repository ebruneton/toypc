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
    fs::{self, File},
    io::Write,
};

use super::util::{write_toy, write_toy_file, DiffProducer, MAX_LINE_LENGTH};

#[derive(Default)]
pub struct Transpiler5 {
    original: String,
    placeholders: HashMap<String, String>,
    diff_producer: DiffProducer,
    diff: String,
    closed: bool,
}

impl Transpiler5 {
    pub fn new(filename: &str) -> Self {
        Self {
            original: fs::read_to_string(filename).unwrap(),
            placeholders: HashMap::default(),
            diff_producer: DiffProducer::default(),
            diff: String::new(),
            closed: false,
        }
    }

    pub fn new_str(original: &str) -> Self {
        Self {
            original: String::from(original),
            placeholders: HashMap::default(),
            diff_producer: DiffProducer::default(),
            diff: String::new(),
            closed: false,
        }
    }

    pub fn add_placeholder(&mut self, name: &str, value: &str) {
        self.placeholders
            .insert(String::from(name), String::from(value));
    }

    pub fn add(&mut self, line: &str) {
        if self.placeholders.is_empty() {
            self.add_internal(line);
        } else {
            let mut expanded_line = String::from(line);
            for (name, value) in &self.placeholders {
                expanded_line = expanded_line.replace(name, value);
            }
            self.add_internal(&expanded_line);
        }
    }

    fn add_internal(&mut self, line: &str) {
        if self.original.is_empty() {
            self.add_line_internal(&format!("@{line}"));
        } else {
            self.add_line_internal(line);
        }
    }

    fn add_line_internal(&mut self, line: &str) {
        assert!(!self.closed);
        let mut ellipsis = false;
        if let Some(suffix) = line.strip_prefix('+') {
            self.diff_producer.add_diff();
            self.diff_producer.add_new_str(&Self::fix_spaces(suffix));
            self.diff_producer.add_new('\n');
            self.diff_producer.add_diff();
        } else if let Some(suffix) = line.strip_prefix('-') {
            self.diff_producer.add_diff();
            self.diff_producer.add_old_str(&Self::fix_spaces(suffix));
            self.diff_producer.add_old('\n');
            self.diff_producer.add_diff();
        } else if let Some(suffix) = line.strip_prefix('~') {
            let mut i = 0;
            for part in suffix.split('/') {
                match i {
                    0 => {
                        self.diff_producer.add_old_str(&Self::fix_spaces(part));
                        self.diff_producer.add_new_str(&Self::fix_spaces(part));
                    }
                    1 => {
                        self.diff_producer.add_diff();
                        self.diff_producer.add_old_str(&Self::fix_spaces(part));
                    }
                    _ => {
                        self.diff_producer.add_new_str(&Self::fix_spaces(part));
                        self.diff_producer.add_diff();
                    }
                }
                i = (i + 1) % 3;
            }
            self.diff_producer.add_old('\n');
            self.diff_producer.add_new('\n');
        } else if let Some(suffix) = line.strip_prefix("...") {
            Self::add_both(&mut self.diff_producer, suffix);
            ellipsis = true;
        } else {
            Self::add_both(&mut self.diff_producer, line);
        }
        if ellipsis {
            if !self.diff.ends_with("\\ToyComment{...}\n") {
                self.diff.push_str("\\ToyComment{...}\n");
            }
            self.diff_producer.get_diff();
        } else {
            self.diff.push_str(&self.diff_producer.get_diff());
        }
    }

    fn add_both(diff_producer: &mut DiffProducer, line: &str) {
        let fixed_line = &Self::fix_spaces(line);
        diff_producer.add_old_str(fixed_line);
        diff_producer.add_old('\n');
        diff_producer.add_new_str(fixed_line);
        diff_producer.add_new('\n');
    }

    fn fix_spaces(s: &str) -> String {
        s.replace("  ", "\t")
    }

    pub fn add_unchanged(&mut self, start_line: &str, end_line: &str) {
        assert!(!self.closed);
        let mut found = false;
        let mut write = false;
        for line in self.original.lines() {
            if !found && line.starts_with(start_line) {
                found = true;
                write = true;
            }
            if write && line.starts_with(end_line) {
                write = false;
            }
            if write {
                Self::add_both(&mut self.diff_producer, line);
            }
        }
        if !found {
            panic!("Start line {start_line} not found!");
        }
        self.diff_producer.get_diff();
    }

    pub fn close_unchecked(&mut self) {
        self.closed = true;
    }

    pub fn get_raw(&mut self) -> String {
        self.close();
        String::from(self.diff_producer.get_new())
    }

    pub fn get_toy5(&mut self) -> String {
        self.close();
        let mut buffer = Vec::new();
        write_toy(self.diff_producer.get_new(), &mut buffer, false, true).unwrap();
        String::from_utf8(buffer).unwrap()
    }

    pub fn write(&mut self, filename: &str) {
        self.write_internal(filename).unwrap();
    }

    fn write_internal(&mut self, filename: &str) -> std::io::Result<()> {
        let mut output = File::create(filename)?;
        writeln!(&mut output, "\\begin{{Code}}")?;
        output.write_all(self.diff.as_bytes())?;
        self.diff.clear();
        writeln!(&mut output, "\\end{{Code}}")
    }

    pub fn write_toy4(&mut self, filename: &str) -> std::io::Result<()> {
        self.close();
        write_toy_file(self.diff_producer.get_old(), filename, false, true)
    }

    pub fn write_toy5(&mut self, filename: &str) -> std::io::Result<()> {
        self.close();
        write_toy_file(self.diff_producer.get_new(), filename, false, true)
    }

    fn close(&mut self) {
        if !self.closed {
            if !self.original.is_empty() {
                self.diff_producer.check_changes_str(&self.original);
            }
            self.closed = true;
        }
    }

    pub fn split_changes(source_code: &str) -> String {
        let mut result = String::new();
        let mut previous_line = Option::None;
        for line in source_code.lines() {
            if line.contains('~') {
                if let Some(line) = previous_line {
                    result.push_str(line);
                    result.push('\n');
                }
                result.push_str("\\ToyChange");
                result.push_str(line);
                result.push_str("\n...\n");
            }
            previous_line = Option::Some(line);
        }
        result = DiffProducer::wrap_lines(
            &result
                .replace('{', "\\{")
                .replace('}', "\\}")
                .replace("\\ToyChange", "\\ToyChange{}"),
            MAX_LINE_LENGTH,
        );
        while result.ends_with('\n') {
            result.pop();
        }
        result
    }
}
