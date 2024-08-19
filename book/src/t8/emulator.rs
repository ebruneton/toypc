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

pub struct T8Emulator {
    memory: [u8; 32],
    program_counter: usize,
    r0: u8,
    carry: bool,
}

impl T8Emulator {
    pub fn new() -> Self {
        let memory = [
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 56, 95, 24,
            224, 160, 152, 1,
        ];
        Self {
            memory,
            program_counter: 28,
            r0: 0,
            carry: false,
        }
    }

    pub fn emulate(&mut self, program: &[u8], inputs: &[u8], max_outputs: usize) -> Vec<u8> {
        let mut all_inputs = Vec::new();
        for &value in program {
            all_inputs.push(value);
            if value == 0 {
                break;
            }
        }
        if all_inputs.last() != Some(&0) {
            all_inputs.push(0);
        }
        for &value in inputs {
            all_inputs.push(value);
        }

        let mut next_input = 0;
        let mut outputs = Vec::new();
        loop {
            let insn = self.memory[self.program_counter];
            let address = (insn & 0b11111) as usize;
            match insn >> 5 {
                0b000 => self.store(address, self.r0),
                0b001 => self.r0 = self.load(address),
                0b010 => self.add(self.load(address)),
                0b011 => self.subtract(self.load(address)),
                0b100 => {
                    self.program_counter = address;
                    continue;
                }
                0b101 => {
                    if self.r0 == 0 {
                        self.program_counter = address;
                        continue;
                    }
                }
                0b110 => {
                    if self.carry {
                        self.program_counter = address;
                        continue;
                    }
                }
                0b111 => {
                    if insn & 0b00010000 == 0 {
                        if next_input < all_inputs.len() {
                            self.r0 = all_inputs[next_input];
                            next_input += 1;
                        } else {
                            break;
                        }
                    } else {
                        if outputs.len() == max_outputs {
                            break;
                        }
                        outputs.push(self.r0);
                    }
                }
                _ => panic!("Internal error"),
            }
            self.program_counter += 1;
        }
        outputs
    }

    fn load(&self, address: usize) -> u8 {
        self.memory[address]
    }

    fn store(&mut self, address: usize, value: u8) {
        if address < 25 {
            self.memory[address] = value;
        }
    }

    fn add(&mut self, value: u8) {
        let result = self.r0 as u32 + value as u32;
        self.r0 = result as u8;
        self.carry = (result >> 8) != 0;
    }

    fn subtract(&mut self, value: u8) {
        let result = self.r0 as i32 - value as i32;
        self.r0 = result as u8;
        self.carry = (result >> 8) != 0;
    }
}
