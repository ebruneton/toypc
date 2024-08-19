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
    fs::{create_dir_all, File},
    io::Write,
    ops::Range,
    path::Path,
};

pub const MAX_LINE_LENGTH: usize = 75;

pub fn check_line_lengths(s: &str) {
    for line in s.lines() {
        if line.len() + line.chars().filter(|c| *c == '\t').count() > 100 {
            panic!("Line length too long '{}'", line);
        }
    }
}

fn write_line(line: &str, output: &mut dyn Write, fix_indentation: bool) -> std::io::Result<()> {
    if fix_indentation {
        let trim = line.trim();
        if !trim.is_empty()
            && !trim.starts_with("const ")
            && !trim.starts_with("static ")
            && !trim.starts_with("fn ")
            && !trim.starts_with(':')
            && !trim.starts_with('}')
        {
            output.write_all(&[b'\t'])?;
        }
        output.write_all(trim.as_bytes())?;
    } else {
        output.write_all(line.trim_end().as_bytes())?;
    }
    writeln!(output)
}

pub fn write_toy(
    source: &str,
    output: &mut dyn Write,
    fix_indentation: bool,
    preserve_empty_lines_in_functions: bool,
) -> std::io::Result<()> {
    check_line_lengths(source);
    let mut previous_line_empty = true;
    let mut previous_line_starts_with_const = false;
    let mut previous_line_starts_with_static = false;
    let mut previous_line_starts_with_fn = false;
    let mut previous_line_ends_block = false;
    let mut in_function = false;
    for line in source.lines() {
        if line.trim().is_empty() {
            if previous_line_starts_with_const || previous_line_ends_block || in_function {
                write_line(line, output, fix_indentation)?;
                previous_line_empty = true;
            }
            previous_line_starts_with_const = false;
            previous_line_starts_with_static = false;
            previous_line_starts_with_fn = false;
            previous_line_ends_block = false;
        } else {
            if line.starts_with("struct ") && !previous_line_empty {
                writeln!(output)?;
            }
            if line.starts_with("const ")
                && !(previous_line_empty || previous_line_starts_with_const)
            {
                writeln!(output)?;
            }
            if line.starts_with("static ")
                && !(previous_line_empty
                    || (line.ends_with("];") && previous_line_starts_with_static))
            {
                writeln!(output)?;
            }
            if line.starts_with("fn ")
                && !(previous_line_empty
                    || ((line.ends_with('}') || line.ends_with(']'))
                        && previous_line_starts_with_fn))
            {
                writeln!(output)?;
            }
            write_line(line, output, fix_indentation)?;
            previous_line_empty = false;
            previous_line_starts_with_const = line.starts_with("const ");
            previous_line_starts_with_static = line.starts_with("static ");
            previous_line_starts_with_fn = line.starts_with("fn ");
            previous_line_ends_block = line.contains(']')
                || (line.starts_with("fn ") && line.trim_end().ends_with('}'))
                || line.starts_with('}');
            if !in_function && preserve_empty_lines_in_functions {
                in_function = line.starts_with("fn ") && line.trim_end().ends_with('{');
            } else if line.starts_with('}') {
                in_function = false;
            }
        }
    }
    Ok(())
}

pub fn write_toy_file(
    source: &str,
    filename: &str,
    fix_indentation: bool,
    preserve_empty_lines_in_functions: bool,
) -> std::io::Result<()> {
    check_line_lengths(source);
    create_dir_all(Path::new(filename).parent().unwrap())?;
    let mut output = File::create(filename)?;
    write_toy(
        source,
        &mut output,
        fix_indentation,
        preserve_empty_lines_in_functions,
    )
}

#[derive(Default)]
pub struct DiffProducer {
    old: String,
    old_changes: String,
    new: String,
    diff: String,
    diff_old: String,
    diff_new: String,
}

