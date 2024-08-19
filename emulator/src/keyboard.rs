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

/// A US Qwerty PS/2 keyboard emulator. See https://wiki.osdev.org/PS/2_Keyboard#Scan_Code_Set_2
/// and http://www.quadibloc.com/comp/scan.htm.
#[derive(Clone)]
pub struct Keyboard {
    key_pressed_scancodes: HashMap<&'static str, Vec<u8>>,
    key_released_scancodes: HashMap<&'static str, Vec<u8>>,
    empty_scancodes: Vec<u8>,
}

impl Default for Keyboard {
    fn default() -> Self {
        Self::new()
    }
}

impl Keyboard {
    pub fn new() -> Self {
        let mut key_pressed = HashMap::new();
        key_pressed.insert("0", vec![0x45]);
        key_pressed.insert("1", vec![0x16]);
        key_pressed.insert("2", vec![0x1E]);
        key_pressed.insert("3", vec![0x26]);
        key_pressed.insert("4", vec![0x25]);
        key_pressed.insert("5", vec![0x2E]);
        key_pressed.insert("6", vec![0x36]);
        key_pressed.insert("7", vec![0x3D]);
        key_pressed.insert("8", vec![0x3E]);
        key_pressed.insert("9", vec![0x46]);
        key_pressed.insert("A", vec![0x1C]);
        key_pressed.insert("Backspace", vec![0x66]);
        key_pressed.insert("`", vec![0x0E]);
        key_pressed.insert("B", vec![0x32]);
        key_pressed.insert("CapsLock", vec![0x58]);
        key_pressed.insert("C", vec![0x21]);
        key_pressed.insert("ArrowDown", vec![0xE0, 0x72]);
        key_pressed.insert("ArrowLeft", vec![0xE0, 0x6B]);
        key_pressed.insert("ArrowRight", vec![0xE0, 0x74]);
        key_pressed.insert("ArrowUp", vec![0xE0, 0x75]);
        key_pressed.insert("Delete", vec![0xE0, 0x71]);
        key_pressed.insert("D", vec![0x23]);
        key_pressed.insert("End", vec![0xE0, 0x69]);
        key_pressed.insert("Enter", vec![0x5A]);
        key_pressed.insert("E", vec![0x24]);
        key_pressed.insert("Escape", vec![0x76]);
        key_pressed.insert("F10", vec![0x09]);
        key_pressed.insert("F11", vec![0x78]);
        key_pressed.insert("F12", vec![0x07]);
        key_pressed.insert("F1", vec![0x05]);
        key_pressed.insert("F2", vec![0x06]);
        key_pressed.insert("F3", vec![0x04]);
        key_pressed.insert("F4", vec![0x0C]);
        key_pressed.insert("F5", vec![0x03]);
        key_pressed.insert("F6", vec![0x0B]);
        key_pressed.insert("F7", vec![0x83]);
        key_pressed.insert("F8", vec![0x0A]);
        key_pressed.insert("F9", vec![0x01]);
        key_pressed.insert("F", vec![0x2B]);
        key_pressed.insert("G", vec![0x34]);
        key_pressed.insert("Home", vec![0xE0, 0x6C]);
        key_pressed.insert("H", vec![0x33]);
        key_pressed.insert("Insert", vec![0xE0, 0x70]);
        key_pressed.insert("I", vec![0x43]);
        key_pressed.insert("J", vec![0x3B]);
        key_pressed.insert("K", vec![0x42]);
        key_pressed.insert("Alt", vec![0x11]);
        key_pressed.insert("Control", vec![0x14]);
        key_pressed.insert("Shift", vec![0x12]);
        key_pressed.insert("L", vec![0x4B]);
        key_pressed.insert("M", vec![0x3A]);
        key_pressed.insert("N", vec![0x31]);
        key_pressed.insert("NumLock", vec![0x77]);
        key_pressed.insert("ScrollLock", vec![0x7E]);
        key_pressed.insert("O", vec![0x44]);
        key_pressed.insert("PageDown", vec![0xE0, 0x7A]);
        key_pressed.insert("PageUp", vec![0xE0, 0x7D]);
        key_pressed.insert("PrintScreen", vec![0xE0, 0x12, 0xE0, 0x7C]);
        key_pressed.insert(
            "Pause",
            vec![0xE1, 0x14, 0x77, 0xE1, 0xF0, 0x14, 0xF0, 0x77],
        );
        key_pressed.insert("P", vec![0x4D]);
        key_pressed.insert(",", vec![0x41]);
        key_pressed.insert(".", vec![0x49]);
        key_pressed.insert("/", vec![0x4A]);
        key_pressed.insert(";", vec![0x4C]);
        key_pressed.insert("-", vec![0x4E]);
        key_pressed.insert("'", vec![0x52]);
        key_pressed.insert("[", vec![0x54]);
        key_pressed.insert("=", vec![0x55]);
        key_pressed.insert("]", vec![0x5B]);
        key_pressed.insert("\\", vec![0x5D]);
        key_pressed.insert("Q", vec![0x15]);
        key_pressed.insert("R", vec![0x2D]);
        key_pressed.insert(" ", vec![0x29]);
        key_pressed.insert("S", vec![0x1B]);
        key_pressed.insert("Tab", vec![0x0D]);
        key_pressed.insert("T", vec![0x2C]);
        key_pressed.insert("U", vec![0x3C]);
        key_pressed.insert("V", vec![0x2A]);
        key_pressed.insert("W", vec![0x1D]);
        key_pressed.insert("X", vec![0x22]);
        key_pressed.insert("Y", vec![0x35]);
        key_pressed.insert("Z", vec![0x1A]);

        let mut key_released = HashMap::new();
        key_released.insert("0", vec![0xF0, 0x45]);
        key_released.insert("1", vec![0xF0, 0x16]);
        key_released.insert("2", vec![0xF0, 0x1E]);
        key_released.insert("3", vec![0xF0, 0x26]);
        key_released.insert("4", vec![0xF0, 0x25]);
        key_released.insert("5", vec![0xF0, 0x2E]);
        key_released.insert("6", vec![0xF0, 0x36]);
        key_released.insert("7", vec![0xF0, 0x3D]);
        key_released.insert("8", vec![0xF0, 0x3E]);
        key_released.insert("9", vec![0xF0, 0x46]);
        key_released.insert("A", vec![0xF0, 0x1C]);
        key_released.insert("Backspace", vec![0xF0, 0x66]);
        key_released.insert("`", vec![0xF0, 0x0E]);
        key_released.insert("B", vec![0xF0, 0x32]);
        key_released.insert("CapsLock", vec![0xF0, 0x58]);
        key_released.insert("C", vec![0xF0, 0x21]);
        key_released.insert("ArrowDown", vec![0xE0, 0xF0, 0x72]);
        key_released.insert("ArrowLeft", vec![0xE0, 0xF0, 0x6B]);
        key_released.insert("ArrowRight", vec![0xE0, 0xF0, 0x74]);
        key_released.insert("ArrowUp", vec![0xE0, 0xF0, 0x75]);
        key_released.insert("Delete", vec![0xE0, 0xF0, 0x71]);
        key_released.insert("D", vec![0xF0, 0x23]);
        key_released.insert("End", vec![0xE0, 0xF0, 0x69]);
        key_released.insert("Enter", vec![0xF0, 0x5A]);
        key_released.insert("E", vec![0xF0, 0x24]);
        key_released.insert("Escape", vec![0xF0, 0x76]);
        key_released.insert("F10", vec![0xF0, 0x09]);
        key_released.insert("F11", vec![0xF0, 0x78]);
        key_released.insert("F12", vec![0xF0, 0x07]);
        key_released.insert("F1", vec![0xF0, 0x05]);
        key_released.insert("F2", vec![0xF0, 0x06]);
        key_released.insert("F3", vec![0xF0, 0x04]);
        key_released.insert("F4", vec![0xF0, 0x0C]);
        key_released.insert("F5", vec![0xF0, 0x03]);
        key_released.insert("F6", vec![0xF0, 0x0B]);
        key_released.insert("F7", vec![0xF0, 0x83]);
        key_released.insert("F8", vec![0xF0, 0x0A]);
        key_released.insert("F9", vec![0xF0, 0x01]);
        key_released.insert("F", vec![0xF0, 0x2B]);
        key_released.insert("G", vec![0xF0, 0x34]);
        key_released.insert("Home", vec![0xE0, 0xF0, 0x6C]);
        key_released.insert("H", vec![0xF0, 0x33]);
        key_released.insert("Insert", vec![0xE0, 0xF0, 0x70]);
        key_released.insert("I", vec![0xF0, 0x43]);
        key_released.insert("J", vec![0xF0, 0x3B]);
        key_released.insert("K", vec![0xF0, 0x42]);
        key_released.insert("Alt", vec![0xF0, 0x11]);
        key_released.insert("Control", vec![0xF0, 0x14]);
        key_released.insert("Shift", vec![0xF0, 0x12]);
        key_released.insert("L", vec![0xF0, 0x4B]);
        key_released.insert("M", vec![0xF0, 0x3A]);
        key_released.insert("N", vec![0xF0, 0x31]);
        key_released.insert("NumLock", vec![0xF0, 0x77]);
        key_released.insert("ScrollLock", vec![0xF0, 0x7E]);
        key_released.insert("O", vec![0xF0, 0x44]);
        key_released.insert("PageDown", vec![0xE0, 0xF0, 0x7A]);
        key_released.insert("PageUp", vec![0xE0, 0xF0, 0x7D]);
        key_released.insert("PrintScreen", vec![0xE0, 0xF0, 0x7C, 0xE0, 0xF0, 0x12]);
        key_released.insert("P", vec![0xF0, 0x4D]);
        key_released.insert("Q", vec![0xF0, 0x15]);
        key_released.insert(",", vec![0xF0, 0x41]);
        key_released.insert(".", vec![0xF0, 0x49]);
        key_released.insert("/", vec![0xF0, 0x4A]);
        key_released.insert(";", vec![0xF0, 0x4C]);
        key_released.insert("-", vec![0xF0, 0x4E]);
        key_released.insert("'", vec![0xF0, 0x52]);
        key_released.insert("[", vec![0xF0, 0x54]);
        key_released.insert("=", vec![0xF0, 0x55]);
        key_released.insert("]", vec![0xF0, 0x5B]);
        key_released.insert("\\", vec![0xF0, 0x5D]);
        key_released.insert("R", vec![0xF0, 0x2D]);
        key_released.insert(" ", vec![0xF0, 0x29]);
        key_released.insert("S", vec![0xF0, 0x1B]);
        key_released.insert("Tab", vec![0xF0, 0x0D]);
        key_released.insert("T", vec![0xF0, 0x2C]);
        key_released.insert("U", vec![0xF0, 0x3C]);
        key_released.insert("V", vec![0xF0, 0x2A]);
        key_released.insert("W", vec![0xF0, 0x1D]);
        key_released.insert("X", vec![0xF0, 0x22]);
        key_released.insert("Y", vec![0xF0, 0x35]);
        key_released.insert("Z", vec![0xF0, 0x1A]);
        Self {
            key_pressed_scancodes: key_pressed,
            key_released_scancodes: key_released,
            empty_scancodes: Vec::new(),
        }
    }

    pub fn key_pressed(&self, key: &str) -> &[u8] {
        self.key_pressed_scancodes
            .get(key)
            .unwrap_or(&self.empty_scancodes)
            .as_slice()
    }

    pub fn key_released(&self, key: &str) -> &[u8] {
        self.key_released_scancodes
            .get(key)
            .unwrap_or(&self.empty_scancodes)
            .as_slice()
    }
}

#[cfg(test)]
mod tests {
    use super::Keyboard;

    #[test]
    fn key_pressed() {
        let keyboard = Keyboard::default();

        assert_eq!(keyboard.key_pressed("A"), [0x1C]);
    }

    #[test]
    fn key_released() {
        let keyboard = Keyboard::default();

        assert_eq!(keyboard.key_released("A"), [0xF0, 0x1C]);
    }
}
