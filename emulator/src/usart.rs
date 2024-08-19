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

pub const UNIVERSAL_RECEIVER_TRANSMITTER_BEGIN: u32 = 0x40098000;
pub const UNIVERSAL_RECEIVER_TRANSMITTER_END: u32 = 0x4009812C;
pub const UNIVERSAL_RECEIVER_TRANSMITTER_LAST: u32 = UNIVERSAL_RECEIVER_TRANSMITTER_END - 1;

pub const CONTROL_REGISTER: u32 = 0x40098000;
pub const MODE_REGISTER: u32 = 0x40098004;
pub const INTERRUPT_ENABLE_REGISTER: u32 = 0x40098008;
pub const INTERRUPT_DISABLE_REGISTER: u32 = 0x4009800C;
pub const INTERRUPT_MASK_REGISTER: u32 = 0x40098010;
pub const CHANNEL_STATUS_REGISTER: u32 = 0x40098014;
pub const RECEIVE_HOLDING_REGISTER: u32 = 0x40098018;
pub const TRANSMIT_HOLDING_REGISTER: u32 = 0x4009801C;

// Control Register flags.
const RESET_RECEIVER: u32 = 1 << 2;
const RESET_TRANSMITTER: u32 = 1 << 3;
const RECEIVER_ENABLE: u32 = 1 << 4;
const RECEIVER_DISABLE: u32 = 1 << 5;
const TRANSMITTER_ENABLE: u32 = 1 << 6;
const TRANSMITTER_DISABLE: u32 = 1 << 7;
const RESET_STATUS: u32 = 1 << 8;
const SUPPORTED_CONTROL_BITS: u32 = RESET_RECEIVER
    | RESET_TRANSMITTER
    | RECEIVER_ENABLE
    | RECEIVER_DISABLE
    | TRANSMITTER_ENABLE
    | TRANSMITTER_DISABLE
    | RESET_STATUS;
const CONTROL_BITS: u32 = 0x3CFFFC;

// Mode Register used bits.
const MODE_BITS: u32 = !(1 << 27);

// Interrupt Mask Register flags.
const RECEIVER_READY_INTERRUPT: u32 = 1;

// Channel Status Register flags.
const RECEIVER_READY: u32 = 1 << 0;
const TRANSMITTER_READY: u32 = 1 << 1;
const OVERRUN_ERROR: u32 = 1 << 5;
const RESET_BITS: u32 = 0b00111111000000001110010011100100;

/// The Universal Synchronous Asynchronous Receiver Transmitter (USART0). See section 35, p769 of
/// the Atmel SAM3X Datasheet. This implementation only supports the receiver part.
#[derive(Clone)]
pub struct UniversalReceiverTransmitter {
    mode: u32,
    interrupt_mask: u32,
    channel_status: u32,
    receiver_enabled: bool,
    receive_holding: u32,
    transmitter_enabled: bool,
    transmit_holding: u32,
}

impl UniversalReceiverTransmitter {
    const PERIPHERAL_ID: u32 = 17;

    const CONTROL_REGISTER_INDEX: u32 =
        (CONTROL_REGISTER - UNIVERSAL_RECEIVER_TRANSMITTER_BEGIN) / 4;
    const MODE_REGISTER_INDEX: u32 = (MODE_REGISTER - UNIVERSAL_RECEIVER_TRANSMITTER_BEGIN) / 4;
    const INTERRUPT_ENABLE_REGISTER_INDEX: u32 =
        (INTERRUPT_ENABLE_REGISTER - UNIVERSAL_RECEIVER_TRANSMITTER_BEGIN) / 4;
    const INTERRUPT_DISABLE_REGISTER_INDEX: u32 =
        (INTERRUPT_DISABLE_REGISTER - UNIVERSAL_RECEIVER_TRANSMITTER_BEGIN) / 4;
    const INTERRUPT_MASK_REGISTER_INDEX: u32 =
        (INTERRUPT_MASK_REGISTER - UNIVERSAL_RECEIVER_TRANSMITTER_BEGIN) / 4;
    const CHANNEL_STATUS_REGISTER_INDEX: u32 =
        (CHANNEL_STATUS_REGISTER - UNIVERSAL_RECEIVER_TRANSMITTER_BEGIN) / 4;
    const RECEIVE_HOLDING_REGISTER_INDEX: u32 =
        (RECEIVE_HOLDING_REGISTER - UNIVERSAL_RECEIVER_TRANSMITTER_BEGIN) / 4;
    const TRANSMIT_HOLDING_REGISTER_INDEX: u32 =
        (TRANSMIT_HOLDING_REGISTER - UNIVERSAL_RECEIVER_TRANSMITTER_BEGIN) / 4;

