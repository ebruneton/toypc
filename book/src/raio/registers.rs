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
    pub id: u8,
    pub name: &'static str,
    pub base: u8,
    pub fields: [Field; F],
}

impl<const F: usize> RegisterType<F> {
    pub fn definition(&self) -> String {
        format!(
            "\\makebox[3.5em][l]{{\\sffamily\\bfseries R{:02X}$_{{16}}$}} {} \\hfill {}",
            self.id,
            self.name,
            self.bit_pattern()
        )
    }

    pub fn bit_pattern(&self) -> String {
        let mut field_number = [0u8; 8];
        for (i, field) in self.fields.iter().enumerate() {
            for j in 0..field.len {
                field_number[(field.offset + j) as usize] = i as u8 + 1;
            }
        }
        let mut result = String::new();
        result.push_str("\\begin{tikzpicture}[x=0.75ex,y=1ex,baseline=0.7ex]\n");
        result.push_str("\\useasboundingbox (0,0) rectangle +(24,3) ;\n");
        for i in 0..8 {
            if i > 0 {
                if field_number[i] != field_number[i - 1] {
                    result.push_str(&format!("\\draw ({},0) -- +(0,3) ;\n", (8 - i) * 3));
                } else {
                    result.push_str(&format!("\\draw[gray] ({},0) -- +(0,0.7) ;\n", (8 - i) * 3));
                }
            }
            if field_number[i] == 0 {
                result.push_str(&format!(
                    "\\node[anchor=base] at ({}.5,0.7) {{{}}} ;\n",
                    (8 - i) * 3 - 2,
                    (self.base >> i) & 1
                ));
            }
        }
        result.push_str("\\draw (0,0) rectangle +(24,3) ;\n");
        for field in &self.fields {
            result.push_str(&format!(
                "\\node[anchor=base] at ({},0.8) {{${}$}} ;\n",
                ((8 - field.offset) * 3) as f32 - field.len as f32 * 1.5,
                field.name
            ));
        }
        result.push_str("\\end{tikzpicture}\n");
        result
    }
}

pub const RAIO_PWRR: RegisterType<1> = RegisterType::<1> {
    id: 0x01,
    name: "Power and Display Control",
    base: 0,
    fields: [Field {
        name: "d",
        offset: 7,
        len: 1,
    }],
};

pub const RAIO_PCSR: RegisterType<2> = RegisterType::<2> {
    id: 0x04,
    name: "Pixel Clock Setting",
    base: 0,
    fields: [
        Field {
            name: "i",
            offset: 7,
            len: 1,
        },
        Field {
            name: "p",
            offset: 0,
            len: 2,
        },
    ],
};

pub const RAIO_HDWR: RegisterType<1> = RegisterType::<1> {
    id: 0x14,
    name: "LCD Horizontal Display Width",
    base: 0,
    fields: [Field {
        name: "w",
        offset: 0,
        len: 7,
    }],
};

pub const RAIO_HNDFTR: RegisterType<1> = RegisterType::<1> {
    id: 0x15,
    name: "LCD Horizontal Non-Display Period Fine Tuning",
    base: 0,
    fields: [Field {
        name: "hndft",
        offset: 0,
        len: 4,
    }],
};

pub const RAIO_HNDR: RegisterType<1> = RegisterType::<1> {
    id: 0x16,
    name: "LCD Horizontal Non-Display Period",
    base: 0,
    fields: [Field {
        name: "hnd",
        offset: 0,
        len: 5,
    }],
};

pub const RAIO_HSTR: RegisterType<1> = RegisterType::<1> {
    id: 0x17,
    name: "HSYNC Start Position",
    base: 0,
    fields: [Field {
        name: "hsp",
        offset: 0,
        len: 5,
    }],
};

pub const RAIO_HPWR: RegisterType<1> = RegisterType::<1> {
    id: 0x18,
    name: "HSYNC Pulse Width",
    base: 0,
    fields: [Field {
        name: "hpw",
        offset: 0,
        len: 5,
    }],
};

pub const RAIO_VHDR0: RegisterType<1> = RegisterType::<1> {
    id: 0x19,
    name: "LCD Vertical Display Height 0",
    base: 0,
    fields: [Field {
        name: "h_0",
        offset: 0,
        len: 8,
    }],
};

pub const RAIO_VHDR1: RegisterType<1> = RegisterType::<1> {
    id: 0x1A,
    name: "LCD Vertical Display Height 1",
    base: 0,
    fields: [Field {
        name: "h_1",
        offset: 0,
        len: 1,
    }],
};

pub const RAIO_VNDR0: RegisterType<1> = RegisterType::<1> {
    id: 0x1B,
    name: "LCD Vertical Non-Display Period 0",
    base: 0,
    fields: [Field {
        name: "vnd",
        offset: 0,
        len: 8,
    }],
};

pub const RAIO_VSTR0: RegisterType<1> = RegisterType::<1> {
    id: 0x1D,
    name: "VSYNC Start Position 0",
    base: 0,
    fields: [Field {
        name: "vsp",
        offset: 0,
        len: 8,
    }],
};

pub const RAIO_VPWR: RegisterType<1> = RegisterType::<1> {
    id: 0x1F,
    name: "VSYNC Pulse Width",
    base: 0,
    fields: [Field {
        name: "vpw",
        offset: 0,
        len: 7,
    }],
};

pub const RAIO_DPCR: RegisterType<1> = RegisterType::<1> {
    id: 0x20,
    name: "Display Configuration",
    base: 0,
    fields: [Field {
        name: "l",
        offset: 7,
        len: 1,
    }],
};

