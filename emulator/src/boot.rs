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

use crate::bus::BusMatrix;
use std::fmt::Write;

#[derive(Clone)]
enum Command {
    GetVersion,
    ReadByte,
    WriteByte,
    ReadHalfWord,
    WriteHalfWord,
    ReadWord,
    WriteWord,
    Go,
    None,
}

/// A SAM3X Boot Program emulator. See section 20, p318 of the Atmel SAM3X Datasheet.
#[derive(Clone)]
pub struct BootMonitor {
    command: Command,
    address: u32,
    value: u32,
    input: String,
    output: String,
}

impl BootMonitor {
    pub fn new() -> Self {
        Self {
            command: Command::None,
            address: 0,
            value: 0,
            input: String::new(),
            output: String::new(),
        }
    }

    pub fn parse_input(&mut self, input: &str, bus_matrix: &mut BusMatrix) -> Option<u32> {
        self.input.push_str(input);
        let mut parsed_count = 0;
        let mut result = Option::None;
        for c in self.input.chars() {
            parsed_count += 1;
            match c {
                'V' => self.command = Command::GetVersion,
                'o' => {
                    self.command = Command::ReadByte;
                    self.value = 0;
                }
                'O' => {
                    self.command = Command::WriteByte;
                    self.value = 0;
                }
                'h' => {
                    self.command = Command::ReadHalfWord;
                    self.value = 0;
                }
                'H' => {
                    self.command = Command::WriteHalfWord;
                    self.value = 0;
                }
                'w' => {
                    self.command = Command::ReadWord;
                    self.value = 0;
                }
                'W' => {
                    self.command = Command::WriteWord;
                    self.value = 0;
                }
                'G' => {
                    self.command = Command::Go;
                    self.value = 0;
                }
                '0'..='9' => self.value = self.value.wrapping_shl(4) | (c as u32 - '0' as u32),
                'A'..='F' => self.value = self.value.wrapping_shl(4) | (c as u32 - 'A' as u32 + 10),
                'a'..='f' => self.value = self.value.wrapping_shl(4) | (c as u32 - 'a' as u32 + 10),
                ',' => {
                    self.address = self.value;
                    self.value = 0;
                }
                '#' => {
                    match self.command {
                        Command::GetVersion => {
                            write!(self.output, "v1.1 Dec 15 2010 19:25:04\n>").unwrap()
                        }
                        Command::ReadByte => {
                            let byte = bus_matrix.get8(self.address);
                            write!(self.output, "{:#04X}\n>", byte).unwrap()
                        }
                        Command::WriteByte => {
                            bus_matrix.set8(self.address, self.value as u8);
                            write!(self.output, "\n>").unwrap()
                        }
                        Command::ReadHalfWord => {
                            let half_word = bus_matrix.get16(self.address);
                            write!(self.output, "{:#06X}\n>", half_word).unwrap()
                        }
                        Command::WriteHalfWord => {
                            bus_matrix.set16(self.address, self.value as u16);
                            write!(self.output, "\n>").unwrap()
                        }
                        Command::ReadWord => {
                            let word = bus_matrix.get32(self.address);
                            write!(self.output, "{:#010X}\n>", word).unwrap()
                        }
                        Command::WriteWord => {
                            bus_matrix.set32(self.address, self.value);
                            write!(self.output, "\n>").unwrap()
                        }
                        Command::Go => {
                            result = Option::Some(self.value);
                            break;
                        }
                        Command::None => write!(self.output, "\n>").unwrap(),
                    }
                    self.command = Command::None;
                }
                '\n' | '\r' => (),
                _ => self.command = Command::None,
            }
        }
        self.input.drain(..parsed_count);
        result
    }

    pub fn write_prompt(&mut self) {
        write!(self.output, "\n>").unwrap();
    }

    pub fn get_output(&mut self) -> String {
        let mut result = String::new();
        std::mem::swap(&mut self.output, &mut result);
        result
    }
}

#[cfg(test)]
mod tests {
    use super::BootMonitor;
    use crate::bus::BusMatrix;