impl DiffProducer {
    pub fn add_old(&mut self, c: char) {
        if c != '@' {
            self.old.push(c);
        }
        self.old_changes.push(c);
        if c == '\t' {
            self.diff_old.push_str("  ");
        } else if c == '\\' {
            self.diff_old.push_str("\\textbackslash");
        } else if c == '{' {
            self.diff_old.push_str("\\{");
        } else if c == '}' {
            self.diff_old.push_str("\\}");
        } else {
            self.diff_old.push(c);
        }
    }

    pub fn add_old_str(&mut self, s: &str) {
        self.old.push_str(&s.replace('@', ""));
        self.old_changes.push_str(s);
        if s.starts_with(':') {
            self.old_changes.push(' ');
        }
        self.diff_old.push_str(
            &s.replace('\t', "  ")
                .replace('\\', "\\textbackslash")
                .replace('{', "\\{")
                .replace('}', "\\}"),
        );
    }

    pub fn add_new(&mut self, c: char) {
        assert!(c != '@');
        self.new.push(c);
        if c == '{' {
            self.diff_new.push_str("\\{");
        } else if c == '\\' {
            self.diff_new.push_str("\\textbackslash");
        } else if c == '}' {
            self.diff_new.push_str("\\}");
        } else {
            self.diff_new.push(c);
        }
    }

    pub fn add_new_str(&mut self, s: &str) {
        self.new.push_str(&s.replace('@', ""));
        self.diff_new.push_str(
            &s.replace('\t', "  ")
                .replace('\\', "\\textbackslash")
                .replace('{', "\\{")
                .replace('}', "\\}"),
        );
    }

    fn split_line(s: &str) -> (&str, &str) {
        if s.ends_with('\n') {
            (&s[0..s.len() - 1], &s[s.len() - 1..s.len()])
        } else {
            (&s[0..], &s[0..0])
        }
    }

    pub fn add_diff(&mut self) {
        let old_lines: Vec<&str> = self.diff_old.split_inclusive('\n').collect();
        let new_lines: Vec<&str> = self.diff_new.split_inclusive('\n').collect();
        if new_lines.len() == old_lines.len() {
            for i in 0..new_lines.len() {
                let old_line = Self::split_line(old_lines[i]);
                let new_line = Self::split_line(new_lines[i]);
                if new_line.0 == old_line.0 {
                    self.diff.push_str(new_lines[i]);
                } else if new_line.0.is_empty() {
                    self.diff.push_str(&format!(
                        "{}{}",
                        Self::diff_commands("Delete", old_line.0),
                        old_line.1
                    ));
                } else if old_line.0.is_empty() {
                    self.diff.push_str(&format!(
                        "{}{}",
                        Self::diff_commands("Insert", new_line.0),
                        new_line.1
                    ));
                } else {
                    Self::add_line_diff(old_line.0, new_line.0, new_line.1, &mut self.diff);
                }
            }
        } else {
            for old_line in old_lines {
                let line = Self::split_line(old_line);
                self.diff.push_str(&Self::diff_commands("Delete", line.0));
                self.diff.push_str(line.1);
            }
            for new_line in new_lines {
                let line = Self::split_line(new_line);
                self.diff.push_str(&Self::diff_commands("Insert", line.0));
                self.diff.push_str(line.1);
            }
        }
        self.diff_old.clear();
        self.diff_new.clear();
    }

    fn add_line_diff(old: &str, new: &str, end: &str, diff: &mut String) {
        let old_parts: Vec<&str> = Self::separate(old);
        let new_parts: Vec<&str> = Self::separate(new);
        let mut shared_start = 0;
        while shared_start < old_parts.len()
            && shared_start < new_parts.len()
            && old_parts[shared_start] == new_parts[shared_start]
        {
            shared_start += 1;
        }
        for part in &old_parts[0..shared_start] {
            diff.push_str(part);
        }
        let mut shared_end = 0;
        while shared_start + shared_end < old_parts.len()
            && shared_start + shared_end < new_parts.len()
            && old_parts[old_parts.len() - 1 - shared_end]
                == new_parts[new_parts.len() - 1 - shared_end]
        {
            shared_end += 1;
        }
        Self::add_parts_diff(
            &old_parts[shared_start..old_parts.len() - shared_end],
            &new_parts[shared_start..new_parts.len() - shared_end],
            diff,
        );
        for i in (0..shared_end).rev() {
            diff.push_str(old_parts[old_parts.len() - 1 - i]);
        }
        diff.push_str(end);
    }

