use crate::util::helper::{is_bit_set, set_bit};

pub enum LcdMode {
    HBlank,
    VBlank,
    Oam,
    Transfer,
}

#[derive(Debug)]
pub struct Lcd {
    pub control: u8,    // LCDC
    pub status: u8,     // LCDS
    pub scroll_y: u8,   // SCY
    pub scroll_x: u8,   // SCX
    pub ly: u8,         // LCD Y
    pub ly_compare: u8, // LYC: LY compare
    pub dma: u8,
    pub bg_palette: u8,
    pub obj_palette_0: u8,
    pub obj_palette_1: u8,
    pub win_y: u8,
    pub win_x: u8,
}

impl Lcd {
    pub fn new() -> Self {
        Lcd {
            control: 0x91,
            status: 0,
            scroll_y: 0,
            scroll_x: 0,
            ly: 0,
            ly_compare: 0,
            dma: 0,
            bg_palette: 0xFC,
            obj_palette_0: 0xFF,
            obj_palette_1: 0xFF,
            win_y: 0,
            win_x: 0,
        }
    }

    fn bg_window_enabled(&self) -> bool {
        is_bit_set(self.control, 0)
    }

    fn obj_enabled(&self) -> bool {
        is_bit_set(self.control, 1)
    }

    fn obj_height(&self) -> u8 {
        if is_bit_set(self.control, 2) {
            16
        } else {
            8
        }
    }

    fn bg_map_area(&self) -> u16 {
        if is_bit_set(self.control, 3) {
            0x9C00
        } else {
            0x9800
        }
    }
    fn bgw_data_area(&self) -> u16 {
        if is_bit_set(self.control, 4) {
            0x8000
        } else {
            0x8800
        }
    }
    fn win_enable(&self) -> bool {
        is_bit_set(self.control, 5)
    }
    fn win_map_area(&self) -> u16 {
        if is_bit_set(self.control, 6) {
            0x9C00
        } else {
            0x9800
        }
    }
    fn lcd_enable(&self) -> bool {
        is_bit_set(self.control, 7)
    }
    fn mode(&self) -> LcdMode {
        match self.status & 0b11 {
            0 => LcdMode::HBlank,
            1 => LcdMode::VBlank,
            2 => LcdMode::Oam,
            3 => LcdMode::Transfer,
            _ => unreachable!(),
        }
    }
    fn set_mode(&mut self, mode: u8) {
        self.status &= !0b11;
        self.status |= mode;
    }
    fn lyc(&self) -> bool {
        is_bit_set(self.status, 2)
    }
    fn set_lyc(&mut self, value: bool) {
        self.status = set_bit(self.status, 2, value)
    }
}
