use super::cartridge::Cartridge;
use super::lcd::{Lcd, Palette};
use super::oam::Oam;
use super::ppu::Ppu;
use crate::memory::dma::Dma;
use crate::memory::interrupts::InterruptHandler;
use crate::memory::timer::Timer;
use crate::util::helper::split_u16;

const V_RAM_SIZE: usize = 8192;
const W_RAM_SIZE: usize = 8192;
const H_RAM_SIZE: usize = 127;

const CART_START: u16 = 0;
const CART_END: u16 = 0x7FFF;

const V_RAM_START: u16 = 0x8000;
const V_RAM_END: u16 = 0x9FFF;

const W_RAM_START: u16 = 0xC000;
const W_RAM_END: u16 = 0xDFFF;

const OAM_START: u16 = 0xFE00;
const OAM_END: u16 = 0xFE9F;

const IO_REGS_START: u16 = 0xFF00;
const IO_REGS_END: u16 = 0xFF70;

const H_RAM_START: u16 = 0xFF80;
const H_RAM_END: u16 = 0xFFFE;

const INTERRUPT_ENABLED: u16 = 0xFFFF;

pub struct Bus {
    pub cartridge: Cartridge,  // mapped in Cartridge data
    pub lcd: Lcd,              // LCD registers
    pub timer: Timer,          // timer registers
    pub int: InterruptHandler, // requested and pending interrupts
    pub dma: Dma,              // Data Transfer unit
    pub oam: Oam,              // Object Attribute Memory
    pub ppu: Ppu,              // Pixel Processing Unit

    v_ram: [u8; V_RAM_SIZE], // video ram
    w_ram: [u8; W_RAM_SIZE], // work ram
    h_ram: [u8; H_RAM_SIZE], // high ram
}

impl Bus {
    pub fn new(cartridge: Cartridge, ppu: Ppu) -> Self {
        Bus {
            cartridge,
            lcd: Lcd::new(),
            timer: Timer::new(),
            int: InterruptHandler::new(),
            dma: Dma::new(),
            oam: Oam::new(),
            ppu,

            v_ram: [0; V_RAM_SIZE],
            w_ram: [0; W_RAM_SIZE],
            h_ram: [0; H_RAM_SIZE],
        }
    }

    // https://gbdev.io/pandocs/Memory_Map.html
    pub fn read(&self, address: u16) -> u8 {
        // println!("Reading bus at {:#x}", address);
        match address {
            CART_START..=CART_END => self.cartridge.read(address as usize),
            V_RAM_START..=V_RAM_END => {
                let v_ram_address = (address - V_RAM_START) as usize;
                self.v_ram[v_ram_address]
            }
            W_RAM_START..=W_RAM_END => {
                let w_ram_address = (address - W_RAM_START) as usize;
                self.w_ram[w_ram_address]
            }
            OAM_START..=OAM_END => {
                let oam_address = address - OAM_START;
                let (_, lower) = split_u16(oam_address);
                self.oam.read(lower)
            }
            IO_REGS_START..=IO_REGS_END => {
                let (_, lower) = split_u16(address);
                self.read_mapped_io_register(lower)
            }
            H_RAM_START..=H_RAM_END => {
                let h_ram_address = (address - H_RAM_START) as usize;
                self.h_ram[h_ram_address]
            }
            INTERRUPT_ENABLED => self.int.enabled(),
            _ => 0, // TODO: _ => unreachable!(),
        }
    }

    fn read_mapped_io_register(&self, offset: u8) -> u8 {
        match offset {
            0x04 => self.timer.divider(),
            0x05 => self.timer.counter(),
            0x06 => self.timer.modulo(),
            0x07 => self.timer.control(),

            0x0F => self.int.requested(),

            0x40 => self.lcd.control,
            0x41 => self.lcd.status,
            0x42 => self.lcd.scroll_y,
            0x43 => self.lcd.scroll_x,
            0x44 => self.lcd.ly,
            0x45 => self.lcd.ly_compare,
            0x46 => {
                if self.dma.is_active() {
                    0xFF
                } else {
                    self.dma.get_last_address()
                }
            }
            0x47 => self.lcd.bg_palette,
            0x48 => self.lcd.obj_palette_0,
            0x49 => self.lcd.obj_palette_1,
            0x4A => self.lcd.win_y,
            0x4B => self.lcd.win_x,
            _ => {
                eprintln!("{} is not mapped yet!", offset);
                0
            }
        }
    }

    pub fn write(&mut self, address: u16, data: u8) {
        // println!("Writing to address: {:#x} data: {:#x}", address, data);
        match address {
            CART_START..=CART_END => self.cartridge.write(address, data),
            V_RAM_START..=V_RAM_END => {
                let v_ram_address = (address - V_RAM_START) as usize;
                self.v_ram[v_ram_address] = data;
            }
            W_RAM_START..=W_RAM_END => {
                let h_ram_address = (address - W_RAM_START) as usize;
                self.w_ram[h_ram_address] = data
            }
            OAM_START..=OAM_END => {
                let (_, lower) = split_u16(address);
                self.oam.write(lower, data);
            }
            IO_REGS_START..=IO_REGS_END => {
                let (_, lower) = split_u16(address);
                self.write_mapped_io_register(lower, data);
            }
            H_RAM_START..=H_RAM_END => {
                let h_ram_address = (address - H_RAM_START) as usize;
                self.h_ram[h_ram_address] = data
            }
            INTERRUPT_ENABLED => {
                // eprintln!("Writting to FFFF to enable {:b}", data);
                self.int.set_enabled(data);
            }
            _ => (), // TODO: unreachable
        }
    }

    fn write_mapped_io_register(&mut self, offset: u8, data: u8) {
        match offset {
            0x04 => self.timer.reset_divider(),
            0x05 => self.timer.set_counter(data),
            0x06 => self.timer.set_modulo(data),
            0x07 => self.timer.set_control(data),

            0x0F => self.int.set_requested(data),

            0x40 => self.lcd.control = data,
            0x41 => self.lcd.status = data,
            0x42 => self.lcd.scroll_y = data,
            0x43 => self.lcd.scroll_x = data,
            0x44 => self.lcd.ly = data,
            0x45 => self.lcd.ly_compare = data,
            0x46 => self.dma.start(data),
            0x47 => self.lcd.update_palette(data, Palette::Background),
            0x48 => self.lcd.update_palette(data, Palette::Obj0),
            0x49 => self.lcd.update_palette(data, Palette::Obj1),
            0x4A => self.lcd.win_y = data,
            0x4B => self.lcd.win_x = data,
            _ => (),
        }
    }
}