    #[test]
    fn parse_input_get_version() {
        let mut monitor = BootMonitor::new();
        let mut bus_matrix = BusMatrix::default();

        let result = monitor.parse_input("V#\n", &mut bus_matrix);

        assert_eq!(result, Option::None);
        assert_eq!(monitor.get_output(), "v1.1 Dec 15 2010 19:25:04\n>");
    }

    #[test]
    fn parse_input_get_version_extra_characters() {
        let mut monitor = BootMonitor::new();
        let mut bus_matrix = BusMatrix::default();

        let result = monitor.parse_input("V0Ab#\n", &mut bus_matrix);

        assert_eq!(result, Option::None);
        assert_eq!(monitor.get_output(), "v1.1 Dec 15 2010 19:25:04\n>");
    }

    #[test]
    fn parse_input_get_version_unsupported_extra_characters() {
        let mut monitor = BootMonitor::new();
        let mut bus_matrix = BusMatrix::default();

        let result = monitor.parse_input("Vz#\n", &mut bus_matrix);

        assert_eq!(result, Option::None);
        assert_eq!(monitor.get_output(), "\n>");
    }

    #[test]
    fn parse_input_command_character_inside_command_overrides_previous_one() {
        let mut monitor = BootMonitor::new();
        let mut bus_matrix = BusMatrix::default();

        let result = monitor.parse_input("w20070000V#\n", &mut bus_matrix);

        assert_eq!(result, Option::None);
        assert_eq!(monitor.get_output(), "v1.1 Dec 15 2010 19:25:04\n>");
    }

    #[test]
    fn parse_input_read_byte() {
        let mut monitor = BootMonitor::new();
        let mut bus_matrix = BusMatrix::default();
        bus_matrix.set32(0x20071000, 0x11223344);

        let result = monitor.parse_input("o20071001,#\n", &mut bus_matrix);

        assert_eq!(result, Option::None);
        assert_eq!(monitor.get_output(), "0x33\n>");
    }

    #[test]
    fn parse_input_write_byte() {
        let mut monitor = BootMonitor::new();
        let mut bus_matrix = BusMatrix::default();
        bus_matrix.set32(0x20071000, 0x11223344);

        let result = monitor.parse_input("O20071001,AA#", &mut bus_matrix);

        assert_eq!(result, Option::None);
        assert_eq!(monitor.get_output(), "\n>");
        assert_eq!(bus_matrix.get32(0x20071000), 0x1122AA44);
    }

    #[test]
    fn parse_input_write_byte_value_overflow() {
        let mut monitor = BootMonitor::new();
        let mut bus_matrix = BusMatrix::default();
        bus_matrix.set32(0x20071000, 0x11223344);

        let result = monitor.parse_input("O20071001,BBAA#", &mut bus_matrix);

        assert_eq!(result, Option::None);
        assert_eq!(monitor.get_output(), "\n>");
        assert_eq!(bus_matrix.get32(0x20071000), 0x1122AA44);
    }

    #[test]
    fn parse_input_read_half_word() {
        let mut monitor = BootMonitor::new();
        let mut bus_matrix = BusMatrix::default();
        bus_matrix.set32(0x20071000, 0x11223344);

        let result = monitor.parse_input("h20071001,#\n", &mut bus_matrix);

        assert_eq!(result, Option::None);
        assert_eq!(monitor.get_output(), "0x2233\n>");
    }

    #[test]
    fn parse_input_write_half_word() {
        let mut monitor = BootMonitor::new();
        let mut bus_matrix = BusMatrix::default();
        bus_matrix.set32(0x20071000, 0x11223344);

        let result = monitor.parse_input("H20071001,AABB#", &mut bus_matrix);

        assert_eq!(result, Option::None);
        assert_eq!(monitor.get_output(), "\n>");
        assert_eq!(bus_matrix.get32(0x20071000), 0x11AABB44);
    }

    #[test]
    fn parse_input_write_half_word_overflow() {
        let mut monitor = BootMonitor::new();
        let mut bus_matrix = BusMatrix::default();
        bus_matrix.set32(0x20071000, 0x11223344);

        let result = monitor.parse_input("H20071001,AABBCC#", &mut bus_matrix);

        assert_eq!(result, Option::None);
        assert_eq!(monitor.get_output(), "\n>");
        assert_eq!(bus_matrix.get32(0x20071000), 0x11BBCC44);
    }

