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
};

use emulator::PioDevice;

pub fn bin4<T: Into<u32>>(x: T) -> String {
    format!(r"\bina{{{:04b}}}", x.into())
}

pub fn dec<T: Into<u32>>(x: T) -> String {
    format!(r"{}", x.into())
}

pub fn hex<T: Into<u32>>(x: T) -> String {
    format!(r"\hexa{{{:X}}}", x.into())
}

pub fn hex_opt<T: Into<u32>>(x: T) -> String {
    let x32: u32 = x.into();
    if x32 < 10 {
        dec(x32)
    } else {
        hex(x32)
    }
}

pub fn hex_word<T: Into<u32>>(x: T) -> String {
    format!("{:08X}", x.into())
}

pub fn hex_word_low<T: Into<u32>>(x: T) -> String {
    format!("{:08x}", x.into())
}

pub fn hex_dec<T: Into<u32>>(x: T) -> String {
    let x32: u32 = x.into();
    format!(r"\hexa{{{:X}}} = {}", x32, x32)
}

pub fn dec_hex<T: Into<u32>>(x: T) -> String {
    let x32: u32 = x.into();
    format!(r"{} = \hexa{{{:X}}}", x32, x32)
}

pub fn bin_dec<T: Into<u32>>(x: T) -> String {
    let x32: u32 = x.into();
    format!(r"\bina{{{:b}}} = {}", x32, x32)
}

pub fn bin_hex16<T: Into<u32>>(x: T) -> String {
    let x32: u32 = x.into();
    format!(r"\bina{{{:016b}}} = \hexa{{{:04X}}}", x32, x32)
}

pub fn define(command: &str, value: &str) -> String {
    format!(r"\newcommand{{\{}}}{{{}}}", command, value)
}

pub fn bytes_to_words(bytes: &[u8]) -> Vec<u32> {
    let mut result = Vec::<u32>::with_capacity((bytes.len() + 3) / 4);
    let mut word = 0;
    let mut shift = 0;
    for byte in bytes {
        word |= (*byte as u32) << shift;
        shift += 8;
        if shift == 32 {
            result.push(word);
            word = 0;
            shift = 0;
        }
    }
    if shift != 0 {
        result.push(word);
    }
    result
}

pub fn boot_assistant_commands(words: &[u32], base: u32) -> Vec<String> {
    let mut result = Vec::<String>::new();
    let mut address = base;
    for word in words {
        result.push(format!("W{:X},{:08X}#", address, word));
        address += 4;
    }
    result
}

pub fn flash_helper_commands(text: &str, base: u32) -> Vec<String> {
    let mut words = Vec::<u32>::new();
    words.push(text.len() as u32);
    words.extend(&bytes_to_words(text.as_bytes()));
    let mut result = boot_assistant_commands(&words, base);
    result.push(String::from("flash#"));
    result.push(String::from("reset#"));
    result
}

pub fn write_lines(directory: &str, name: &str, lines: &Vec<String>) -> std::io::Result<()> {
    create_dir_all(directory)?;
    let mut output = File::create(format!("{directory}/{name}"))?;
    for line in lines {
        writeln!(&mut output, "{line}")?;
    }
    Ok(())
}

const BEGIN_CODE: &str = "\\vspace{-4pt}\n\\begin{Code}\n";
const END_CODE: &str = "\n\\end{Code}\n\\vspace{-4pt}\n";

pub fn code(source_code: &str) -> String {
    code_raw(
        &source_code
            .replace('\t', "  ")
            .replace('{', "\\{")
            .replace('}', "\\}")
            .replace("\\n", "\\textbackslash{}"),
    )
}

pub fn code_raw(source_code: &str) -> String {
    let mut result = String::new();
    result.push_str(BEGIN_CODE);
    result.push_str(source_code);
    result.push_str(END_CODE);
    result
}

pub fn code_changes(source_code: &str, original: &str, changes: &[usize]) -> String {
    let mut result = String::new();
    result.push_str(BEGIN_CODE);
    let mut unchanged = String::new();
    let mut next_change = 0;
    for (i, line) in source_code.lines().enumerate() {
        if next_change < changes.len() && i == changes[next_change] {
            assert!(original.contains(&unchanged));
            unchanged.clear();
            result.push_str("\\ToyChange{}");
            next_change += 1;
        } else {
            unchanged.push_str(line);
            unchanged.push('\n');
        }
        result.push_str(&line.replace('\t', "  "));
        result.push('\n');
    }
    result.pop();
    result.push_str(END_CODE);
    result
}

pub fn host_log(log: &str) -> String {
    format!(
        r"\begin{{Shaded}}
\begin{{Highlighting}}
{log}
\end{{Highlighting}}
\end{{Shaded}}"
    )
}

pub fn host_log_multicols(log: &str, num_cols: usize) -> String {
    let lines: Vec<&str> = log.lines().collect();
    let lines_per_column = (lines.len() + num_cols - 1) / num_cols;
    let mut result = String::from(r"\vspace{-\baselineskip}");
    result.push_str(&format!("\\begin{{multicols}}{{{}}}\n", num_cols));
    let mut line = 0;
    for _ in 0..num_cols {
        result.push_str("\\begin{Shaded}\n");
        result.push_str("\\begin{Highlighting}\n");
        for _ in 0..lines_per_column {
            if line < lines.len() {
                result.push_str(lines[line]);
                result.push('\n');
                line += 1;
            } else {
                break;
            }
        }
        result.push_str("\\end{Highlighting}\n");
        result.push_str("\\end{Shaded}\n");
    }
    result.push_str(r"\end{multicols}");
    result
}

#[derive(Default)]
pub struct BlinkLedCounter {
    pub blink_count: u32,
}

impl PioDevice for BlinkLedCounter {
    fn pio_state_changed(&mut self, pins: &[u32; 4]) {
        if (pins[1] & (1 << 27)) != 0 {
            self.blink_count += 1;
        }
    }
}

pub fn next_page_address(address: u32) -> u32 {
    (address + 255) & 0xFFFFFF00
}

pub fn page_number(address: u32) -> u32 {
    if address >= 0x80000 {
        if address < 0xC0000 {
            return (address - 0x80000) / 256;
        } else if address < 0x100000 {
            return (address - 0xC0000) / 256;
        }
    }
    panic!("Unsupported page address {address}");
}

pub fn med_page_row(row: &str) -> String {
    format!("{{\\tt {} \\color{{gray}}{{{}}}}}", &row[..36], &row[36..])
}
