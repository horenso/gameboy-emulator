use crate::cartridge::Cartridge;
use crate::cpu::cpu::Cpu;
use crate::util::helper::split_u16;

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

const INT_MASTER_ENABLED: u16 = 0xFFFF;

pub struct Bus {
    cartridge: Cartridge,
    v_ram: [u8; V_RAM_SIZE], // video ram
    w_ram: [u8; W_RAM_SIZE], // work ram
    h_ram: [u8; H_RAM_SIZE], // high ram
    pub v_ram_dirty: bool,
}

impl Bus {
    pub fn new(cartridge: Cartridge) -> Self {
        Bus {
            cartridge,
            v_ram: [0; V_RAM_SIZE],
            w_ram: [0; W_RAM_SIZE],
            h_ram: [0; H_RAM_SIZE],
            v_ram_dirty: false,
        }
    }

    // https://gbdev.io/pandocs/Memory_Map.html
    pub fn read(&self, cpu: &Cpu, address: u16) -> u8 {
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
                let (_, lower) = split_u16(address);
                self.read_mapped_io_register(cpu, lower)
            }
            H_RAM_START..=H_RAM_END => {
                let h_ram_address = (address - H_RAM_START) as usize;
                self.h_ram[h_ram_address]
            }
            INT_MASTER_ENABLED => {
                if cpu.interrupt_master_enabled {
                    1
                } else {
                    0
                }
            }
            _ => 0, // TODO: _ => unreachable!(),
        }
    }

    fn read_mapped_io_register(&self, cpu: &Cpu, offset: u8) -> u8 {
        match offset {
            0 => 0,
            4 => {
                let (high, _) = split_u16(cpu.divider);
                high
            }
            _ => {
                eprintln!("{} is not mapped yet!", offset);
                0
            }
        }
    }

    pub fn write(&mut self, cpu: &mut Cpu, address: u16, data: u8) {
        // println!("Writing to address: {:#x} data: {:#x}", address, data);
        match address {
            CART_START..=CART_END => (),
            V_RAM_START..=V_RAM_END => {
                let v_ram_address = (address - V_RAM_START) as usize;
                self.v_ram[v_ram_address] = data;
                self.v_ram_dirty = true
            }
            W_RAM_START..=W_RAM_END => {
                let h_ram_address = (address - W_RAM_START) as usize;
                self.w_ram[h_ram_address] = data
            }
            IO_REGS_START..=IO_REGS_END => {
                let (_, lower) = split_u16(address);
                self.write_mapped_io_register(cpu, lower);
            }
            H_RAM_START..=H_RAM_END => {
                let h_ram_address = (address - H_RAM_START) as usize;
                self.h_ram[h_ram_address] = data
            }
            0xFFFF => {
                eprintln!("Writting to FFFF to enable {}", data);
                // TODO interrupts (write only)
            }
            _ => unreachable!(),
        }
    }

    fn write_mapped_io_register(&self, cpu: &mut Cpu, offset: u8) {
        match offset {
            4 => cpu.divider = 0,
            _ => eprintln!("{} is not mapped yet!", offset),
        }
    }
}
