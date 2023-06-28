use crate::util::helper::{is_bit_set, set_bit};

pub enum LcdMode {
    HBlank,
    VBlank,
    SearchingOam,
    TransferingData,
}

#[derive(PartialEq)]
pub enum Palette {
    Background,
    Obj0,
    Obj1,
}

impl LcdMode {
    pub fn toStatusBit(&self) -> u8 {
        match self {
            Self::HBlank => 0b00,
            Self::VBlank => 0b01,
            Self::SearchingOam => 0b10,
            Self::TransferingData => 0b11,
        }
    }

    pub fn getInterruptSourceBit(&self) -> u8 {
        match self {
            Self::HBlank => 1 << 3,
            Self::VBlank => 1 << 4,
            Self::SearchingOam => 1 << 5,
            Self::TransferingData => 1 << 6,
        }
    }
}

#[derive(Debug)]
pub struct Lcd {
    pub control: u8,    // LCDC: LCD control
    pub status: u8,     // LCDS: LCD status
    pub scroll_y: u8,   // SCY
    pub scroll_x: u8,   // SCX
    pub ly: u8,         // LCD Y
    pub ly_compare: u8, // LYC: LY compare
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
            bg_palette: 0xFC,
            obj_palette_0: 0xFF,
            obj_palette_1: 0xFF,
            win_y: 0,
            win_x: 0,
        }
    }

    pub fn bg_window_enabled(&self) -> bool {
        is_bit_set(self.control, 0)
    }

    pub fn obj_enabled(&self) -> bool {
        is_bit_set(self.control, 1)
    }

    pub fn obj_height(&self) -> u8 {
        if is_bit_set(self.control, 2) {
            16
        } else {
            8
        }
    }

    pub fn bg_map_area(&self) -> u16 {
        if is_bit_set(self.control, 3) {
            0x9C00
        } else {
            0x9800
        }
    }

    pub fn bgw_data_area(&self) -> u16 {
        if is_bit_set(self.control, 4) {
            0x8000
        } else {
            0x8800
        }
    }

    pub fn win_enable(&self) -> bool {
        is_bit_set(self.control, 5)
    }

    pub fn win_map_area(&self) -> u16 {
        if is_bit_set(self.control, 6) {
            0x9C00
        } else {
            0x9800
        }
    }

    pub fn lcd_enable(&self) -> bool {
        is_bit_set(self.control, 7)
    }

    pub fn mode(&self) -> LcdMode {
        match self.status & 0b11 {
            0b00 => LcdMode::HBlank,
            0b01 => LcdMode::VBlank,
            0b10 => LcdMode::SearchingOam,
            0b11 => LcdMode::TransferingData,
            _ => unreachable!(),
        }
    }

    pub fn set_mode(&mut self, mode: LcdMode) {
        self.status &= 0b1111_1100;
        self.status |= mode.toStatusBit();
    }

    pub fn lyc(&self) -> bool {
        is_bit_set(self.status, 2)
    }

    pub fn set_lyc(&mut self, value: bool) {
        self.status = set_bit(self.status, 2, value)
    }

    pub fn status_interrupt_int(&self, source: u8) {
        self.status & source;
    }

    pub fn update_palette(&mut self, value: u8, palette: Palette) {
        match palette {
            Palette::Background => self.bg_palette = value,
            // Object palettes only have three colors,
            // the third is transparent
            Palette::Obj0 => self.obj_palette_0 = value & 0b1111_1100,
            Palette::Obj1 => self.obj_palette_1 = value & 0b1111_1100,
        }
    }
}
