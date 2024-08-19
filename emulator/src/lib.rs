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

mod arm;
mod backup;
mod boot;
mod bus;
mod chip;
mod flash;
mod gpu;
mod interrupt;
mod keyboard;
mod memory;
mod mpu;
mod pio;
mod power;
mod reset;
mod spi;
mod system;
mod time;
mod usart;
mod watchdog;

pub use arm::Instruction;
pub use chip::MicroController;
pub use gpu::Color;
pub use gpu::Display;
pub use gpu::GraphicsCard;
pub use gpu::Point;
pub use gpu::TextDisplay;
pub use keyboard::Keyboard;
pub use pio::Controller;
pub use pio::EmptyPioDevice;
pub use pio::PioDevice;
pub use spi::EmptySpiDevice;
pub use spi::SpiDevice;