    fn separate(s: &str) -> Vec<&str> {
        let mut result = Vec::new();
        let mut last_i = 0;
        let mut last_is_whitespace_or_punctuation = false;
        for (i, c) in s.chars().enumerate() {
            if c.is_whitespace() || c.is_ascii_punctuation() {
                if i > last_i {
                    result.push(&s[last_i..i]);
                }
                last_i = i;
                last_is_whitespace_or_punctuation = true;
            } else {
                if last_is_whitespace_or_punctuation {
                    if i > last_i {
                        result.push(&s[last_i..i]);
                    }
                    last_i = i;
                }
                last_is_whitespace_or_punctuation = false;
            }
        }
        if last_i < s.len() {
            result.push(&s[last_i..]);
        }
        result
    }

    fn add_parts_diff(old: &[&str], new: &[&str], diff: &mut String) {
        let mut shared_middle = 0;
        for i in 0..std::cmp::min(old.len(), new.len()) {
            if old[old.len() - i..] == new[0..i] {
                shared_middle = i;
            }
        }
        let mut delete = String::new();
        for part in &old[0..old.len() - shared_middle] {
            delete.push_str(part);
        }
        let mut middle = String::new();
        for part in &old[old.len() - shared_middle..] {
            middle.push_str(part);
        }
        let mut insert = String::new();
        for part in &new[shared_middle..] {
            insert.push_str(part);
        }
        if middle.is_empty() && (insert.trim().is_empty() || insert.trim() == "@") {
            diff.push_str(&Self::diff_commands("Insert", &insert));
            diff.push_str(&Self::diff_commands("Delete", &delete));
        } else {
            diff.push_str(&Self::diff_commands("Delete", &delete));
            diff.push_str(&middle);
            diff.push_str(&Self::diff_commands("Insert", &insert));
        }
    }

    fn diff_commands(command: &str, argument: &str) -> String {
        let mut result = String::new();
        if !argument.is_empty() {
            result.push_str(&format!("\\Toy{}{{{}}}", command, argument));
        }
        result
    }

    pub fn get_old(&self) -> &str {
        &self.old
    }

    pub fn get_new(&self) -> &str {
        &self.new
    }

    pub fn get_diff(&mut self) -> String {
        self.add_diff();
        let mut changes = String::new();
        for line in self.diff.lines() {
            if let Some(suffix) = line.strip_prefix('@') {
                changes.push_str(suffix);
            } else if line.contains('@') {
                changes.push_str(&line.replace('@', ""));
            } else {
                if !line.is_empty() {
                    changes.push_str("\\ToyChange{}");
                }
                changes.push_str(line);
            }
            changes.push('\n');
        }
        let diff = Self::wrap_lines(&changes, MAX_LINE_LENGTH);
        self.diff.clear();
        diff
    }

    pub fn check_changes(&self, filename: &str) -> std::io::Result<()> {
        self.check_changes_str(std::fs::read_to_string(filename)?.as_str());
        Ok(())
    }

    pub fn check_changes_str(&self, original: &str) {
        let reference = Self::normalize(original);
        let mut unchanged = String::new();
        for line in self.old_changes.lines() {
            if let Some(suffix) = line.strip_prefix('@') {
                unchanged.push_str(suffix);
                unchanged.push('\n');
            } else {
                Self::check_unchanged(&unchanged, &reference);
                unchanged.clear();
            }
        }
        Self::check_unchanged(&unchanged, &reference);
    }

