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

pub struct Field {
    pub name: &'static str,
    pub offset: u8,
    pub len: u8,
}

pub struct RegisterType<const F: usize> {
    pub address: u32,
    pub opcode: u32,
    pub fields: [Field; F],
}

impl<const F: usize> RegisterType<F> {
    pub fn bit_pattern(&self, inline: bool) -> String {
        let mut field_number = [0u8; 32];
        for (i, field) in self.fields.iter().enumerate() {
            for j in 0..field.len {
                field_number[(field.offset + j) as usize] = i as u8 + 1;
            }
        }
        let mut result = String::new();
        if inline {
            result.push_str("\\noindent \\hfill");
        }
        result.push_str("\\begin{tikzpicture}[x=0.75ex,y=1ex,baseline=0.7ex]\n");
        if inline {
            result.push_str("\\useasboundingbox (0,-1) rectangle (96,3.5) ;\n");
        } else {
            result.push_str("\\useasboundingbox (0,0) rectangle (96,3) ;\n");
        }
        for i in 0..32 {
            if i > 0 {
                if field_number[i] != field_number[i - 1] {
                    result.push_str(&format!("\\draw ({},0) -- +(0,3) ;\n", (32 - i) * 3));
                } else {
                    result.push_str(&format!(
                        "\\draw[gray] ({},0) -- +(0,0.7) ;\n",
                        (32 - i) * 3
                    ));
                }
            }
            if field_number[i] == 0 {
                result.push_str(&format!(
                    "\\node[anchor=base] at ({}.5,0.7) {{{}}} ;\n",
                    (32 - i) * 3 - 2,
                    (self.opcode >> i) & 1
                ));
            }
        }
        result.push_str("\\draw (0,0) rectangle +(96,3) ;\n");
        for field in &self.fields {
            result.push_str(&format!(
                "\\node[anchor=base] at ({},0.8) {{${}$}} ;\n",
                ((32 - field.offset) * 3) as f32 - field.len as f32 * 1.5,
                field.name
            ));
        }
        result.push_str("\\end{tikzpicture}\n");
        result
    }
}

pub const EEFC0_FMR: RegisterType<1> = RegisterType::<1> {
    address: 0x400E0A00,
    opcode: 0,
    fields: [Field {
        name: "wait",
        offset: 8,
        len: 4,
    }],
};

pub const EEFC0_FCR: RegisterType<3> = RegisterType::<3> {
    address: 0x400E0A04,
    opcode: 0,
    fields: [
        Field {
            name: "password",
            offset: 24,
            len: 8,
        },
        Field {
            name: "argument",
            offset: 8,
            len: 16,
        },
        Field {
            name: "command",
            offset: 0,
            len: 8,
        },
    ],
};

pub const EEFC0_FSR: RegisterType<1> = RegisterType::<1> {
    address: 0x400E0A08,
    opcode: 0,
    fields: [Field {
        name: "r",
        offset: 0,
        len: 1,
    }],
};

pub const EEFC1_FMR: RegisterType<1> = RegisterType::<1> {
    address: 0x400E0C00,
    ..EEFC0_FMR
};

pub const EEFC1_FCR: RegisterType<3> = RegisterType::<3> {
    address: 0x400E0C04,
    ..EEFC0_FCR
};

pub const EEFC1_FSR: RegisterType<1> = RegisterType::<1> {
    address: 0x400E0C08,
    ..EEFC0_FSR
};

pub const CONTROL: RegisterType<2> = RegisterType::<2> {
    address: 0,
    opcode: 0,
    fields: [
        Field {
            name: "p",
            offset: 0,
            len: 1,
        },
        Field {
            name: "s",
            offset: 1,
            len: 1,
        },
    ],
};

pub const MPU_CONTROL: RegisterType<2> = RegisterType::<2> {
    address: 0xE000ED94,
    opcode: 0,
    fields: [
        Field {
            name: "e",
            offset: 0,
            len: 1,
        },
        Field {
            name: "b",
            offset: 2,
            len: 1,
        },
    ],
};

pub const MPU_RNR: RegisterType<1> = RegisterType::<1> {
    address: 0xE000ED98,
    opcode: 0,
    fields: [Field {
        name: "\\it{region}",
        offset: 0,
        len: 8,
    }],
};

