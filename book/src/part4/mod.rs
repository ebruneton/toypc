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

use crate::maybe_generate;

mod chapter1_generator;
mod chapter2_generator;
mod chapter3_generator;
mod chapter4_generator;
mod chapter5_generator;
mod chapter6_generator;
mod chapter7_generator;
mod chapter8_generator;

pub fn generate() -> std::io::Result<()> {
    maybe_generate(
        "part3/generated/chapter8/output.bin",
        vec!["src/part4/chapter1_generator.rs"],
        "part4/generated/chapter1/output.bin",
        chapter1_generator::generate,
    )?;
    maybe_generate(
        "part4/generated/chapter1/output.bin",
        vec!["src/part4/chapter2_generator.rs"],
        "part4/generated/chapter2/output.bin",
        chapter2_generator::generate,
    )?;
    maybe_generate(
        "part4/generated/chapter2/output.bin",
        vec!["src/part4/chapter3_generator.rs"],
        "part4/generated/chapter3/output.bin",
        chapter3_generator::generate,
    )?;
    maybe_generate(
        "part4/generated/chapter3/output.bin",
        vec!["src/part4/chapter4_generator.rs"],
        "part4/generated/chapter4/output.bin",
        chapter4_generator::generate,
    )?;
    maybe_generate(
        "part4/generated/chapter4/output.bin",
        vec!["src/part4/chapter5_generator.rs"],
        "part4/generated/chapter5/output.bin",
        chapter5_generator::generate,
    )?;
    maybe_generate(
        "part4/generated/chapter5/output.bin",
        vec!["src/part4/chapter6_generator.rs"],
        "part4/generated/chapter6/output.bin",
        chapter6_generator::generate,
    )?;
    maybe_generate(
        "part4/generated/chapter6/output.bin",
        vec!["src/part4/chapter7_generator.rs"],
        "part4/generated/chapter7/output.bin",
        chapter7_generator::generate,
    )?;
    maybe_generate(
        "part4/generated/chapter7/output.bin",
        vec!["src/part4/chapter8_generator.rs"],
        "part4/generated/chapter8/output.bin",
        chapter8_generator::generate,
    )
}
