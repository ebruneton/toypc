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

use std::{cell::RefCell, ptr::null, rc::Rc};

use emulator::{Color, Display, GraphicsCard, Instruction, MicroController, PioDevice, Point};
use scripts::{BootHelper, FlashHelper};

mod panic {
    mod js {
        #[link(wasm_import_module = "panic")]
        extern "C" {
            pub fn log(chars: *const u8, length: usize);
        }
    }

    pub fn log(s: &str) {
        unsafe {
            js::log(s.as_ptr(), s.len());
        }
    }
}

mod timer {
    mod js {
        #[link(wasm_import_module = "timer")]
        extern "C" {
            pub fn wait(micros: u32);
        }
    }

    pub fn wait(micros: u32) {
        unsafe {
            js::wait(micros);
        }
    }
}

mod serial {
    mod js {
        #[link(wasm_import_module = "serial")]
        extern "C" {
            pub fn send(chars: *const u8, length: usize) -> bool;
            pub fn receive() -> u32;
        }
    }

    pub fn send(s: &str) -> bool {
        unsafe { js::send(s.as_ptr(), s.len()) }
    }

    pub fn receive() -> String {
        let mut result = String::new();
        loop {
            let c = unsafe { js::receive() };
            if c < 256 {
                result.push((c as u8) as char);
            } else {
                return result;
            }
        }
    }
}

mod display {
    #[link(wasm_import_module = "display")]
    extern "C" {
        pub fn set_on(on: bool);
        pub fn set_read_layer(layer: u32);
        pub fn set_write_layer(layer: u32);
        pub fn draw_char(x: u32, y: u32, c: u8, foreground: u32, background: u32);
        pub fn set_cursor(x: u32, y: u32, enabled: bool, blink_time: i32);
        pub fn clear(left: u32, top: u32, right: u32, bottom: u32, full_screen: bool);
        pub fn reset();
        pub fn set_led(on: bool);
    }
}

#[derive(Default)]
struct RemoteDisplay {}

impl RemoteDisplay {
    fn to_u32(color: Color) -> u32 {
        ((color.r as u32) << 16) | ((color.g as u32) << 8) | (color.b as u32)
    }
}

impl Display for RemoteDisplay {
    fn set_on(&mut self, on: bool) {
        unsafe {
            display::set_on(on);
        }
    }

    fn set_read_layer(&mut self, layer: u32) {
        unsafe {
            display::set_read_layer(layer);
        }
    }

    fn set_write_layer(&mut self, layer: u32) {
        unsafe {
            display::set_write_layer(layer);
        }
    }

    fn draw_char(&mut self, x: u32, y: u32, c: u8, foreground: Color, background: Color) {
        unsafe {
            display::draw_char(x, y, c, Self::to_u32(foreground), Self::to_u32(background));
        }
    }

    fn set_cursor(&mut self, x: u32, y: u32, enabled: bool, blink_time: Option<u8>) {
        unsafe {
            display::set_cursor(x, y, enabled, blink_time.map_or(-1, |v| v as i32));
        }
    }

    fn clear(&mut self, top_left: &Point, bottom_right: &Point, full_screen: bool) {
        unsafe {
            display::clear(
                top_left.x,
                top_left.y,
                bottom_right.x,
                bottom_right.y,
                full_screen,
            );
        }
    }

    fn reset(&mut self) {
        unsafe {
            display::reset();
        }
    }
}

struct PioDevices {
    gpu: Rc<RefCell<GraphicsCard>>,
}

impl PioDevice for PioDevices {
    fn pio_state_changed(&mut self, pins: &[u32; 4]) {
        unsafe {
            display::set_led((pins[1] & (1 << 27)) != 0);
        }
        self.gpu.borrow_mut().pio_state_changed(pins);
    }
}

pub struct Arduino {
    micro_controller: RefCell<MicroController>,
    last_in_get_char: bool,
    input_buffer: String,
}

