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

use std::collections::HashMap;

struct Key {
    name: &'static str,
    lower_case: u8,
    upper_case: u8,
    press: &'static str,
    release: &'static str,
}

impl Key {
    const fn new(
        name: &'static str,
        lower_case: u8,
        upper_case: u8,
        press: &'static str,
        release: &'static str,
    ) -> Self {
        Self {
            name,
            lower_case,
            upper_case,
            press,
            release,
        }
    }
}

const KEYS: [Key; 98] = [
    Key::new("F9", 0x88, 0x88, "01", "F0,01"),
    Key::new("F5", 0x84, 0x84, "03", "F0,03"),
    Key::new("F3", 0x82, 0x82, "04", "F0,04"),
    Key::new("F1", 0x80, 0x80, "05", "F0,05"),
    Key::new("F2", 0x81, 0x81, "06", "F0,06"),
    Key::new("F12", 0x8B, 0x8B, "07", "F0,07"),
    Key::new("F10", 0x89, 0x89, "09", "F0,09"),
    Key::new("F8", 0x87, 0x87, "0A", "F0,0A"),
    Key::new("F6", 0x85, 0x85, "0B", "F0,0B"),
    Key::new("F4", 0x83, 0x83, "0C", "F0,0C"),
    Key::new("Tab", 0x09, 0x09, "0D", "F0,0D"),
    Key::new("` / ~", 0x60, 0x7E, "0E", "F0,0E"),
    Key::new("Alt (left)", 0x8D, 0x8D, "11", "F0,11"),
    Key::new("Shift (left)", 0, 0, "12", "F0,12"),
    Key::new("Ctrl (left)", 0x8C, 0x8C, "14", "F0,14"),
    Key::new("q / Q", 0x71, 0x51, "15", "F0,15"),
    Key::new("1 / !", 0x31, 0x21, "16", "F0,16"),
    Key::new("z / Z", 0x7A, 0x5A, "1A", "F0,1A"),
    Key::new("s / S", 0x73, 0x53, "1B", "F0,1B"),
    Key::new("a / A", 0x61, 0x41, "1C", "F0,1C"),
    Key::new("w / W", 0x77, 0x57, "1D", "F0,1D"),
    Key::new("2 / @", 0x32, 0x40, "1E", "F0,1E"),
    Key::new("c / C", 0x63, 0x43, "21", "F0,21"),
    Key::new("x / X", 0x78, 0x58, "22", "F0,22"),
    Key::new("d / D", 0x64, 0x44, "23", "F0,23"),
    Key::new("e / E", 0x65, 0x45, "24", "F0,24"),
    Key::new("4 / $", 0x34, 0x24, "25", "F0,25"),
    Key::new("3 / #", 0x33, 0x23, "26", "F0,26"),
    Key::new("Space", 0x20, 0x20, "29", "F0,29"),
    Key::new("v / V", 0x76, 0x56, "2A", "F0,2A"),
    Key::new("f / F", 0x66, 0x46, "2B", "F0,2B"),
    Key::new("t / T", 0x74, 0x54, "2C", "F0,2C"),
    Key::new("r / R", 0x72, 0x52, "2D", "F0,2D"),
    Key::new("5 / %", 0x35, 0x25, "2E", "F0,2E"),
    Key::new("n / N", 0x6E, 0x4E, "31", "F0,31"),
    Key::new("b / B", 0x62, 0x42, "32", "F0,32"),
    Key::new("h / H", 0x68, 0x48, "33", "F0,33"),
    Key::new("g / G", 0x67, 0x47, "34", "F0,34"),
    Key::new("y / Y", 0x79, 0x59, "35", "F0,35"),
    Key::new("6 / ^", 0x36, 0x5E, "36", "F0,36"),
    Key::new("m / M", 0x6D, 0x4D, "3A", "F0,3A"),
    Key::new("j / J", 0x6A, 0x4A, "3B", "F0,3B"),
    Key::new("u / U", 0x75, 0x55, "3C", "F0,3C"),
    Key::new("7 / &", 0x37, 0x26, "3D", "F0,3D"),
    Key::new("8 / *", 0x38, 0x2A, "3E", "F0,3E"),
    Key::new(", / <", 0x2C, 0x3C, "41", "F0,41"),
    Key::new("k / K", 0x6B, 0x4B, "42", "F0,42"),
    Key::new("i / I", 0x69, 0x49, "43", "F0,43"),
    Key::new("o / O", 0x6F, 0x4F, "44", "F0,44"),
    Key::new("0 / )", 0x30, 0x29, "45", "F0,45"),
    Key::new("9 / (", 0x39, 0x28, "46", "F0,46"),
    Key::new(". / >", 0x2E, 0x3E, "49", "F0,49"),
    Key::new("/ / ?", 0x2F, 0x3F, "4A", "F0,4A"),
    Key::new("l / L", 0x6C, 0x4C, "4B", "F0,4B"),
    Key::new("; / :", 0x3B, 0x3A, "4C", "F0,4C"),
    Key::new("p / P", 0x70, 0x50, "4D", "F0,4D"),
    Key::new("- / _", 0x2D, 0x5F, "4E", "F0,4E"),
    Key::new("' / \"", 0x27, 0x22, "52", "F0,52"),
    Key::new("[ / {", 0x5B, 0x7B, "54", "F0,54"),
    Key::new("= / +", 0x3D, 0x2B, "55", "F0,55"),
    Key::new("CapsLock", 0x8F, 0x8F, "58", "F0,58"),
    Key::new("Shift (right)", 0, 0, "59", "F0,59"),
    Key::new("Enter", 0x0A, 0x0A, "5A", "F0,5A"),
    Key::new("] / }", 0x5D, 0x7D, "5B", "F0,5B"),
    Key::new("\\ / |", 0x5C, 0x7C, "5D", "F0,5D"),
    Key::new("BackSpace", 0x08, 0x08, "66", "F0,66"),
    Key::new("1 (keypad)", 0x31, 0x31, "69", "F0,69"),
    Key::new("4 (keypad)", 0x34, 0x34, "6B", "F0,6B"),
    Key::new("7 (keypad)", 0x37, 0x37, "6C", "F0,6C"),
    Key::new("0 (keypad)", 0x30, 0x30, "70", "F0,70"),
    Key::new(". (keypad)", 0x2E, 0x2E, "71", "F0,71"),
    Key::new("2 (keypad)", 0x32, 0x32, "72", "F0,72"),
    Key::new("5 (keypad)", 0x35, 0x35, "73", "F0,73"),
    Key::new("6 (keypad)", 0x36, 0x36, "74", "F0,74"),
    Key::new("8 (keypad)", 0x38, 0x38, "75", "F0,75"),
    Key::new("Escape", 0x1B, 0x1B, "76", "F0,76"),
    Key::new("NumLock", 0x8E, 0x8E, "77", "F0,77"),
    Key::new("F11", 0x8A, 0x8A, "78", "F0,78"),
    Key::new("+ (keypad)", 0x2B, 0x2B, "79", "F0,79"),
    Key::new("3 (keypad)", 0x33, 0x33, "7A", "F0,7A"),
    Key::new("- (keypad)", 0x2D, 0x2D, "7B", "F0,7B"),
    Key::new("* (keypad)", 0x2A, 0x2A, "7C", "F0,7C"),
    Key::new("9 (keypad)", 0x39, 0x39, "7D", "F0,7D"),
    Key::new("ScrollLock", 0x90, 0x90, "7E", "F0,7E"),
    Key::new("F7", 0x86, 0x86, "83", "F0,83"),
    Key::new(
        "PrintScreen",
        0xFC,
        0xFC,
        "E0,12,E0,7C",
        "E0,F0,7C,E0,F0,12",
    ),
    Key::new("Alt (right)", 0x91, 0x91, "E0,11", "E0,F0,11"),
    Key::new("End", 0xE9, 0xE9, "E0,69", "E0,F0,69"),
    Key::new("ArrowLeft", 0xEB, 0xEB, "E0,6B", "E0,F0,6B"),
    Key::new("Home", 0xEC, 0xEC, "E0,6C", "E0,F0,6C"),
    Key::new("Insert", 0xF0, 0xF0, "E0,70", "E0,F0,70"),
    Key::new("Delete", 0xF1, 0xF1, "E0,71", "E0,F0,71"),
    Key::new("ArrowDown", 0xF2, 0xF2, "E0,72", "E0,F0,72"),
    Key::new("ArrowRight", 0xF4, 0xF4, "E0,74", "E0,F0,74"),
    Key::new("ArrowUp", 0xF5, 0xF5, "E0,75", "E0,F0,75"),
    Key::new("PageDown", 0xFA, 0xFA, "E0,7A", "E0,F0,7A"),
    Key::new("PageUp", 0xFD, 0xFD, "E0,7D", "E0,F0,7D"),
    Key::new("Pause", 0x00, 0x00, "E1,14,77,E1,F0,14,F0,77", ""),
];

