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

use postcard::{from_bytes, to_allocvec};
use serde::{Deserialize, Serialize, Serializer};
use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet, HashMap},
    fs::File,
    io::{Read, Write},
    path::Path,
    rc::Rc,
};

use emulator::{GraphicsCard, Instruction, Keyboard, MicroController, TextDisplay};

use crate::{arm::Assembler, util::write_lines};

pub fn ordered_map<S, K: Ord + Serialize, V: Serialize>(
    value: &HashMap<K, V>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let ordered: BTreeMap<_, _> = value.iter().collect();
    ordered.serialize(serializer)
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Label {
    pub offset: u32,
    pub description: String,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum RegionKind {
    Default,
    DataBuffer,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct MemoryRegion {
    pub kind: RegionKind,
    pub start: u32,
    pub len: u32,
    #[serde(serialize_with = "ordered_map")]
    pub labels: HashMap<String, Label>,
    pub instruction_count: u32,
    pub instruction_bytes: u32,
    pub data_bytes: u32,
    pub words: Vec<u32>,
}

impl MemoryRegion {
    pub fn new(
        kind: RegionKind,
        start: u32,
        len: u32,
        labels: &HashMap<String, Label>,
        instruction_count: u32,
        instruction_bytes: u32,
        data_bytes: u32,
        words: Vec<u32>,
    ) -> Self {
        Self {
            kind,
            start,
            len: 4 * ((len + 3) / 4),
            labels: labels.clone(),
            instruction_count,
            instruction_bytes,
            data_bytes,
            words,
        }
    }

    pub fn code_start(&self) -> u32 {
        if self.kind == RegionKind::DataBuffer {
            self.start + 4
        } else {
            self.start
        }
    }

    pub fn label_address(&self, label: &str) -> u32 {
        self.code_start() + self.labels.get(label).unwrap().offset
    }

    pub fn end(&self) -> u32 {
        self.start + self.len
    }

    pub fn labels_table_rows(regions: Vec<&MemoryRegion>) -> String {
        let mut entries = Vec::new();
        for region in regions {
            for (label, value) in &region.labels {
                if value.description.contains("[private]") {
                    continue;
                }
                entries.push((
                    label,
                    &value.description,
                    value.offset + region.code_start(),
                ));
            }
        }
        entries.sort_by(|x, y| x.0.cmp(y.0));

        let mut result = String::new();
        for entry in entries {
            result.push_str(&format!(
                "\\hyperlink{{{}}}{{{}{}}} & \
                 \\makecell{{{{\\tt {:X}}} (\\hexa{{C0000}}+{})}} \\\\\n",
                entry.0.replace('_', "-"),
                entry.0.replace('_', "\\_"),
                entry.1,
                entry.2,
                entry.2 - 0xC0000
            ));
        }
        result.pop();
        result.pop();
        result.pop();
        result
    }
}

pub struct Context {
    micro_controller: RefCell<MicroController>,
    graphics_card: Option<Rc<RefCell<GraphicsCard>>>,
    display: Option<Rc<RefCell<TextDisplay>>>,
    memory_map: HashMap<String, MemoryRegion>,
    memory_interval_bounds: BTreeSet<u32>,
    programs: HashMap<String, Assembler>,
    keyboard: Keyboard,
    error_codes: BTreeMap<u32, String>,
}

#[derive(Serialize, Deserialize)]
struct SerializableContext {
    micro_controller_state: Vec<u32>,
    #[serde(serialize_with = "ordered_map")]
    memory_map: HashMap<String, MemoryRegion>,
    memory_interval_bounds: BTreeSet<u32>,
    #[serde(serialize_with = "ordered_map")]
    programs: HashMap<String, Assembler>,
    error_codes: BTreeMap<u32, String>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            micro_controller: RefCell::new(MicroController::default()),
            graphics_card: Option::None,
            display: Option::None,
            memory_map: HashMap::new(),
            memory_interval_bounds: BTreeSet::new(),
            programs: HashMap::new(),
            keyboard: Keyboard::new(),
            error_codes: BTreeMap::new(),
        }
    }

    pub fn from_file(name: &str) -> std::io::Result<Self> {
        let mut input = File::open(name)?;
        let mut bytes = Vec::<u8>::new();
        input.read_to_end(&mut bytes)?;
        Ok(Self::deserialize(&bytes))
    }

    pub fn to_file(&self, name: &str) -> std::io::Result<()> {
        File::create(name)?.write_all(&self.serialize())
    }

    pub fn micro_controller(&self) -> &RefCell<MicroController> {
        &self.micro_controller
    }

    pub fn set_display(&mut self, display: Rc<RefCell<TextDisplay>>) {
        self.set_graphics_card_and_display(
            Rc::new(RefCell::new(GraphicsCard::new(display.clone()))),
            display,
        );
    }

    fn set_graphics_card_and_display(
        &mut self,
        gpu: Rc<RefCell<GraphicsCard>>,
        display: Rc<RefCell<TextDisplay>>,
    ) {
        self.micro_controller
            .borrow_mut()
            .set_spi_device(gpu.clone());
        self.micro_controller
            .borrow_mut()
            .set_pio_device(gpu.clone());
        self.graphics_card = Some(gpu);
        self.display = Some(display.clone());
    }

    pub fn get_display(&self) -> Rc<RefCell<TextDisplay>> {
        self.display.clone().unwrap()
    }

    pub fn memory_region(&self, name: &str) -> &MemoryRegion {
        self.memory_map.get(name).unwrap()
    }

    pub fn program(&self, name: &str) -> &Assembler {
        self.programs.get(name).unwrap()
    }

    pub fn add_memory_region(&mut self, name: &str, memory_region: MemoryRegion) {
        let begin = memory_region.start;
        let end = memory_region.start + memory_region.len;
        assert_eq!(self.memory_interval_bounds.range(begin..end).count(), 0);
        self.memory_interval_bounds.insert(begin);
        self.memory_interval_bounds.insert(end - 1);
        self.memory_map.insert(String::from(name), memory_region);
    }

    // Updates memory region to actual size of its data buffer.
    // Must be used for a data buffer memory region (ie starting with a 4 bytes 'len' header).
    pub fn update_memory_region(&mut self, name: &str) {
        let memory_region = self.memory_map.get_mut(name).unwrap();
        // Total size of the region is data size + header size.
        let len = self
            .micro_controller
            .borrow_mut()
            .debug_get32(memory_region.start)
            + 4;
        assert!(len <= memory_region.len);
        assert!(self
            .memory_interval_bounds
            .remove(&(memory_region.end() - 1)));
        memory_region.len = len;
        self.memory_interval_bounds.insert(memory_region.end() - 1);
    }

    pub fn check_memory_regions(&mut self) {
        for (name, region) in &self.memory_map {
            if region.kind != RegionKind::DataBuffer || region.start > 0x100000 {
                continue;
            }
            let actual_size = self.micro_controller.borrow_mut().debug_get32(region.start);
            if actual_size == 0xFFFFFFFF {
                // Region not yet filled with data.
                continue;
            }
            assert!(
                actual_size <= region.len - 4,
                "Region {} {} {} {actual_size}",
                name,
                region.start,
                region.len
            );
        }
    }

    pub fn add_program(&mut self, name: &str, program: Assembler) {
        self.programs.insert(String::from(name), program);
    }

    pub fn add_error_code(&mut self, code: u32, message: &str) {
        let result = self.error_codes.insert(code, String::from(message));
        assert!(result.is_none());
    }

    pub fn get_error_codes_table(&self) -> String {
        let mut result = String::new();
        result.push_str("\\begin{longtable}{|r|r|l|}\n");
        result.push_str("\\hline \\multicolumn{2}{|c|}{\\bf Code} & ");
        result.push_str("\\makecell{\\bf Meaning} \\\\\\hline \\endhead \n");
        result.push_str("\\hline \\endfoot \n");
        for (code, message) in &self.error_codes {
            result.push_str(&format!(
                "{} & {:02X}$_{{16}}$ & \\makecell{{{}}} \\\\\n",
                code, code, message
            ));
        }
        result.push_str("\\end{longtable}\\hspace{2mm}\n");
        result
    }

    const USART_REQUIRED_MASK: u32 = 0b00100000_10000001_11111111_11111111;
    const USART_REQUIRED_MODE: u32 = 0b00000000_00000000_00000011_11110000;

    pub fn send_scancode(&mut self, scancode: u8) {
        self.micro_controller.borrow_mut().set_usart_input(
            scancode as u32,
            Self::USART_REQUIRED_MASK,
            Self::USART_REQUIRED_MODE,
        );
    }

    fn at_get_char(insn: Instruction, r0: u32, r1: u32) -> bool {
        match insn {
            Instruction::StrRtRnImm5 {
                rt: _,
                rn: 0,
                imm: 0,
            } => r0 == crate::atmel::NVIC_ICER0,
            Instruction::StrRtRnImm5 {
                rt: _,
                rn: 1,
                imm: 0,
            } => r1 == crate::atmel::NVIC_ICER0,
            _ => false,
        }
    }

    fn in_get_char(insn: Instruction, r0: u32, r1: u32) -> bool {
        const KEYBOARD_HANDLER_CHAR: u32 = 0x400E1A98;
        match insn {
            Instruction::LdrRtRnImm5 {
                rt: _,
                rn: 0,
                imm: 0,
            } => r0 == KEYBOARD_HANDLER_CHAR,
            Instruction::LdrRtRnImm5 {
                rt: _,
                rn: 1,
                imm: 0,
            } => r1 == KEYBOARD_HANDLER_CHAR,
            _ => false,
        }
    }

    pub fn run_until_get_char(&mut self) {
        self.micro_controller
            .borrow_mut()
            .run_until(Self::in_get_char);
    }

    pub fn type_keys(&mut self, keys: Vec<&str>) {
        for key in keys {
            let scancodes = if let Some(suffix) = key.strip_prefix('~') {
                self.keyboard.key_released(suffix)
            } else {
                self.keyboard.key_pressed(key)
            };
            assert!(!scancodes.is_empty());
            for scancode in scancodes {
                self.micro_controller.borrow_mut().set_usart_input(
                    *scancode as u32,
                    Self::USART_REQUIRED_MASK,
                    Self::USART_REQUIRED_MODE,
                );
                self.micro_controller
                    .borrow_mut()
                    .run_until(Self::at_get_char);
            }
        }
        self.micro_controller
            .borrow_mut()
            .run_until(Self::at_get_char);
        self.micro_controller
            .borrow_mut()
            .run_until(Self::in_get_char);
    }

    pub fn type_ascii(&mut self, text: &str) {
        let mut key_strings = Vec::with_capacity(text.len());
        for c in text.chars() {
            if c == '\n' {
                key_strings.push(String::from("Enter"));
            } else if c == '\t' {
                key_strings.push(String::from("Tab"));
            } else {
                key_strings.push(format!("{c}"));
            }
        }
        let mut keys = Vec::with_capacity(key_strings.len());
        for key_string in &key_strings {
            keys.push(key_string.as_str());
        }
        self.type_keys(keys);
    }

    pub fn get_text(&mut self, address: u32) -> String {
        let mut micro_controller = self.micro_controller.borrow_mut();
        let mut result = Vec::new();
        let length = micro_controller.debug_get32(address);
        for i in 0..length {
            result.push((micro_controller.debug_get32(address + 4 + i) & 0xFF) as u8);
        }
        String::from_utf8(result).unwrap()
    }

    pub fn store_text(&mut self, address: u32, text: &str) {
        let mut micro_controller = self.micro_controller.borrow_mut();
        micro_controller.debug_set32(address, text.len() as u32);
        let bytes = text.as_bytes();
        let byte = |index: usize| {
            if index >= bytes.len() {
                0
            } else {
                bytes[index] as u32
            }
        };
        for i in 0..((text.len() + 3) / 4) {
            let p = 4 * i;
            let value = byte(p + 3) << 24 | byte(p + 2) << 16 | byte(p + 1) << 8 | byte(p);
            micro_controller.debug_set32(address + 4 * i as u32 + 4, value);
        }
    }

    pub fn enter_text_editor_text(&mut self, text: &str) {
        let process_sp = self.micro_controller.borrow().get_stack_pointer(false);
        // Access saved register R1 in SVC handler stacked frame. Points to the first argument
        // of the 'read' system call, in the stack frame of this function call
        let r1 = self
            .micro_controller
            .borrow_mut()
            .debug_get32(process_sp + 4);
        // Get address of 'gap' local variable in 'text_editor' function's stack frame.
        // Skips 4 words for the 'read' stack frame (3 parameters + return address), and 1 word for
        // the 'c' local variable of 'text_editor'.
        let gap = r1 + (4 + 1) * 4;
        // Get the address of other local variables in 'text_editor' stack frame.
        let end = gap + 4;
        let cursor = end + 4;
        let begin = cursor + 4;
        // Store the text and adjust the local variable values.
        let begin_value = self.micro_controller.borrow_mut().debug_get32(begin);
        let end_value = self.micro_controller.borrow_mut().debug_get32(end);
        let cursor_value = begin_value + text.len() as u32;
        self.micro_controller
            .borrow_mut()
            .debug_set32(cursor, cursor_value);
        self.micro_controller
            .borrow_mut()
            .debug_set32(gap, end_value - cursor_value);
        self.store_text(begin_value - 4, text);
    }

    pub fn get_file_content(&mut self, name: &str) -> String {
        let mut file_block = self
            .micro_controller
            .borrow_mut()
            .debug_get32(0x80000 + 256);
        while file_block > 256 {
            if self.get_file_name(file_block) == name {
                return self.get_file_text(file_block);
            }
            file_block = self
                .micro_controller
                .borrow_mut()
                .debug_get32(file_block + 4);
        }
        String::default()
    }

    fn get_file_name(&mut self, file_block: u32) -> String {
        let name_length = self
            .micro_controller
            .borrow_mut()
            .debug_get32(file_block + 8);
        let name_start = file_block + 12;
        let mut name = String::new();
        for i in 0..name_length {
            name.push(
                (self
                    .micro_controller
                    .borrow_mut()
                    .debug_get32(name_start + i) as u8) as char,
            );
        }
        name
    }

    fn get_file_text(&mut self, block: u32) -> String {
        String::from_utf8(self.get_file_bytes(block)).unwrap()
    }

    fn get_file_bytes(&mut self, mut block: u32) -> Vec<u8> {
        let name_length = self.micro_controller.borrow_mut().debug_get32(block + 8);
        let mut offset = 12 + name_length;
        let mut bytes = Vec::new();
        loop {
            let size = self.micro_controller.borrow_mut().debug_get32(block);
            if size <= 256 {
                for i in block + offset..block + size {
                    bytes.push(self.micro_controller.borrow_mut().debug_get32(i) as u8);
                }
            } else {
                for i in block + offset..block + 256 {
                    bytes.push(self.micro_controller.borrow_mut().debug_get32(i) as u8);
                }
            }
            if size <= 256 {
                break;
            }
            block = self.micro_controller.borrow_mut().debug_get32(block);
            offset = 4;
            assert!(block >= 0x80000 + 512, "{}", block);
            assert!(block < 0xC0000, "{}", block);
        }
        bytes
    }

    pub fn get_file_stats(&mut self) -> BTreeMap<(u8, String), (usize, u32)> {
        let mut result = BTreeMap::new();
        let mut file_block = self
            .micro_controller
            .borrow_mut()
            .debug_get32(0x80000 + 256);
        while file_block > 256 {
            let name = self.get_file_name(file_block);
            let level = name.chars().filter(|c| *c == '/').count() as u8;
            let content = self.get_file_bytes(file_block);
            if name.ends_with(".toy") {
                result.insert(
                    (level, name),
                    (content.len(), Self::get_line_count(&content)),
                );
            } else {
                result.insert((level, name), (content.len(), 0));
            }
            file_block = self
                .micro_controller
                .borrow_mut()
                .debug_get32(file_block + 4);
        }
        result
    }

    fn get_line_count(bytes: &[u8]) -> u32 {
        const NEW_LINE: u8 = 10;
        let mut last_char = NEW_LINE;
        let mut count = 0;
        for b in bytes {
            if last_char == NEW_LINE {
                count += 1;
            }
            last_char = *b;
        }
        count
    }

    pub fn dump_files(&mut self, directory: &str) -> std::io::Result<()> {
        let mut file_block = self
            .micro_controller
            .borrow_mut()
            .debug_get32(0x80000 + 256);
        while file_block > 256 {
            let name = self.get_file_name(file_block);
            if name.ends_with(".toy") || name.ends_with("BUILD") {
                let content = self.get_file_bytes(file_block);
                let path = Path::new(&directory).join(name);
                std::fs::create_dir_all(path.parent().unwrap())?;
                File::create(path)?.write_all(&content)?;
            }
            file_block = self
                .micro_controller
                .borrow_mut()
                .debug_get32(file_block + 4);
        }
        Ok(())
    }

    pub fn write_backup(&mut self, directory: &str, name: &str) -> std::io::Result<()> {
        let mut micro_controller = self.micro_controller.borrow_mut();
        let mut words = Vec::new();
        for address in (0x80000..0x100000).step_by(4) {
            let word = micro_controller.debug_get32(address);
            if word != 0xFFFFFFFF {
                words.push(format!("W{:X},{:08X}#", address, word));
            }
        }
        words.push(String::from("flash#"));
        if micro_controller.run_from_flash() {
            words.push(String::from("reset#"));
        }
        write_lines(directory, name, &words)
    }

    pub fn check_equal_buffer(&mut self, other: &mut Context, buffer: u32) {
        let mut micro_controller = self.micro_controller().borrow_mut();
        let mut other_micro_controller = other.micro_controller().borrow_mut();
        let size = micro_controller.debug_get32(buffer) / 4 + 1;
        for i in (buffer..buffer + size).step_by(4) {
            let actual = other_micro_controller.debug_get32(i);
            let expected = micro_controller.debug_get32(i);
            assert_eq!(actual, expected);
        }
    }

    fn serialize(&self) -> Vec<u8> {
        let serializable = SerializableContext {
            micro_controller_state: self.micro_controller.borrow().serialize(),
            memory_map: self.memory_map.clone(),
            memory_interval_bounds: self.memory_interval_bounds.clone(),
            programs: self.programs.clone(),
            error_codes: self.error_codes.clone(),
        };
        to_allocvec(&serializable).unwrap()
    }

    fn deserialize(bytes: &[u8]) -> Self {
        let mut serializable: SerializableContext = from_bytes(bytes).unwrap();
        let micro_controller = RefCell::new(MicroController::deserialize(
            &mut serializable.micro_controller_state,
        ));
        Context {
            micro_controller,
            graphics_card: Option::None,
            display: Option::None,
            memory_map: serializable.memory_map,
            memory_interval_bounds: serializable.memory_interval_bounds,
            programs: serializable.programs,
            keyboard: Keyboard::new(),
            error_codes: serializable.error_codes.clone(),
        }
    }
}

impl Clone for Context {
    fn clone(&self) -> Self {
        let mut clone = Self {
            micro_controller: self.micro_controller.clone(),
            graphics_card: Option::None,
            display: Option::None,
            memory_map: self.memory_map.clone(),
            memory_interval_bounds: self.memory_interval_bounds.clone(),
            programs: self.programs.clone(),
            keyboard: self.keyboard.clone(),
            error_codes: self.error_codes.clone(),
        };
        if let Some(gpu) = &self.graphics_card {
            if let Some(display) = &self.display {
                let mut gpu_clone = gpu.borrow().clone();
                let display_clone = Rc::new(RefCell::new(display.borrow().clone()));
                gpu_clone.set_display(display_clone.clone());
                clone
                    .set_graphics_card_and_display(Rc::new(RefCell::new(gpu_clone)), display_clone);
            }
        }
        clone
    }
}