impl Arduino {
    fn new(seed: u32) -> Self {
        let mut micro_controller = MicroController::new(seed, Some(timer::wait));
        micro_controller.set_max_boot_program_go_cycles(100_000_000);
        let display = Rc::new(RefCell::new(RemoteDisplay::default()));
        let gpu = Rc::new(RefCell::new(GraphicsCard::new(display)));
        micro_controller.set_spi_device(gpu.clone());
        micro_controller.set_pio_device(Rc::new(RefCell::new(PioDevices { gpu })));
        Arduino {
            micro_controller: RefCell::new(micro_controller),
            last_in_get_char: false,
            input_buffer: String::new(),
        }
    }

    fn get_flash_content(&self) -> *const u32 {
        self.micro_controller
            .borrow_mut()
            .debug_get_flash_content()
            .as_ptr()
    }

    fn run_from_flash(&self) -> bool {
        self.micro_controller.borrow().run_from_flash()
    }

    fn run(&mut self) -> i32 {
        const SYSTEM_TIMER_CONTROL_AND_STATUS_REGISTER: u32 = 0xE000E010;
        const KEYBOARD_HANDLER_CHAR: u32 = 0x400E1A98;
        if !self.micro_controller.borrow_mut().run_from_flash() {
            return -1;
        }
        let mut counter = 0;
        let mut in_delay = false;
        let mut in_get_char = false;
        self.micro_controller
            .borrow_mut()
            .run_until_reset_or(|insn, r0, r1| {
                counter += 1;
                counter >= 1000000
                    || match insn {
                        Instruction::LdrRtRnImm5 {
                            rt: _,
                            rn: 0,
                            imm: 0,
                        } => {
                            in_delay = r0 == SYSTEM_TIMER_CONTROL_AND_STATUS_REGISTER;
                            in_get_char = r0 == KEYBOARD_HANDLER_CHAR;
                            in_delay || in_get_char
                        }
                        Instruction::LdrRtRnImm5 {
                            rt: _,
                            rn: 1,
                            imm: 0,
                        } => {
                            in_delay = r1 == SYSTEM_TIMER_CONTROL_AND_STATUS_REGISTER;
                            in_get_char = r1 == KEYBOARD_HANDLER_CHAR;
                            in_delay || in_get_char
                        }
                        _ => false,
                    }
            });
        let previous_in_get_char = self.last_in_get_char;
        self.last_in_get_char = in_get_char;
        if in_delay {
            -1
        } else if in_get_char && previous_in_get_char {
            10
        } else {
            0
        }
    }

    fn process_scancode(&mut self, scancode: u32) {
        const USART0_RECEIVE_HOLDING_REGISTER: u32 = 0x40098018;
        const USART_REQUIRED_MASK: u32 = 0b00100000_10000001_11111111_11111111;
        const USART_REQUIRED_MODE: u32 = 0b00000000_00000000_00000011_11110000;
        if !self.micro_controller.borrow_mut().run_from_flash() {
            return;
        }
        self.micro_controller.borrow_mut().set_usart_input(
            scancode,
            USART_REQUIRED_MASK,
            USART_REQUIRED_MODE,
        );
        let mut counter = 0;
        self.micro_controller
            .borrow_mut()
            .run_until_reset_or(|insn, r0, r1| {
                counter += 1;
                counter >= 1000
                    || match insn {
                        Instruction::LdrRtRnImm5 {
                            rt: _,
                            rn: 0,
                            imm: 0,
                        } => r0 == USART0_RECEIVE_HOLDING_REGISTER,
                        Instruction::LdrRtRnImm5 {
                            rt: _,
                            rn: 1,
                            imm: 0,
                        } => r1 == USART0_RECEIVE_HOLDING_REGISTER,
                        _ => false,
                    }
            });
    }

    fn reset(&mut self) {
        self.micro_controller.borrow_mut().reset();
        self.last_in_get_char = false;
        self.input_buffer.clear();
    }

    fn new_boot_helper(&mut self) -> *const BootHelper {
        self.input_buffer.clear();
        let mut boot_helper = Box::new(BootHelper::create(&self.micro_controller, true));
        let ok = boot_helper.write("T#");
        serial::send(&boot_helper.read());
        if ok {
            Box::into_raw(boot_helper)
        } else {
            null()
        }
    }

