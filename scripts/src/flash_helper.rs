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
    cell::RefCell,
    collections::BTreeMap,
    fs::File,
    io::{BufRead, BufReader},
};

use crate::BootHelper;
use emulator::MicroController as SerialPort;

const FLASH_BEGIN: u32 = 0x80000;
const FLASH_END: u32 = 0x100000;
const FLASH_PAGE_BYTES: u32 = 256;
const FLASH_PAGE_WORDS: u32 = FLASH_PAGE_BYTES / 4;
const FLASH_PAGES_PER_CONTROLLER: u32 = (FLASH_END - FLASH_BEGIN) / FLASH_PAGE_BYTES / 2;

fn run(serial_port: &RefCell<SerialPort>, command: &str) -> Result<String, String> {
    match crate::boot_helper::run(serial_port, command) {
        Ok(Some(mut result)) => {
            result.truncate(result.trim_end().len());
            Result::Ok(result)
        }
        Ok(None) => Ok(String::new()),
        Err(error) => Result::Err(error),
    }
}

fn wait_ready(serial_port: &RefCell<SerialPort>, register: u32) -> Result<(), String> {
    let command = format!("w{register:08X},#");
    while run(serial_port, command.as_str())? != "0x00000001" {}
    Ok(())
}

struct Page {
    index: u32,
    address: u32,
    values: [u32; FLASH_PAGE_WORDS as usize],
    dirty: bool,
}

impl Page {
    fn new(
        index: u32,
        serial_port: &RefCell<SerialPort>,
        output: &mut String,
    ) -> Result<Self, String> {
        let mut page = Self {
            index,
            address: index * FLASH_PAGE_BYTES + FLASH_BEGIN,
            values: [0; FLASH_PAGE_WORDS as usize],
            dirty: false,
        };
        output.push_str(format!("Reading page {}...", page.index).as_str());
        for i in 0..FLASH_PAGE_WORDS {
            let address = page.address + 4 * i;
            let value = run(serial_port, &format!("w{address:08X},#"))?;
            page.values[i as usize] =
                u32::from_str_radix(value.trim_start_matches("0x"), 16).unwrap();
        }
        output.push_str(" Done.\n");
        Ok(page)
    }

    fn set(&mut self, index: u32, value: u32) {
        if value != self.values[index as usize] {
            self.values[index as usize] = value;
            self.dirty = true;
        }
    }

    fn flash(&self, serial_port: &RefCell<SerialPort>, output: &mut String) -> Result<(), String> {
        if !self.dirty {
            return Ok(());
        }
        output.push_str(format!("Writing page {}...", self.index).as_str());
        for i in 0..FLASH_PAGE_WORDS {
            let address = self.address + 4 * i;
            let value = self.values[i as usize];
            run(serial_port, &format!("W{address:08X},{value:08X}#"))?;
        }
        if self.index < FLASH_PAGES_PER_CONTROLLER {
            let command = 0x5A000003 | (self.index << 8);
            run(serial_port, &format!("W400E0A04,{command:08X}#"))?;
            wait_ready(serial_port, 0x400E0A08)?;
        } else {
            let command = 0x5A000003 | ((self.index - FLASH_PAGES_PER_CONTROLLER) << 8);
            run(serial_port, &format!("W400E0C04,{command:08X}#"))?;
            wait_ready(serial_port, 0x400E0C08)?;
        }
        for i in 0..FLASH_PAGE_WORDS {
            let address = self.address + 4 * i;
            let value = run(serial_port, &format!("w{address:08X},#"))?;
            let actual = u32::from_str_radix(value.trim_start_matches("0x"), 16).unwrap();
            if actual != self.values[i as usize] {
                return Err(format!(
                    "ERROR: page write failed at address {address:08X}\n"
                ));
            }
        }
        output.push_str(" Done.\n");
        Result::Ok(())
    }
}

const COMMAND_LINE: &str = "user@host:~$ python3 flash_helper.py";

pub struct FlashHelper<'a> {
    serial_port: &'a RefCell<SerialPort>,
    input_from_file: bool,
    terminal: bool,
    pages: BTreeMap<u32, Box<Page>>,
    output: String,
}

impl<'a> FlashHelper<'a> {
    pub fn new(serial_port: &'a RefCell<SerialPort>) -> Self {
        Self::create(serial_port, false)
    }

    pub fn create(serial_port: &'a RefCell<SerialPort>, terminal: bool) -> Self {
        Self {
            serial_port,
            input_from_file: false,
            terminal,
            pages: BTreeMap::<u32, Box<Page>>::default(),
            output: if terminal {
                String::from("")
            } else {
                String::from(&format!("{COMMAND_LINE}\n>"))
            },
        }
    }

    pub fn create_from_file(
        serial_port: &'a RefCell<SerialPort>,
        name: &str,
        terminal: bool,
    ) -> Self {
        Self {
            serial_port,
            input_from_file: true,
            terminal,
            pages: BTreeMap::<u32, Box<Page>>::default(),
            output: if terminal {
                String::from("")
            } else {
                String::from(&format!("{COMMAND_LINE} < {name}\n>"))
            },
        }
    }