    #[test]
    fn parse_input_read_word() {
        let mut monitor = BootMonitor::new();
        let mut bus_matrix = BusMatrix::default();
        bus_matrix.set32(0x2007A000, 0x0000CAFE);

        let result = monitor.parse_input("w2007\na000,#\n", &mut bus_matrix);

        assert_eq!(result, Option::None);
        assert_eq!(monitor.get_output(), "0x0000CAFE\n>");
    }

    #[test]
    fn parse_input_read_word_address_overflow() {
        let mut monitor = BootMonitor::new();
        let mut bus_matrix = BusMatrix::default();
        bus_matrix.set32(0x2007A000, 0x0000CAFE);

        let result = monitor.parse_input("wF2007A000,#\n", &mut bus_matrix);

        assert_eq!(result, Option::None);
        assert_eq!(monitor.get_output(), "0x0000CAFE\n>");
    }

    #[test]
    fn parse_input_read_word_multiple_addresses() {
        let mut monitor = BootMonitor::new();
        let mut bus_matrix = BusMatrix::default();
        bus_matrix.set32(0x2007A000, 0x0000CAFE);
        bus_matrix.set32(0x2007A004, 0x0000DECA);

        let result = monitor.parse_input("w2007a000,2007a004,#\n", &mut bus_matrix);

        assert_eq!(result, Option::None);
        assert_eq!(monitor.get_output(), "0x0000DECA\n>");
    }

    #[test]
    fn parse_input_read_word_without_address_uses_previous_address() {
        let mut monitor = BootMonitor::new();
        let mut bus_matrix = BusMatrix::default();
        bus_matrix.set32(0x2007A000, 0x0000CAFE);

        let result1 = monitor.parse_input("w2007a000,#\n", &mut bus_matrix);
        let result2 = monitor.parse_input("w#\n", &mut bus_matrix);

        assert_eq!(result1, Option::None);
        assert_eq!(result2, Option::None);
        assert_eq!(monitor.get_output(), "0x0000CAFE\n>0x0000CAFE\n>");
    }

    #[test]
    fn parse_input_read_word_address_computed_at_comma() {
        let mut monitor = BootMonitor::new();
        let mut bus_matrix = BusMatrix::default();
        bus_matrix.set32(0x2007A000, 0x0000CAFE);
        bus_matrix.set32(0x2007A004, 0x0000DECA);

        let result1 = monitor.parse_input("w2007a000,#\n", &mut bus_matrix);
        let result2 = monitor.parse_input("w2007a004#\n", &mut bus_matrix);

        assert_eq!(result1, Option::None);
        assert_eq!(result2, Option::None);
        assert_eq!(monitor.get_output(), "0x0000CAFE\n>0x0000CAFE\n>");
    }

    #[test]
    fn parse_input_read_word_value_ignored() {
        let mut monitor = BootMonitor::new();
        let mut bus_matrix = BusMatrix::default();
        bus_matrix.set32(0x2007A000, 0x0000CAFE);

        let result = monitor.parse_input("w2007A000,DECA#\n", &mut bus_matrix);

        assert_eq!(result, Option::None);
        assert_eq!(monitor.get_output(), "0x0000CAFE\n>");
    }

    #[test]
    fn parse_input_write_word() {
        let mut monitor = BootMonitor::new();
        let mut bus_matrix = BusMatrix::default();

        let result = monitor.parse_input("W20071000,0000CAFE#", &mut bus_matrix);

        assert_eq!(result, Option::None);
        assert_eq!(monitor.get_output(), "\n>");
        assert_eq!(bus_matrix.get32(0x20071000), 0x0000CAFE);
    }

    #[test]
    fn parse_input_write_word_value_overflow() {
        let mut monitor = BootMonitor::new();
        let mut bus_matrix = BusMatrix::default();

        let result = monitor.parse_input("W20071000,DECAF0000CAFE#", &mut bus_matrix);

        assert_eq!(result, Option::None);
        assert_eq!(monitor.get_output(), "\n>");
        assert_eq!(bus_matrix.get32(0x20071000), 0x0000CAFE);
    }