    fn run_boot_helper(&mut self, boot_helper: &mut BootHelper) -> bool {
        self.input_buffer.push_str(&serial::receive());
        if let Some(index) = self.input_buffer.rfind('#') {
            let exit = !boot_helper.write(&self.input_buffer[..index + 1]);
            self.input_buffer = String::from(&self.input_buffer[index + 1..]);
            serial::send(&boot_helper.read());
            if exit {
                return true;
            }
        }
        false
    }

    fn new_flash_helper(&mut self) -> *const FlashHelper {
        self.input_buffer.clear();
        let filename = serial::receive();
        let mut flash_helper = Box::new(if filename.is_empty() {
            FlashHelper::create(&self.micro_controller, true)
        } else {
            FlashHelper::create_from_file(&self.micro_controller, &filename, true)
        });
        let ok = flash_helper.write("T#");
        serial::send(&flash_helper.read());
        if ok {
            Box::into_raw(flash_helper)
        } else {
            null()
        }
    }

    fn run_flash_helper(&mut self, flash_helper: &mut FlashHelper) -> bool {
        self.input_buffer.push_str(&serial::receive());
        if let Some(index) = self.input_buffer.rfind('#') {
            let exit = !flash_helper.write(&self.input_buffer[..index + 1]);
            self.input_buffer = String::from(&self.input_buffer[index + 1..]);
            serial::send(&flash_helper.read());
            if exit {
                return true;
            }
        }
        false
    }
}

fn panic_hook(panic: &core::panic::PanicInfo<'_>) {
    panic::log(&panic.to_string());
}

#[no_mangle]
pub extern "C" fn initialize() {
    std::panic::set_hook(Box::new(panic_hook));
}

#[no_mangle]
extern "C" fn new_arduino(seed: u32) -> *const Arduino {
    Box::into_raw(Box::new(Arduino::new(seed)))
}

#[no_mangle]
extern "C" fn get_flash_content(arduino: *const Arduino) -> *const u32 {
    unsafe { &(*arduino) }.get_flash_content()
}

#[no_mangle]
extern "C" fn run_from_flash(arduino: *const Arduino) -> bool {
    unsafe { &(*arduino) }.run_from_flash()
}

#[no_mangle]
extern "C" fn run(arduino: *mut Arduino) -> i32 {
    unsafe { &mut (*arduino) }.run()
}

#[no_mangle]
extern "C" fn process_scancode(arduino: *mut Arduino, scancode: u32) {
    unsafe { &mut (*arduino) }.process_scancode(scancode);
}

#[no_mangle]
extern "C" fn reset(arduino: *mut Arduino) {
    unsafe { &mut (*arduino) }.reset();
}

#[no_mangle]
extern "C" fn new_boot_helper(arduino: *mut Arduino) -> *const BootHelper<'static> {
    unsafe { &mut (*arduino) }.new_boot_helper()
}

#[no_mangle]
extern "C" fn run_boot_helper(
    arduino: *mut Arduino,
    boot_helper: *mut BootHelper<'static>,
) -> bool {
    unsafe { &mut (*arduino) }.run_boot_helper(unsafe { &mut (*boot_helper) })
}

#[no_mangle]
extern "C" fn delete_boot_helper(raw_boot_helper: *mut BootHelper<'static>) {
    unsafe { drop(Box::from_raw(raw_boot_helper)) };
}

#[no_mangle]
extern "C" fn new_flash_helper(arduino: *mut Arduino) -> *const FlashHelper<'static> {
    unsafe { &mut (*arduino) }.new_flash_helper()
}

#[no_mangle]
extern "C" fn run_flash_helper(
    arduino: *mut Arduino,
    flash_helper: *mut FlashHelper<'static>,
) -> bool {
    unsafe { &mut (*arduino) }.run_flash_helper(unsafe { &mut (*flash_helper) })
}

#[no_mangle]
extern "C" fn delete_flash_helper(raw_flash_helper: *mut FlashHelper<'static>) {
    unsafe { drop(Box::from_raw(raw_flash_helper)) };
}