pub const MPU_RBAR: RegisterType<3> = RegisterType::<3> {
    address: 0xE000ED9C,
    opcode: 0,
    fields: [
        Field {
            name: "\\it{base\\ address}",
            offset: 5,
            len: 27,
        },
        Field {
            name: "v",
            offset: 4,
            len: 1,
        },
        Field {
            name: "\\it{region}",
            offset: 0,
            len: 4,
        },
    ],
};

pub const MPU_RASR: RegisterType<5> = RegisterType::<5> {
    address: 0xE000EDA0,
    opcode: 0,
    fields: [
        Field {
            name: "e",
            offset: 0,
            len: 1,
        },
        Field {
            name: "\\it{size}",
            offset: 1,
            len: 5,
        },
        Field {
            name: "\\it{subregions}",
            offset: 8,
            len: 8,
        },
        Field {
            name: "\\it{attributes}",
            offset: 16,
            len: 6,
        },
        Field {
            name: "\\it{access}",
            offset: 24,
            len: 3,
        },
    ],
};

pub const NVIC_ISER0: u32 = 0xE000E100;
pub const NVIC_ICER0: u32 = 0xE000E180;

pub const PIOA_PER: u32 = 0x400E0E00;
pub const PIOA_PDR: u32 = 0x400E0E04;
pub const PIOA_PSR: u32 = 0x400E0E08;
pub const PIOA_OER: u32 = 0x400E0E10;
pub const PIOA_ODR: u32 = 0x400E0E14;
pub const PIOA_OSR: u32 = 0x400E0E18;
pub const PIOA_SODR: u32 = 0x400E0E30;
pub const PIOA_CODR: u32 = 0x400E0E34;
pub const PIOA_ODSR: u32 = 0x400E0E38;
pub const PIO_PDSR: u32 = 0x400E0E3C;
pub const PIOA_PUDR: u32 = 0x400E0E60;
pub const PIOA_PUER: u32 = 0x400E0E64;
pub const PIOA_PUSR: u32 = 0x400E0E68;
pub const PIOA_ABSR: u32 = 0x400E0E70;

pub const PIOB_PER: u32 = 0x400E1000;
pub const PIOB_PDR: u32 = 0x400E1004;
pub const PIOB_PSR: u32 = 0x400E1008;
pub const PIOB_OER: u32 = 0x400E1010;
pub const PIOB_ODR: u32 = 0x400E1014;
pub const PIOB_OSR: u32 = 0x400E1018;
pub const PIOB_SODR: u32 = 0x400E1030;
pub const PIOB_CODR: u32 = 0x400E1034;
pub const PIOB_ODSR: u32 = 0x400E1038;
pub const PIOB_PUDR: u32 = 0x400E1060;
pub const PIOB_PUER: u32 = 0x400E1064;
pub const PIOB_PUSR: u32 = 0x400E1068;
pub const PIOB_ABSR: u32 = 0x400E1070;

pub const PMC_PCER0: u32 = 0x400E0610;
pub const PMC_PCDR0: u32 = 0x400E0614;
pub const PMC_PCSR0: u32 = 0x400E0618;

pub const PMC_MOR: RegisterType<5> = RegisterType::<5> {
    address: 0x400E0620,
    opcode: 0,
    fields: [
        Field {
            name: "r",
            offset: 0,
            len: 1,
        },
        Field {
            name: "c",
            offset: 3,
            len: 1,
        },
        Field {
            name: "startup",
            offset: 8,
            len: 8,
        },
        Field {
            name: "password",
            offset: 16,
            len: 8,
        },
        Field {
            name: "s",
            offset: 24,
            len: 1,
        },
    ],
};

pub const PMC_PLLAR: RegisterType<3> = RegisterType::<3> {
    address: 0x400E0628,
    opcode: 1 << 29,
    fields: [
        Field {
            name: "divider",
            offset: 0,
            len: 8,
        },
        Field {
            name: "startup",
            offset: 8,
            len: 6,
        },
        Field {
            name: "multiplier",
            offset: 16,
            len: 11,
        },
    ],
};

pub const PMC_MCKR: RegisterType<1> = RegisterType::<1> {
    address: 0x400E0630,
    opcode: 0,
    fields: [Field {
        name: "css",
        offset: 0,
        len: 2,
    }],
};

pub const PMC_SR: RegisterType<4> = RegisterType::<4> {
    address: 0x400E0668,
    opcode: 0,
    fields: [
        Field {
            name: "c",
            offset: 0,
            len: 1,
        },
        Field {
            name: "p",
            offset: 1,
            len: 1,
        },
        Field {
            name: "m",
            offset: 3,
            len: 1,
        },
        Field {
            name: "s",
            offset: 16,
            len: 1,
        },
    ],
};

