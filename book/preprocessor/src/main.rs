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
    fs::create_dir_all,
    fs::File,
    io::{BufRead, BufReader, Write},
    path::Path,
    process::Command,
    time::{Instant, SystemTime},
};

const PREAMBLE: &str = r"
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

#![allow(unused_imports)]
use crate::arm::*;
use crate::atmel::*;
use crate::context::{Context, Label, MemoryRegion, RegionKind};
use crate::keyboard;
use crate::raio::*;
use crate::t8::*;
use crate::toy::{Transpiler1, Transpiler2, Transpiler3, Transpiler4, Transpiler5};
use crate::util::*;
use crate::vm::*;
use emulator::{Controller, Keyboard, MicroController, TextDisplay};
use scripts::{BootHelper, FlashHelper};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::rc::Rc;

pub fn generate(context: &mut Context) -> std::io::Result<()> {
";

fn preprocess(directory: &str, file: &str, preprocessor_time: SystemTime) -> std::io::Result<()> {
    create_dir_all(format!("src/{directory}"))?;
    create_dir_all(format!("{directory}/generated/{file}/"))?;

    let input_name = format!("{directory}/{file}.tex");
    let output_name = format!("src/{directory}/{file}_generator.rs");
    if Path::new(&output_name).exists() {
        let input_time = std::fs::metadata(&input_name).unwrap().modified().unwrap();
        let output_time = std::fs::metadata(&output_name).unwrap().modified().unwrap();
        if output_time.gt(&input_time) && output_time.gt(&preprocessor_time) {
            return Ok(());
        }
    }

    let get_generator_variable = |line: &str, variable: &mut Option<char>| {
        let l = line.trim();
        if l.len() > 2 && l.chars().nth(1).unwrap() == '.' {
            *variable = Option::Some(l.chars().next().unwrap());
        }
    };

    let input = File::open(&input_name)?;
    let mut output = Vec::new();
    output.write_all(PREAMBLE[1..].as_bytes())?;

    let lines = BufReader::new(input).lines();
    let mut is_rust_code = false;
    let mut is_byte_code = false;
    let mut byte_code_options = String::new();
    let mut is_toy_code = false;
    let mut file_index = 0;
    let mut generator_variable = Option::None;
    let mut line_buffer = String::new();
    for line in lines.map_while(Result::ok) {
        if line.trim() == r"\startrust" {
            writeln!(&mut output, "{{")?;
        } else if line.trim() == r"\stoprust" {
            writeln!(&mut output, "}}")?;
        } else if line.trim() == r"\rust{" {
            is_rust_code = true;
        } else if line.trim() == r"\bytecode{" {
            is_byte_code = true;
            byte_code_options = String::new();
            generator_variable = Option::None;
        } else if line.starts_with(r"\bytecode[") {
            is_byte_code = true;
            byte_code_options = String::from(&line[10..line.find(']').unwrap()]);
            generator_variable = Option::None;
        } else if line.trim() == r"\toy{" {
            is_toy_code = true;
        } else if is_rust_code {
            if line == "}" {
                is_rust_code = false;
            } else {
                writeln!(&mut output, "{}", line)?;
            }
        } else if is_byte_code {
            if line == "}" {
                writeln!(
                    &mut output,
                    "  {}.write(\"{directory}/generated/{file}/input{}.tex\", \"{}\");",
                    generator_variable.unwrap(),
                    file_index,
                    byte_code_options
                )?;
                file_index += 1;
                is_byte_code = false;
            } else {
                get_generator_variable(&line, &mut generator_variable);
                writeln!(&mut output, "{}", line)?;
            }
        } else if is_toy_code {
            if line == "}%toy" {
                writeln!(
                    &mut output,
                    "  t.write(\"{directory}/generated/{file}/input{}.tex\");",
                    file_index
                )?;
                file_index += 1;
                is_toy_code = false;
            } else if line.ends_with('\\') && !line.ends_with("\\\\") {
                line_buffer.push_str(&line);
            } else if line_buffer.is_empty() {
                writeln!(
                    &mut output,
                    "  t.add(\"{}\");",
                    line.replace("\\{", "{").replace("\\}", "}")
                )?;
            } else {
                line_buffer.pop();
                line_buffer.push_str(&line);
                writeln!(
                    &mut output,
                    "  t.add(\"{}\");",
                    line_buffer.replace("\\{", "{").replace("\\}", "}")
                )?;
                line_buffer.clear();
            }
        } else {
            let mut slice = line.as_str();
            let mut start_rs_index = slice.find(r"\rs{");
            while let Some(start_index) = start_rs_index {
                slice = &slice[start_index + 4..];
                let end_index = slice.find('}').unwrap();
                writeln!(
                    &mut output,
                    "  fs::write(\"{directory}/generated/{file}/input{}.tex\",{})?;",
                    file_index,
                    &slice[0..end_index]
                )?;

                start_rs_index = slice.find(r"\rs{");
                file_index += 1;
            }
        }
    }

    writeln!(&mut output, "Ok(())")?;
    writeln!(&mut output, "}}")?;

    if Path::new(&output_name).exists() && output == std::fs::read(&output_name)? {
        return Ok(());
    }
    File::create(&output_name)?.write_all(&output)
}

fn main() -> std::io::Result<()> {
    if std::env::args().count() > 1 {
        let directory = std::env::args().nth(1).unwrap();
        std::env::set_current_dir(&directory)?;
    }
    let preprocessor_time = std::fs::metadata("preprocessor/src/main.rs")
        .unwrap()
        .modified()
        .unwrap();
    preprocess("part1", "chapter4", preprocessor_time)?;
    preprocess("part1", "chapter5", preprocessor_time)?;
    preprocess("part2", "chapter1", preprocessor_time)?;
    preprocess("part2", "chapter2", preprocessor_time)?;
    preprocess("part2", "chapter3", preprocessor_time)?;
    preprocess("part2", "chapter4", preprocessor_time)?;
    preprocess("part2", "chapter5", preprocessor_time)?;
    preprocess("part2", "chapter6", preprocessor_time)?;
    preprocess("part2", "chapter7", preprocessor_time)?;
    preprocess("part3", "introduction", preprocessor_time)?;
    preprocess("part3", "chapter1", preprocessor_time)?;
    preprocess("part3", "chapter2", preprocessor_time)?;
    preprocess("part3", "chapter3", preprocessor_time)?;
    preprocess("part3", "chapter4", preprocessor_time)?;
    preprocess("part3", "chapter5", preprocessor_time)?;
    preprocess("part3", "chapter6", preprocessor_time)?;
    preprocess("part3", "chapter7", preprocessor_time)?;
    preprocess("part3", "chapter8", preprocessor_time)?;
    preprocess("part4", "chapter1", preprocessor_time)?;
    preprocess("part4", "chapter2", preprocessor_time)?;
    preprocess("part4", "chapter3", preprocessor_time)?;
    preprocess("part4", "chapter4", preprocessor_time)?;
    preprocess("part4", "chapter5", preprocessor_time)?;
    preprocess("part4", "chapter6", preprocessor_time)?;
    preprocess("part4", "chapter7", preprocessor_time)?;
    preprocess("part4", "chapter8", preprocessor_time)?;

    let start = Instant::now();
    let output = Command::new("cargo").args(["run", "--release"]).output()?;
    if output.status.success() {
        println!("{}", String::from_utf8(output.stdout).unwrap());
        println!(
            "LaTeX files preprocessed in {}s.",
            Instant::now().duration_since(start).as_secs_f32()
        );
    } else {
        println!("{}", String::from_utf8(output.stdout).unwrap());
        println!("{}", String::from_utf8(output.stderr).unwrap());
        println!("LaTeX preprocessing FAILED!");
    }
    Ok(())
}