    fn check_unchanged(s: &str, reference: &str) {
        if !reference.contains(&Self::normalize(s)) {
            panic!("Code incorrectly marked as unchanged: '{s}'");
        }
    }

    fn normalize(s: &str) -> String {
        s.split(|c: char| c.is_ascii_whitespace())
            .filter(|s| !s.is_empty())
            .fold(String::new(), |mut f, s| {
                f.push_str(s);
                f.push(' ');
                f
            })
    }

    pub fn wrap_lines(s: &str, max_line_length: usize) -> String {
        let mut result = String::new();
        for line in s.lines() {
            result.push_str(&Self::wrap_line(line, max_line_length));
            result.push('\n');
        }
        result
    }

    pub fn wrap_line(s: &str, max_line_length: usize) -> String {
        #[derive(Clone)]
        struct State {
            length: usize, // length to index (excluded)
            index: usize,  // where cut is possible (not possible inside command names)
            command: Range<usize>,
        }
        let mut previous = State {
            length: 0,
            index: 0,
            command: 0..0,
        };
        let mut current = previous.clone();
        let mut wrap = current.clone();
        let mut in_argument = false;
        let bytes = s.as_bytes();
        for (i, &byte) in bytes.iter().enumerate() {
            if current.command.is_empty() {
                previous = current.clone();
                if byte == b'\\' {
                    current.command = current.index..current.index + 1;
                } else {
                    current.length += 1;
                    current.index = i + 1;
                }
            } else if in_argument {
                if byte == b'}' && bytes[i - 1] != b'\\' {
                    current.index = i + 1;
                    current.command = 0..0;
                    in_argument = false;
                } else {
                    previous = current.clone();
                    if byte != b'\\' {
                        current.index = i + 1;
                        current.length += 1;
                    }
                }
            } else if (byte == b'{' || byte == b'}') && bytes[i - 1] == b'\\' {
                previous = current.clone();
                current.length += 1;
                current.index = i + 1;
                current.command = 0..0;
            } else if byte == b'{' {
                current.command.end = i + 1;
                current.index = i + 1;
                if &s[current.command.start..current.command.end] == "\\ToyLet{" {
                    current.length += 3; // accounts for the \leftarrow added by this command.
                }
                in_argument = true;
            }
            if (current.length >= max_line_length
                || current.length + 1 >= max_line_length
                    && !current.command.is_empty()
                    && !in_argument)
                && current.index < bytes.len()
                && wrap.length == 0
            {
                wrap = previous.clone();
            }
        }
        if wrap.length > 0 {
            let padding = max_line_length - 1 - (current.length - wrap.length);
            Self::wrap_line_at(s, wrap.index, &wrap.command, padding)
        } else {
            String::from(s)
        }
    }

    fn wrap_line_at(s: &str, index: usize, command: &Range<usize>, padding: usize) -> String {
        let mut result = String::new();
        result.push_str(&s[0..index]);
        if !command.is_empty() {
            result.push('}');
        }
        result.push_str("\\ToyWrap{}");
        result.push('\n');
        if result.starts_with("\\ToyChange{}") {
            result.push_str("\\ToyChange{}");
        }
        for _i in 0..padding {
            result.push(' ');
        }
        result.push_str("\\ToyUnwrap{}");
        result.push_str(&s[command.start..command.end]);
        result.push_str(&s[index..]);
        result
    }
}

#[cfg(test)]
mod tests {
    use super::DiffProducer;

    #[test]
    fn test_wrap_raw_text() {
        let text = "abcdefghijkl";

        assert_eq!(
            DiffProducer::wrap_line(text, 11),
            "abcdefghij\\ToyWrap{}\n        \\ToyUnwrap{}kl"
        );
        assert_eq!(
            DiffProducer::wrap_line(text, 10),
            "abcdefghi\\ToyWrap{}\n      \\ToyUnwrap{}jkl"
        );
        assert_eq!(
            DiffProducer::wrap_line(text, 9),
            "abcdefgh\\ToyWrap{}\n    \\ToyUnwrap{}ijkl"
        );
        assert_eq!(
            DiffProducer::wrap_line(text, 7),
            "abcdef\\ToyWrap{}\n\\ToyUnwrap{}ghijkl"
        );
    }