    #[test]
    fn parse_input_write_word_multiple_addresses() {
        let mut monitor = BootMonitor::new();
        let mut bus_matrix = BusMatrix::default();
        bus_matrix.set32(0x20071000, 0x0000DECA);

        let result = monitor.parse_input("W20071000,20071004,0000CAFE#", &mut bus_matrix);

        assert_eq!(result, Option::None);
        assert_eq!(monitor.get_output(), "\n>");
        assert_eq!(bus_matrix.get32(0x20071000), 0x0000DECA);
        assert_eq!(bus_matrix.get32(0x20071004), 0x0000CAFE);
    }

    #[test]
    fn parse_input_write_word_without_address_uses_previous_address() {
        let mut monitor = BootMonitor::new();
        let mut bus_matrix = BusMatrix::default();
        bus_matrix.set32(0x20071000, 0x0000DECA);

        let result1 = monitor.parse_input("w20071000,#", &mut bus_matrix);
        let result2 = monitor.parse_input("W0000CAFE#", &mut bus_matrix);

        assert_eq!(result1, Option::None);
        assert_eq!(result2, Option::None);
        assert_eq!(monitor.get_output(), "0x0000DECA\n>\n>");
        assert_eq!(bus_matrix.get32(0x20071000), 0x0000CAFE);
    }

    #[test]
    fn parse_input_write_word_without_value_uses_zero_value() {
        let mut monitor = BootMonitor::new();
        let mut bus_matrix = BusMatrix::default();
        bus_matrix.set32(0x20071000, 0x0000DECA);

        let result = monitor.parse_input("W20071000,#", &mut bus_matrix);

        assert_eq!(result, Option::None);
        assert_eq!(monitor.get_output(), "\n>");
        assert_eq!(bus_matrix.get32(0x20071000), 0);
    }

    #[test]
    fn parse_input_go() {
        let mut monitor = BootMonitor::new();
        let mut bus_matrix = BusMatrix::default();

        let result = monitor.parse_input("G20071000#", &mut bus_matrix);

        assert_eq!(result, Option::Some(0x20071000));
        assert_eq!(monitor.get_output(), "");
    }

    #[test]
    fn parse_input_buffer_input() {
        let mut monitor = BootMonitor::new();
        let mut bus_matrix = BusMatrix::default();

        let result1 = monitor.parse_input("G20071000#G20072000#", &mut bus_matrix);
        let result2 = monitor.parse_input("", &mut bus_matrix);

        assert_eq!(result1, Option::Some(0x20071000));
        assert_eq!(result2, Option::Some(0x20072000));
        assert_eq!(monitor.get_output(), "");
    }

    #[test]
    fn parse_input_empty_command() {
        let mut monitor = BootMonitor::new();
        let mut bus_matrix = BusMatrix::default();
        bus_matrix.set32(0x2007A000, 0x0000CAFE);

        let result1 = monitor.parse_input("w2007a000,#\n", &mut bus_matrix);
        let result2 = monitor.parse_input("#\n", &mut bus_matrix);

        assert_eq!(result1, Option::None);
        assert_eq!(result2, Option::None);
        assert_eq!(monitor.get_output(), "0x0000CAFE\n>\n>");
    }

    #[test]
    fn get_output() {
        let mut monitor = BootMonitor::new();

        monitor.write_prompt();
        let output = monitor.get_output();

        assert_eq!(output, "\n>");
    }

    #[test]
    fn get_output_interleaved() {
        let mut monitor = BootMonitor::new();
        let mut bus_matrix = BusMatrix::default();
        bus_matrix.set32(0x20070000, 0x0000CAFE);
        bus_matrix.set32(0x20070004, 0xDECA0000);

        let result1 = monitor.parse_input("w20070000,#\n", &mut bus_matrix);
        let output1 = monitor.get_output();
        let result2 = monitor.parse_input("w20070004,#\n", &mut bus_matrix);
        let output2 = monitor.get_output();

        assert_eq!(result1, Option::None);
        assert_eq!(output1, "0x0000CAFE\n>");
        assert_eq!(result2, Option::None);
        assert_eq!(output2, "0xDECA0000\n>");
    }
}