    pub fn from_file(
        serial_port: &'a RefCell<SerialPort>,
        directory: &str,
        name: &str,
    ) -> std::io::Result<Self> {
        let mut result = Self::create_from_file(serial_port, name, false);
        let input = File::open(format!("{directory}{name}"))?;
        let lines = BufReader::new(input).lines();
        for line in lines.map_while(Result::ok) {
            result.write(&line);
        }
        result.output.push_str("Done.");
        Ok(result)
    }

    pub fn write(&mut self, commands: &str) -> bool {
        match self.write_internal(commands) {
            Ok(result) => result,
            Err(error) => {
                self.output.push_str(&error);
                false
            }
        }
    }

    fn write_internal(&mut self, commands: &str) -> Result<bool, String> {
        if !self.terminal && !self.input_from_file {
            self.output.push_str(commands);
            self.output.push('\n');
        }
        for command in commands.split_inclusive('#') {
            if command.trim() == "exit#" {
                return Ok(false);
            }
            if command.trim() == "flash#" {
                for page in self.pages.values() {
                    page.flash(self.serial_port, &mut self.output)?;
                }
                self.pages.clear();
                self.output.push('>');
                continue;
            }
            if command.trim() == "reset#" {
                run(self.serial_port, "W400E0A04,5A00010B#")?; // Set boot from flash.
                wait_ready(self.serial_port, 0x400E0A08)?;
                self.serial_port
                    .borrow_mut()
                    .set_serial_input("W400E1A00,A500000D#"); // Reset.
                return Ok(false);
            }
            if let Option::Some((address, value)) =
                Self::get_flash_address_and_value(command.trim())
            {
                let page_index = (address - FLASH_BEGIN) / FLASH_PAGE_BYTES;
                let word_index = (address % FLASH_PAGE_BYTES) / 4;
                if let Some(page) = self.pages.get_mut(&page_index) {
                    page.set(word_index, value);
                } else {
                    let mut page = Page::new(page_index, self.serial_port, &mut self.output)?;
                    page.set(word_index, value);
                    self.pages.insert(page_index, Box::new(page));
                }
                if !self.input_from_file {
                    self.output.push('>');
                }
                continue;
            }
            match crate::boot_helper::run(self.serial_port, command) {
                Ok(Some(result)) => {
                    if !self.input_from_file {
                        self.output.push_str(&result);
                        self.output.push('>');
                    }
                }
                Ok(None) => (),
                Err(error) => return Err(error),
            }
        }
        Ok(true)
    }

    pub fn read(&mut self) -> String {
        BootHelper::read_output(&mut self.output, self.terminal)
    }

    fn get_flash_address_and_value(line: &str) -> Option<(u32, u32)> {
        if !line.starts_with('W') || !line.ends_with('#') {
            return Option::None;
        }
        let comma = line.find(',');
        let len = line.len();
        if let Option::Some(index) = comma {
            if let Result::Ok(address) = u32::from_str_radix(&line[1..index], 16) {
                if let Result::Ok(value) = u32::from_str_radix(&line[index + 1..len - 1], 16) {
                    if (FLASH_BEGIN..FLASH_END).contains(&address) {
                        if address % 4 != 0 {
                            panic!("Invalid address {address}");
                        }
                        return Option::Some((address, value));
                    }
                }
            }
        }
        Option::None
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use crate::{flash_helper::COMMAND_LINE, FlashHelper};
    use emulator::MicroController;

    #[test]
    fn get_version() {
        let controller = RefCell::new(MicroController::default());
        let mut flash_helper = FlashHelper::new(&controller);

        flash_helper.write("V#");
        flash_helper.write("exit#");

        assert_eq!(
            flash_helper.read(),
            COMMAND_LINE.to_string() + "\n>V#\nv1.1 Dec 15 2010 19:25:04\n>exit#"
        );
    }

    #[test]
    fn write_and_read_word() {
        let controller = RefCell::new(MicroController::default());
        let mut flash_helper = FlashHelper::new(&controller);

        flash_helper.write("W20071000,DECACAFE#");
        flash_helper.write("w20071000,#");
        flash_helper.write("exit#");

        assert_eq!(
            flash_helper.read(),
            COMMAND_LINE.to_string() + "\n>W20071000,DECACAFE#\n>w20071000,#\n0xDECACAFE\n>exit#"
        );
    }

    #[test]
    fn flash_two_pages() {
        let controller = RefCell::new(MicroController::default());
        let mut flash_helper = FlashHelper::new(&controller);

        flash_helper.write("W00080114,DECACAFE#");
        flash_helper.write("W00080118,DECA1234#");
        flash_helper.write("W000C0114,CAFEBABE#");
        flash_helper.write("flash#");
        flash_helper.write("w00080118,#");
        flash_helper.write("exit#");

        assert_eq!(
            flash_helper.read(),
            "user@host:~$ python3 flash_helper.py\n\
            >W00080114,DECACAFE#\n\
            Reading page 1... Done.\n\
            >W00080118,DECA1234#\n\
            >W000C0114,CAFEBABE#\n\
            Reading page 1025... Done.\n\
            >flash#\n\
            Writing page 1... Done.\n\
            Writing page 1025... Done.\n\
            >w00080118,#\n\
            0xDECA1234\n\
            >exit#"
        );
    }
}