    pub fn new() -> Self {
        Self {
            mode: 0,
            interrupt_mask: 0,
            channel_status: 0,
            receiver_enabled: false,
            receive_holding: 0,
            transmitter_enabled: false,
            transmit_holding: 0,
        }
    }

    pub fn get32_aligned(&mut self, address: u32) -> u32 {
        debug_assert!(address % 4 == 0);
        match (address - UNIVERSAL_RECEIVER_TRANSMITTER_BEGIN) >> 2 {
            Self::CONTROL_REGISTER_INDEX => 0,
            Self::MODE_REGISTER_INDEX => self.mode,
            Self::INTERRUPT_ENABLE_REGISTER_INDEX => 0,
            Self::INTERRUPT_DISABLE_REGISTER_INDEX => 0,
            Self::INTERRUPT_MASK_REGISTER_INDEX => self.interrupt_mask,
            Self::CHANNEL_STATUS_REGISTER_INDEX => self.channel_status,
            Self::RECEIVE_HOLDING_REGISTER_INDEX => {
                let result = self.receive_holding;
                self.channel_status &= !RECEIVER_READY;
                self.receive_holding = 0;
                result
            }
            Self::TRANSMIT_HOLDING_REGISTER_INDEX => 0,
            _ => panic!("Unsupported USART register {address:#010X}"),
        }
    }

    pub fn set32_aligned(&mut self, address: u32, value: u32) {
        debug_assert!(address % 4 == 0);
        match (address - UNIVERSAL_RECEIVER_TRANSMITTER_BEGIN) >> 2 {
            Self::CONTROL_REGISTER_INDEX => {
                if (value & SUPPORTED_CONTROL_BITS) != (value & CONTROL_BITS) {
                    panic!("Unsupported USART Control Register value {value:#010X}");
                }
                if value & RESET_RECEIVER != 0 {
                    self.receive_holding = 0;
                    self.channel_status &= !RECEIVER_READY;
                }
                if value & RESET_TRANSMITTER != 0 {
                    self.transmit_holding = 0;
                    self.channel_status &= !TRANSMITTER_READY;
                }
                if value & RECEIVER_DISABLE != 0 {
                    self.receiver_enabled = false;
                } else if value & RECEIVER_ENABLE != 0 {
                    self.receiver_enabled = true;
                }
                if value & TRANSMITTER_DISABLE != 0 {
                    self.transmitter_enabled = false;
                } else if value & TRANSMITTER_ENABLE != 0 {
                    self.transmitter_enabled = true;
                }
                if value & RESET_STATUS != 0 {
                    self.channel_status &= !RESET_BITS;
                }
            }
            Self::MODE_REGISTER_INDEX => self.mode = value & MODE_BITS,
            Self::INTERRUPT_ENABLE_REGISTER_INDEX => {
                if value & !RECEIVER_READY_INTERRUPT != 0 {
                    panic!("Unsupported USART Interrupt Enable Register value {value:#010X}");
                }
                self.interrupt_mask |= value;
            }
            Self::INTERRUPT_DISABLE_REGISTER_INDEX => self.interrupt_mask &= !value,
            Self::INTERRUPT_MASK_REGISTER_INDEX => (),
            Self::CHANNEL_STATUS_REGISTER_INDEX => (),
            Self::RECEIVE_HOLDING_REGISTER_INDEX => (),
            Self::TRANSMIT_HOLDING_REGISTER_INDEX => self.transmit_holding = value,
            _ => panic!("Unsupported USART register {address:#010X}"),
        }
    }

    pub fn data_received(&mut self, character: u32, required_mode_mask: u32, required_mode: u32) {
        if !self.receiver_enabled || self.mode & required_mode_mask != required_mode {
            return;
        }
        self.receive_holding = character & 0xFF;
        if self.channel_status & RECEIVER_READY != 0 {
            self.channel_status |= OVERRUN_ERROR;
        } else {
            self.channel_status |= RECEIVER_READY;
        }
    }