    #[test]
    fn test_wrap_escaped_opening_brace() {
        let text = "abcdefgh\\{jkl";

        assert_eq!(
            DiffProducer::wrap_line(text, 11),
            "abcdefgh\\{j\\ToyWrap{}\n        \\ToyUnwrap{}kl"
        );
        assert_eq!(
            DiffProducer::wrap_line(text, 10),
            "abcdefgh\\{\\ToyWrap{}\n      \\ToyUnwrap{}jkl"
        );
        assert_eq!(
            DiffProducer::wrap_line(text, 9),
            "abcdefgh\\ToyWrap{}\n    \\ToyUnwrap{}\\{jkl"
        );
        assert_eq!(
            DiffProducer::wrap_line(text, 7),
            "abcdef\\ToyWrap{}\n\\ToyUnwrap{}gh\\{jkl"
        );
    }

    #[test]
    fn test_wrap_escaped_closing_brace() {
        let text = "abcdefgh\\}jkl";

        assert_eq!(
            DiffProducer::wrap_line(text, 11),
            "abcdefgh\\}j\\ToyWrap{}\n        \\ToyUnwrap{}kl"
        );
        assert_eq!(
            DiffProducer::wrap_line(text, 10),
            "abcdefgh\\}\\ToyWrap{}\n      \\ToyUnwrap{}jkl"
        );
        assert_eq!(
            DiffProducer::wrap_line(text, 9),
            "abcdefgh\\ToyWrap{}\n    \\ToyUnwrap{}\\}jkl"
        );
        assert_eq!(
            DiffProducer::wrap_line(text, 7),
            "abcdef\\ToyWrap{}\n\\ToyUnwrap{}gh\\}jkl"
        );
    }

    #[test]
    fn test_wrap_command() {
        let text = "abcdef\\Command{ghi}jkl";

        assert_eq!(
            DiffProducer::wrap_line(text, 11),
            "abcdef\\Command{ghi}j\\ToyWrap{}\n        \\ToyUnwrap{}kl"
        );
        assert_eq!(
            DiffProducer::wrap_line(text, 10),
            "abcdef\\Command{ghi}\\ToyWrap{}\n      \\ToyUnwrap{}jkl"
        );
        assert_eq!(
            DiffProducer::wrap_line(text, 9),
            "abcdef\\Command{gh}\\ToyWrap{}\n    \\ToyUnwrap{}\\Command{i}jkl"
        );
        assert_eq!(
            DiffProducer::wrap_line(text, 7),
            "abcdef\\ToyWrap{}\n\\ToyUnwrap{}\\Command{ghi}jkl"
        );
    }

    #[test]
    fn test_wrap_escaped_brace_in_command() {
        let text = "abcdef\\Command{gh\\{}jkl";

        assert_eq!(
            DiffProducer::wrap_line(text, 11),
            "abcdef\\Command{gh\\{}j\\ToyWrap{}\n        \\ToyUnwrap{}kl"
        );
        assert_eq!(
            DiffProducer::wrap_line(text, 10),
            "abcdef\\Command{gh\\{}\\ToyWrap{}\n      \\ToyUnwrap{}jkl"
        );
        assert_eq!(
            DiffProducer::wrap_line(text, 9),
            "abcdef\\Command{gh}\\ToyWrap{}\n    \\ToyUnwrap{}\\Command{\\{}jkl"
        );
        assert_eq!(
            DiffProducer::wrap_line(text, 7),
            "abcdef\\ToyWrap{}\n\\ToyUnwrap{}\\Command{gh\\{}jkl"
        );
    }
}