pub const RSTC_CR: u32 = 0x400E1A00;

pub const SCB_VTOR: u32 = 0xE000ED08;

pub const SPI_CR: RegisterType<1> = RegisterType::<1> {
    address: 0x40008000,
    opcode: 0,
    fields: [Field {
        name: "e",
        offset: 0,
        len: 1,
    }],
};

pub const SPI_MR: RegisterType<1> = RegisterType::<1> {
    address: 0x40008004,
    opcode: 0,
    fields: [Field {
        name: "m",
        offset: 0,
        len: 1,
    }],
};

pub const SPI_RDR: RegisterType<1> = RegisterType::<1> {
    address: 0x40008008,
    opcode: 0,
    fields: [Field {
        name: "rd",
        offset: 0,
        len: 16,
    }],
};

pub const SPI_TDR: RegisterType<1> = RegisterType::<1> {
    address: 0x4000800C,
    opcode: 0,
    fields: [Field {
        name: "td",
        offset: 0,
        len: 16,
    }],
};

pub const SPI_CSR: RegisterType<3> = RegisterType::<3> {
    address: 0x40008030,
    opcode: 0,
    fields: [
        Field {
            name: "c",
            offset: 1,
            len: 1,
        },
        Field {
            name: "bits",
            offset: 4,
            len: 4,
        },
        Field {
            name: "divider",
            offset: 8,
            len: 8,
        },
    ],
};

pub const SPI_SR: RegisterType<2> = RegisterType::<2> {
    address: 0x40008010,
    opcode: 0,
    fields: [
        Field {
            name: "r",
            offset: 0,
            len: 1,
        },
        Field {
            name: "t",
            offset: 1,
            len: 1,
        },
    ],
};

pub const SYSTICK_CTRL: RegisterType<2> = RegisterType::<2> {
    address: 0xE000E010,
    opcode: 0,
    fields: [
        Field {
            name: "e",
            offset: 0,
            len: 1,
        },
        Field {
            name: "z",
            offset: 16,
            len: 1,
        },
    ],
};

pub const SYSTICK_LOAD: RegisterType<1> = RegisterType::<1> {
    address: 0xE000E014,
    opcode: 0,
    fields: [Field {
        name: "reload",
        offset: 0,
        len: 24,
    }],
};

pub const SYSTICK_CURRENT: RegisterType<1> = RegisterType::<1> {
    address: 0xE000E018,
    opcode: 0,
    fields: [Field {
        name: "current",
        offset: 0,
        len: 24,
    }],
};

pub const USART_CR: RegisterType<2> = RegisterType::<2> {
    address: 0x40098000,
    opcode: 0,
    fields: [
        Field {
            name: "e_r",
            offset: 4,
            len: 1,
        },
        Field {
            name: "e_t",
            offset: 6,
            len: 1,
        },
    ],
};

pub const USART_MR: RegisterType<6> = RegisterType::<6> {
    address: 0x40098004,
    opcode: 0,
    fields: [
        Field {
            name: "clk",
            offset: 4,
            len: 2,
        },
        Field {
            name: "bits",
            offset: 6,
            len: 2,
        },
        Field {
            name: "s",
            offset: 8,
            len: 1,
        },
        Field {
            name: "parity",
            offset: 9,
            len: 3,
        },
        Field {
            name: "stop",
            offset: 12,
            len: 2,
        },
        Field {
            name: "o",
            offset: 16,
            len: 1,
        },
    ],
};

pub const USART_IER: RegisterType<2> = RegisterType::<2> {
    address: 0x40098008,
    opcode: 0,
    fields: [
        Field {
            name: "i_r",
            offset: 0,
            len: 1,
        },
        Field {
            name: "i_t",
            offset: 1,
            len: 1,
        },
    ],
};

pub const USART_CSR: RegisterType<2> = RegisterType::<2> {
    address: 0x40098014,
    opcode: 0,
    fields: [
        Field {
            name: "s_r",
            offset: 0,
            len: 1,
        },
        Field {
            name: "s_t",
            offset: 1,
            len: 1,
        },
    ],
};

pub const USART_RHR: RegisterType<1> = RegisterType::<1> {
    address: 0x40098018,
    opcode: 0,
    fields: [Field {
        name: "chr",
        offset: 0,
        len: 9,
    }],
};

pub const WDT_MR: u32 = 0x400E1A54;