pub fn ascii_table() -> String {
    let mut items = Vec::new();
    for i in 0..128 {
        let code = String::from(&format!("{i:02X}"));
        let value = if i <= 32 {
            match i {
                8 => String::from("BackSpace"),
                9 => String::from("Tab"),
                10 => String::from("Enter"),
                27 => String::from("Escape"),
                32 => String::from("Space"),
                _ => continue,
            }
        } else if i == 127 {
            String::from("Delete")
        } else {
            format!("\\char{i}")
        };
        items.push((code, value));
    }
    const COLUMN_COUNT: usize = 4;
    let mut result = String::new();
    let row_count = (items.len() + COLUMN_COUNT - 1) / COLUMN_COUNT;
    let mut row = 0;
    for (index, (code, value)) in items.iter().enumerate() {
        if row == 0 {
            result.push_str("\\begin{tabular}{|l|l|}\n");
            result.push_str("\\hline \\makecell{\\thead{Code}} & \\thead{Char} \\\\ \\hline\n");
        }
        result.push_str(&format!("\\makecell{{{code}}} & {value} \\\\\n"));
        row += 1;
        if row == row_count {
            result.push_str("\\hline\n\\end{tabular}");
            if index < items.len() - 1 {
                result.push_str("\\hspace{2mm}\n");
            }
            row = 0;
        }
    }
    result
}

