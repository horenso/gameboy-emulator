use crate::cartridge::Cartridge;

const V_RAM_SIZE: usize = 8192;
const W_RAM_SIZE: usize = 8192;
const IO_REGS_SIZE: usize = 113;
const H_RAM_SIZE: usize = 127;

const CART_START: u16 = 0;
const CART_END: u16 = 0x7FFF;

const V_RAM_START: u16 = 0x8000;
const V_RAM_END: u16 = 0x9FFF;

const W_RAM_START: u16 = 0xC000;
const W_RAM_END: u16 = 0xDFFF;

const IO_REGS_START: u16 = 0xFF00;
const IO_REGS_END: u16 = 0xFF70;

const H_RAM_START: u16 = 0xFF80;
const H_RAM_END: u16 = 0xFFFE;

pub struct Bus {
    cartridge: Cartridge,
    v_ram: [u8; V_RAM_SIZE],     // video ram
    w_ram: [u8; W_RAM_SIZE],     // work ram
    io_regs: [u8; IO_REGS_SIZE], // I/O registers like the Joypad
    h_ram: [u8; H_RAM_SIZE],     // high ram
}

impl Bus {
    pub fn new(cartridge: Cartridge) -> Self {
        Bus {
            cartridge,
            v_ram: [0; V_RAM_SIZE],
            w_ram: [0; W_RAM_SIZE],
            io_regs: [0; IO_REGS_SIZE],
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
            0xFF44 => 0x90, // TODO: Remove this, this is for Gameboy Doctor
            IO_REGS_START..=IO_REGS_END => {
                let io_regs_address = (address - IO_REGS_START) as usize;
                self.io_regs[io_regs_address]
            }
            H_RAM_START..=H_RAM_END => {
                let h_ram_address = (address - H_RAM_START) as usize;
                self.h_ram[h_ram_address]
            }
            _ => 0, // TODO: _ => unreachable!(),
        }
    }

    pub fn write(&mut self, address: u16, data: u8) {
        // println!("Writing to address: {:#x} data: {:#x}", address, data);
        match address {
            CART_START..=CART_END => (),
            V_RAM_START..=V_RAM_END => {
                let v_ram_address = (address - V_RAM_START) as usize;
                self.v_ram[v_ram_address] = data
            }
            W_RAM_START..=W_RAM_END => {
                let h_ram_address = (address - W_RAM_START) as usize;
                self.w_ram[h_ram_address] = data
            }
            IO_REGS_START..=IO_REGS_END => {
                let io_regs_address = (address - IO_REGS_START) as usize;
                self.io_regs[io_regs_address] = data
            }
            H_RAM_START..=H_RAM_END => {
                let h_ram_address = (address - H_RAM_START) as usize;
                self.h_ram[h_ram_address] = data
            }
            0xFFFF => {
                // TODO interrupts (write only)
            }
            _ => unreachable!(),
        }
    }
}