pub const RAIO_F_CURXL: RegisterType<1> = RegisterType::<1> {
    id: 0x2A,
    name: "Font Write Cursor Horizontal Position 0",
    base: 0,
    fields: [Field {
        name: "x_0",
        offset: 0,
        len: 8,
    }],
};

pub const RAIO_F_CURXH: RegisterType<1> = RegisterType::<1> {
    id: 0x2B,
    name: "Font Write Cursor Horizontal Position 1",
    base: 0,
    fields: [Field {
        name: "x_1",
        offset: 0,
        len: 2,
    }],
};

pub const RAIO_F_CURYL: RegisterType<1> = RegisterType::<1> {
    id: 0x2C,
    name: "Font Write Cursor Vertical Position 0",
    base: 0,
    fields: [Field {
        name: "y_0",
        offset: 0,
        len: 8,
    }],
};

pub const RAIO_F_CURYH: RegisterType<1> = RegisterType::<1> {
    id: 0x2D,
    name: "Font Write Cursor Vertical Position 1",
    base: 0,
    fields: [Field {
        name: "y_1",
        offset: 0,
        len: 1,
    }],
};

pub const RAIO_HEAW0: RegisterType<1> = RegisterType::<1> {
    id: 0x34,
    name: "Horizontal End Point of Active Window 0",
    base: 0,
    fields: [Field {
        name: "X_0",
        offset: 0,
        len: 8,
    }],
};

pub const RAIO_HEAW1: RegisterType<1> = RegisterType::<1> {
    id: 0x35,
    name: "Horizontal End Point of Active Window 1",
    base: 0,
    fields: [Field {
        name: "X_1",
        offset: 0,
        len: 2,
    }],
};

pub const RAIO_VEAW0: RegisterType<1> = RegisterType::<1> {
    id: 0x36,
    name: "Vertical End Point of Active Window 0",
    base: 0,
    fields: [Field {
        name: "Y_0",
        offset: 0,
        len: 8,
    }],
};

pub const RAIO_VEAW1: RegisterType<1> = RegisterType::<1> {
    id: 0x37,
    name: "Vertical End Point of Active Window 1",
    base: 0,
    fields: [Field {
        name: "Y_1",
        offset: 0,
        len: 1,
    }],
};

pub const RAIO_MWCR0: RegisterType<3> = RegisterType::<3> {
    id: 0x40,
    name: "Memory Write Control 0",
    base: 0,
    fields: [
        Field {
            name: "t",
            offset: 7,
            len: 1,
        },
        Field {
            name: "c",
            offset: 6,
            len: 1,
        },
        Field {
            name: "b",
            offset: 5,
            len: 1,
        },
    ],
};

pub const RAIO_MWCR1: RegisterType<1> = RegisterType::<1> {
    id: 0x41,
    name: "Memory Write Control 1",
    base: 0,
    fields: [Field {
        name: "l",
        offset: 0,
        len: 1,
    }],
};

pub const RAIO_BTCR: RegisterType<1> = RegisterType::<1> {
    id: 0x44,
    name: "Blink Time Control",
    base: 0,
    fields: [Field {
        name: "blink",
        offset: 0,
        len: 8,
    }],
};

pub const RAIO_LTPR0: RegisterType<1> = RegisterType::<1> {
    id: 0x52,
    name: "Layer Transparency 0",
    base: 0,
    fields: [Field {
        name: "l",
        offset: 0,
        len: 1,
    }],
};

pub const RAIO_FGCR0: RegisterType<1> = RegisterType::<1> {
    id: 0x63,
    name: "Foreground Color 0",
    base: 0,
    fields: [Field {
        name: "\\mathit{fc}_r",
        offset: 0,
        len: 3,
    }],
};

pub const RAIO_FGCR1: RegisterType<1> = RegisterType::<1> {
    id: 0x64,
    name: "Foreground Color 1",
    base: 0,
    fields: [Field {
        name: "\\mathit{fc}_g",
        offset: 0,
        len: 3,
    }],
};

pub const RAIO_FGCR2: RegisterType<1> = RegisterType::<1> {
    id: 0x65,
    name: "Foreground Color 2",
    base: 0,
    fields: [Field {
        name: "\\mathit{fc}_b",
        offset: 0,
        len: 2,
    }],
};

pub const RAIO_PLLC1: RegisterType<2> = RegisterType::<2> {
    id: 0x88,
    name: "PLL Control 1",
    base: 0,
    fields: [
        Field {
            name: "m",
            offset: 7,
            len: 1,
        },
        Field {
            name: "n",
            offset: 0,
            len: 5,
        },
    ],
};

pub const RAIO_PLLC2: RegisterType<1> = RegisterType::<1> {
    id: 0x89,
    name: "PLL Control 2",
    base: 0,
    fields: [Field {
        name: "k",
        offset: 0,
        len: 3,
    }],
};

pub const RAIO_P1CR: RegisterType<1> = RegisterType::<1> {
    id: 0x8A,
    name: "PWM1 Control",
    base: 0,
    fields: [Field {
        name: "l",
        offset: 6,
        len: 1,
    }],
};

pub const RAIO_MCLR: RegisterType<2> = RegisterType::<2> {
    id: 0x8E,
    name: "Memory Clear Control",
    base: 0,
    fields: [
        Field {
            name: "s",
            offset: 7,
            len: 1,
        },
        Field {
            name: "a",
            offset: 6,
            len: 1,
        },
    ],
};

pub const RAIO_GPIOX: RegisterType<1> = RegisterType::<1> {
    id: 0xC7,
    name: "Extra General Purpose IO",
    base: 0,
    fields: [Field {
        name: "x",
        offset: 0,
        len: 1,
    }],
};