fn to_latex(key: &str) -> String {
    if key.len() == 5 && key.contains(" / ") {
        String::from(&format!(
            "\\char{} & \\char{}",
            key.chars().next().unwrap() as u8,
            key.chars().nth(4).unwrap() as u8
        ))
    } else {
        String::from(&format!("\\multicolumn{{2}}{{|l|}}{{{}}}", key))
    }
}

pub fn scancode_table() -> String {
    const ROW_COUNTS: [u32; 4] = [24, 24, 24, 24];
    let mut result = String::new();
    let mut row = 0;
    let mut column = 0;
    let mut closed = true;
    for key in KEYS {
        if key.name == "PrintScreen" || key.name == "Pause" {
            continue;
        }
        if row == 0 {
            result.push_str("\\begin{tabular}[t]{|ll|l|l|}\n");
            result.push_str("\\hline \\multicolumn{2}{|l|}{\\thead{Key}} & \\thead{Press} & \\makecell{\\thead{Release}} \\\\ \\hline\n");
            closed = false;
        }
        result.push_str(&format!(
            "{} & {} & \\makecell{{{}}} \\\\\n",
            to_latex(key.name),
            key.press,
            key.release
        ));
        row += 1;
        if row == ROW_COUNTS[column] {
            result.push_str("\\hline\n\\end{tabular}");
            if column % 2 == 0 {
                result.push_str("\\hspace{2mm}\n");
            } else {
                result.push_str("\n\n");
            }
            row = 0;
            column += 1;
            closed = true;
        }
    }
    if !closed {
        result.push_str("\\hline\n\\end{tabular}\\\\\n");
    }
    result.push_str("\\vspace{5mm}");
    result.push_str("\\begin{tabular}[t]{|l|l|l|}\n");
    result.push_str(
        "\\hline \\makecell{\\thead{Key}} & \\thead{Press} & \\thead{Release}  \\\\ \\hline\n",
    );
    for key in KEYS {
        if key.name != "PrintScreen" && key.name != "Pause" {
            continue;
        }
        result.push_str(&format!(
            "{} & {} & \\makecell{{{}}} \\\\\n",
            key.name, key.press, key.release
        ));
    }
    result.push_str("\\hline\n\\end{tabular}");
    result
}

