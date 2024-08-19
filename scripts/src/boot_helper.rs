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

use std::cell::RefCell;

use emulator::MicroController as SerialPort;

pub fn run(serial_port: &RefCell<SerialPort>, command: &str) -> Result<Option<String>, String> {
    if !serial_port.borrow_mut().set_serial_input(command) {
        return Err(String::from("ERROR: no response from device.\n"));
    }
    if command.ends_with('#') {
        let output = serial_port.borrow_mut().get_serial_output();
        Ok(Some(
            output.trim_start().strip_suffix('>').unwrap().to_string(),
        ))
    } else {
        Ok(None)
    }
}

const COMMAND_LINE: &str = "user@host:~$ python3 boot_helper.py\n>";

pub struct BootHelper<'a> {
    serial_port: &'a RefCell<SerialPort>,
    terminal: bool,
    output: String,
}

impl<'a> BootHelper<'a> {
    pub fn new(serial_port: &'a RefCell<SerialPort>) -> Self {
        Self::create(serial_port, false)
    }

    pub fn create(serial_port: &'a RefCell<SerialPort>, terminal: bool) -> Self {
        Self {
            serial_port,
            terminal,
            output: if terminal {
                String::from("")
            } else {
                String::from(COMMAND_LINE)
            },
        }
    }

    pub fn write(&mut self, commands: &str) -> bool {
        if !self.terminal {
            self.output.push_str(commands);
            self.output.push('\n');
        }
        for command in commands.split_inclusive('#') {
            if command.trim() == "exit#" {
                return false;
            }
            match run(self.serial_port, command) {
                Ok(Some(result)) => {
                    self.output.push_str(&result);
                    self.output.push('>');
                }
                Ok(None) => (),
                Err(error) => {
                    self.output.push_str(&error);
                    return false;
                }
            }
        }
        true
    }

    pub fn read(&mut self) -> String {
        Self::read_output(&mut self.output, self.terminal)
    }

    pub fn read_output(output: &mut String, terminal: bool) -> String {
        let mut result = String::new();
        if terminal {
            std::mem::swap(output, &mut result);
        } else if output.ends_with("\n>") {
            output.truncate(output.len() - 2);
            std::mem::swap(output, &mut result);
            output.push('>');
        } else if output.ends_with('\n') {
            output.pop();
            std::mem::swap(output, &mut result);
        } else {
            std::mem::swap(output, &mut result);
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use crate::{boot_helper::COMMAND_LINE, BootHelper};
    use emulator::MicroController;

    #[test]
    fn get_version() {
        let micro_controller = RefCell::new(MicroController::default());
        let mut boot_helper = BootHelper::new(&micro_controller);

        boot_helper.write("V#");
        boot_helper.write("exit#");

        assert_eq!(
            boot_helper.read(),
            COMMAND_LINE.to_string() + "V#\nv1.1 Dec 15 2010 19:25:04\n>exit#"
        );
    }

    #[test]
    fn write_and_read_word() {
        let micro_controller = RefCell::new(MicroController::default());
        let mut boot_helper = BootHelper::new(&micro_controller);

        boot_helper.write("W20071000,DECACAFE#");
        boot_helper.write("w20071000,#");
        boot_helper.write("exit#");

        assert_eq!(
            boot_helper.read(),
            COMMAND_LINE.to_string() + "W20071000,DECACAFE#\n>w20071000,#\n0xDECACAFE\n>exit#"
        );
    }

    #[test]
    fn truncated_command() {
        let micro_controller = RefCell::new(MicroController::default());
        let mut boot_helper = BootHelper::new(&micro_controller);

        boot_helper.write("W20071000");
        boot_helper.write(",DECACAFE#");
        boot_helper.write("w20071000,#");
        boot_helper.write("exit#");

        assert_eq!(
            boot_helper.read(),
            COMMAND_LINE.to_string() + "W20071000\n,DECACAFE#\n>w20071000,#\n0xDECACAFE\n>exit#"
        );
    }

    #[test]
    fn multiple_commands() {
        let micro_controller = RefCell::new(MicroController::default());
        let mut boot_helper = BootHelper::new(&micro_controller);

        boot_helper.write("W20071000,DECACAFE#");
        boot_helper.write("w20071000,#w20071000,#");
        boot_helper.write("exit#");

        assert_eq!(
            boot_helper.read(),
            COMMAND_LINE.to_string()
                + "W20071000,DECACAFE#\n>w20071000,#w20071000,#\n"
                + "0xDECACAFE\n>0xDECACAFE\n>exit#"
        );
    }

    #[test]
    fn multiple_reads() {
        let micro_controller = RefCell::new(MicroController::default());
        let mut boot_helper = BootHelper::new(&micro_controller);

        boot_helper.write("W20071000,DECACAFE#");
        boot_helper.write("w20071000,#");
        let str1 = boot_helper.read();
        boot_helper.write("W20071000,CAFEDECA#");
        boot_helper.write("w20071000,#");
        boot_helper.write("exit#");
        let str2 = boot_helper.read();

        assert_eq!(
            str1,
            COMMAND_LINE.to_string() + "W20071000,DECACAFE#\n>w20071000,#\n0xDECACAFE"
        );
        assert_eq!(
            str2,
            ">W20071000,CAFEDECA#\n>w20071000,#\n0xCAFEDECA\n>exit#"
        );
    }
}
