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

use context::Context;
use std::{
    fs,
    io::Write,
    path::Path,
    time::{Instant, SystemTime},
};

mod arm;
mod atmel;
mod context;
mod keyboard;
mod part1;
mod part2;
mod part3;
mod part4;
mod raio;
mod t8;
mod toy;
mod util;
mod vm;

fn maybe_generate(
    input: &str,
    sources: Vec<&str>,
    output: &str,
    generator: fn(&mut Context) -> std::io::Result<()>,
) -> std::io::Result<()> {
    let cached_input = output.replace("output.bin", "input.bin");
    if Path::new(&output).exists() {
        let mut source_time = SystemTime::UNIX_EPOCH;
        for source in sources {
            source_time = source_time.max(std::fs::metadata(source).unwrap().modified().unwrap());
        }
        let output_time = std::fs::metadata(output).unwrap().modified().unwrap();
        if input.is_empty() {
            if output_time.gt(&source_time) {
                return Ok(());
            }
        } else if Path::new(&cached_input).exists()
            && std::fs::read(&cached_input)? == std::fs::read(input)?
            && output_time.gt(&source_time)
        {
            return Ok(());
        }
    }
    let mut context = if input.is_empty() {
        Context::new()
    } else {
        fs::File::create(cached_input)?.write_all(&std::fs::read(input)?)?;
        let context = Context::from_file(input)?;
        context.micro_controller().borrow_mut().turn_on();
        context
    };
    println!("Generating {output}...");
    generator(&mut context)?;
    context.micro_controller().borrow_mut().turn_off();
    context.check_memory_regions();
    context.to_file(output)
}

fn main() -> std::io::Result<()> {
    let start = Instant::now();
    fs::create_dir_all("generated")?;
    part1::generate()?;
    part2::generate()?;
    part3::generate()?;
    part4::generate()?;
    fs::write("generated/ascii_table.tex", keyboard::ascii_table())?;
    fs::write("generated/scancode_table.tex", keyboard::scancode_table())?;
    fs::write(
        "generated/boot_helper.tex",
        util::code(
            &fs::read_to_string("../scripts/src/boot_helper.py")?
                .lines()
                .skip(12)
                .collect::<Vec<&str>>()
                .join("\n"),
        ),
    )?;
    fs::write(
        "generated/flash_helper.tex",
        util::code(
            &fs::read_to_string("../scripts/src/flash_helper.py")?
                .lines()
                .skip(12)
                .collect::<Vec<&str>>()
                .join("\n"),
        ),
    )?;
    println!(
        "LaTeX files generated in {}s.",
        Instant::now().duration_since(start).as_secs_f32()
    );
    Ok(())
}