const CODE_TABLE_LEN: usize = 132;

pub fn code_tables() -> Vec<u8> {
    let mut result = vec![0; 2 * CODE_TABLE_LEN];
    for key in KEYS {
        if key.press.starts_with("E0,") || key.press.starts_with("E1,") {
            continue;
        }
        let code = usize::from_str_radix(key.press, 16).unwrap();
        assert!(code < CODE_TABLE_LEN);
        result[code] = key.lower_case;
        result[code + CODE_TABLE_LEN] = key.upper_case;
    }
    result
}

pub fn code_tables_listing() -> String {
    const CODES_PER_ROW: usize = 24;
    let mut result = String::new();
    let codes = code_tables();
    for s in 0..2 {
        let mut i = 0;
        while i < CODE_TABLE_LEN {
            let mut j = usize::min(i + CODES_PER_ROW, CODE_TABLE_LEN);
            while j > i {
                j -= 1;
                result.push_str(&format!("{:02X} ", codes[j + s * CODE_TABLE_LEN]));
            }
            result.push_str("\\\\\n");
            i += CODES_PER_ROW;
        }
        if s == 0 {
            result.push('\n');
        }
    }
    result
}

pub fn code_tables_choices() -> String {
    let mut choices = HashMap::new();
    for key in KEYS {
        if key.press.starts_with("E0,") || key.press.starts_with("E1,") {
            continue;
        }
        if key.name.contains("Shift") {
            continue;
        }
        if key.lower_case > 127 || key.upper_case > 127 {
            assert_eq!(key.lower_case, key.upper_case);
            choices.insert(key.name, key.lower_case);
        }
    }
    let mut sorted_choices: Vec<(&&str, &u8)> = choices.iter().collect();
    sorted_choices.sort_by(|a, b| a.1.cmp(b.1));

    let mut result = String::new();
    const NUM_COLUMNS: usize = 4;
    let num_rows = (sorted_choices.len() + NUM_COLUMNS - 1) / NUM_COLUMNS;
    for i in 0..num_rows {
        let mut row = String::new();
        for j in 0..NUM_COLUMNS {
            let choice = &sorted_choices.get(i + j * num_rows);
            if let Some((name, char)) = choice {
                row.push_str(&format!("\\makecell{{{}}} & {:02X}", name, char));
            } else {
                row.push_str(" & ");
            }
            if j == NUM_COLUMNS - 1 {
                if i < num_rows - 1 {
                    row.push_str("\\\\");
                }
            } else {
                row.push_str(" & ");
            }
        }
        result.push_str(&row);
    }
    result
}

#[cfg(test)]
mod tests {
    use crate::keyboard::code_tables;

    use super::KEYS;
    use std::collections::HashSet;

    #[test]
    fn test_keys() {
        let mut extended_codes = HashSet::new();
        for key in KEYS {
            if let Some(scancode) = key.press.strip_prefix("E0,") {
                let code = if scancode == "12,E0,7C" {
                    0x7C + 128
                } else {
                    u8::from_str_radix(scancode, 16).unwrap() + 128
                };
                assert_eq!(key.lower_case, code);
                assert_eq!(key.upper_case, code);
                assert!(extended_codes.insert(code));
            } else if key.press.starts_with("E1,") {
                assert_eq!(key.press, "E1,14,77,E1,F0,14,F0,77");
                assert_eq!(key.lower_case, 0);
                assert_eq!(key.upper_case, 0);
            } else if key.lower_case > 127 || key.upper_case > 127 {
                assert_eq!(key.lower_case, key.upper_case);
                assert!(extended_codes.insert(key.lower_case));
            }
        }
    }

    #[test]
    fn print_code_table() {
        let mut i = 0;
        for value in code_tables() {
            print!("{value}, ");
            if i > 0 && i % 16 == 0 {
                println!();
            }
            i += 1;
            if i == 132 {
                println!();
                i = 0;
            }
        }
        println!();
    }
}