    #[inline]
    pub fn level_interrupts(&self) -> u32 {
        ((self.channel_status & RECEIVER_READY != 0) as u32) << Self::PERIPHERAL_ID
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get32_aligned() {
        let mut usart = UniversalReceiverTransmitter::new();
        usart.mode = 123;
        usart.interrupt_mask = 456;
        usart.channel_status = 789;
        usart.receive_holding = 0xAB;
        usart.transmit_holding = 0xCD;

        assert_eq!(usart.get32_aligned(CONTROL_REGISTER), 0);
        assert_eq!(usart.get32_aligned(MODE_REGISTER), 123);
        assert_eq!(usart.get32_aligned(INTERRUPT_ENABLE_REGISTER), 0);
        assert_eq!(usart.get32_aligned(INTERRUPT_DISABLE_REGISTER), 0);
        assert_eq!(usart.get32_aligned(INTERRUPT_MASK_REGISTER), 456);
        assert_eq!(usart.get32_aligned(CHANNEL_STATUS_REGISTER), 789);
        assert_eq!(usart.get32_aligned(RECEIVE_HOLDING_REGISTER), 0xAB);
        assert_eq!(usart.get32_aligned(TRANSMIT_HOLDING_REGISTER), 0);
        // Side effects of the previous read of the RECEIVE_HOLDING_REGISTER.
        assert_eq!(usart.get32_aligned(CHANNEL_STATUS_REGISTER), 788);
        assert_eq!(usart.get32_aligned(RECEIVE_HOLDING_REGISTER), 0);
    }

    #[test]
    #[should_panic(expected = "Unsupported USART register 0x400980FC")]
    fn get32_aligned_unsupported_register() {
        UniversalReceiverTransmitter::new().get32_aligned(0x400980FC);
    }

    #[test]
    fn set32_aligned_control_reset_receiver() {
        let mut usart = UniversalReceiverTransmitter::new();
        usart.receive_holding = 123;
        usart.channel_status = OVERRUN_ERROR | RECEIVER_READY;

        usart.set32_aligned(CONTROL_REGISTER, RESET_RECEIVER);

        assert_eq!(usart.receive_holding, 0);
        assert_eq!(usart.channel_status, OVERRUN_ERROR);
    }

    #[test]
    fn set32_aligned_control_reset_transmitter() {
        let mut usart = UniversalReceiverTransmitter::new();
        usart.transmit_holding = 123;
        usart.channel_status = OVERRUN_ERROR | TRANSMITTER_READY;

        usart.set32_aligned(CONTROL_REGISTER, RESET_TRANSMITTER);

        assert_eq!(usart.transmit_holding, 0);
        assert_eq!(usart.channel_status, OVERRUN_ERROR);
    }

    #[test]
    fn set32_aligned_control_receiver_enable() {
        let mut usart = UniversalReceiverTransmitter::new();

        usart.set32_aligned(CONTROL_REGISTER, RECEIVER_ENABLE);

        assert!(usart.receiver_enabled);
    }

    #[test]
    fn set32_aligned_control_receiver_disable() {
        let mut usart = UniversalReceiverTransmitter::new();
        usart.receiver_enabled = true;

        usart.set32_aligned(CONTROL_REGISTER, RECEIVER_ENABLE | RECEIVER_DISABLE);

        assert!(!usart.receiver_enabled);
    }

    #[test]
    fn set32_aligned_control_transmitter_enable() {
        let mut usart = UniversalReceiverTransmitter::new();

        usart.set32_aligned(CONTROL_REGISTER, TRANSMITTER_ENABLE);

        assert!(usart.transmitter_enabled);
    }

    #[test]
    fn set32_aligned_control_transmitter_disable() {
        let mut usart = UniversalReceiverTransmitter::new();
        usart.transmitter_enabled = true;

        usart.set32_aligned(CONTROL_REGISTER, TRANSMITTER_ENABLE | TRANSMITTER_DISABLE);

        assert!(!usart.transmitter_enabled);
    }

    #[test]
    fn set32_aligned_control_reset_status() {
        let mut usart = UniversalReceiverTransmitter::new();
        usart.channel_status = 0xFF00FFFE;

        usart.set32_aligned(CONTROL_REGISTER, RESET_STATUS);

        assert_eq!(usart.channel_status, 0xFF00FFFE & !RESET_BITS);
    }

    #[test]
    #[should_panic(expected = "Unsupported USART Control Register value 0x00008000")]
    fn set32_aligned_control_unsupported_bit() {
        UniversalReceiverTransmitter::new().set32_aligned(CONTROL_REGISTER, 0x8000);
    }

    #[test]
    fn set32_aligned_mode() {
        let mut usart = UniversalReceiverTransmitter::new();

        usart.set32_aligned(MODE_REGISTER, 0x12345678);

        assert_eq!(usart.mode, 0x12345678 & MODE_BITS);
    }

    #[test]
    fn set32_aligned_interrupt_enable() {
        let mut usart = UniversalReceiverTransmitter::new();

        usart.set32_aligned(INTERRUPT_ENABLE_REGISTER, RECEIVER_READY_INTERRUPT);

        assert_eq!(usart.interrupt_mask, RECEIVER_READY_INTERRUPT);
    }

    #[test]
    #[should_panic(expected = "Unsupported USART Interrupt Enable Register value 0x00000002")]
    fn set32_aligned_interrupt_enable_unsupported() {
        UniversalReceiverTransmitter::new().set32_aligned(INTERRUPT_ENABLE_REGISTER, 0x02);
    }

    #[test]
    fn set32_aligned_interrupt_disable() {
        let mut usart = UniversalReceiverTransmitter::new();
        usart.interrupt_mask = 0x87654321;

        usart.set32_aligned(INTERRUPT_DISABLE_REGISTER, RECEIVER_READY_INTERRUPT);

        assert_eq!(usart.interrupt_mask, 0x87654320);
    }

    #[test]
    fn set32_aligned_interrupt_mask() {
        let mut usart = UniversalReceiverTransmitter::new();
        usart.interrupt_mask = 123;

        usart.set32_aligned(INTERRUPT_MASK_REGISTER, 456);

        assert_eq!(usart.interrupt_mask, 123);
    }

    #[test]
    fn set32_aligned_channel_status() {
        let mut usart = UniversalReceiverTransmitter::new();
        usart.channel_status = 123;

        usart.set32_aligned(CHANNEL_STATUS_REGISTER, 456);

        assert_eq!(usart.channel_status, 123);
    }

    #[test]
    fn set32_aligned_receive_holding() {
        let mut usart = UniversalReceiverTransmitter::new();
        usart.receive_holding = 123;

        usart.set32_aligned(RECEIVE_HOLDING_REGISTER, 456);

        assert_eq!(usart.receive_holding, 123);
    }

    #[test]
    fn set32_aligned_transmit_holding() {
        let mut usart = UniversalReceiverTransmitter::new();
        usart.transmit_holding = 123;

        usart.set32_aligned(TRANSMIT_HOLDING_REGISTER, 456);

        assert_eq!(usart.transmit_holding, 456);
    }

    #[test]
    #[should_panic(expected = "Unsupported USART register 0x400980FC")]
    fn set32_aligned_unsupported_register() {
        UniversalReceiverTransmitter::new().set32_aligned(0x400980FC, 0);
    }

    #[test]
    fn data_received_not_enabled() {
        let mut usart = UniversalReceiverTransmitter::new();

        usart.data_received(0xABCD, 0, 0);

        assert_eq!(usart.receive_holding, 0);
        assert_eq!(usart.level_interrupts(), 0);
    }

    #[test]
    fn data_received_unexpected_mode() {
        let mut usart = UniversalReceiverTransmitter::new();
        usart.mode = 0x12FF;
        usart.receiver_enabled = true;

        usart.data_received(0xABCD, 0xFF, 0xFE);

        assert_eq!(usart.receive_holding, 0);
        assert_eq!(usart.level_interrupts(), 0);
    }

    #[test]
    fn data_received_ok() {
        let mut usart = UniversalReceiverTransmitter::new();
        usart.mode = 0x12FE;
        usart.receiver_enabled = true;
        usart.receive_holding = 0x34;

        usart.data_received(0xABCD, 0xFF, 0xFE);

        assert_eq!(usart.receive_holding, 0xCD);
        assert_eq!(usart.channel_status & OVERRUN_ERROR, 0);
        assert_eq!(usart.level_interrupts(), 1 << 17);
    }

    #[test]
    fn data_received_overrun() {
        let mut usart = UniversalReceiverTransmitter::new();
        usart.mode = 0x12FE;
        usart.channel_status = RECEIVER_READY;
        usart.receiver_enabled = true;

        usart.data_received(0xABCD, 0xFF, 0xFE);

        assert_eq!(usart.receive_holding, 0xCD);
        assert_eq!(usart.channel_status & OVERRUN_ERROR, OVERRUN_ERROR);
        assert_eq!(usart.level_interrupts(), 1 << 17);
    }

    #[test]
    fn reset() {
        let mut usart = UniversalReceiverTransmitter::new();
        usart.mode = 123;
        usart.interrupt_mask = 456;
        usart.channel_status = 789;
        usart.receiver_enabled = true;
        usart.receive_holding = 0xAB;
        usart.transmitter_enabled = true;
        usart.transmit_holding = 0xCD;

        usart.reset();

        assert_eq!(usart.mode, 0);
        assert_eq!(usart.interrupt_mask, 0);
        assert_eq!(usart.channel_status, 0);
        assert!(!usart.receiver_enabled);
        assert_eq!(usart.receive_holding, 0);
        assert!(!usart.transmitter_enabled);
        assert_eq!(usart.transmit_holding, 0);
    }
}
