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

mod chapter4_generator;
mod chapter5_generator;

pub fn generate() -> std::io::Result<()> {
    maybe_generate(
        "",
        vec!["src/part1/chapter4_generator.rs"],
        "part1/generated/chapter4/output.bin",
        chapter4_generator::generate,
    )?;
    maybe_generate(
        "",
        vec!["src/part1/chapter5_generator.rs"],
        "part1/generated/chapter5/output.bin",
        chapter5_generator::generate,
    )
}
