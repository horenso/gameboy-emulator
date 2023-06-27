use std::arch::x86_64::CpuidResult;
use std::borrow::BorrowMut;
use std::cell::RefCell;

use crate::cartridge::Cartridge;
use crate::proc::cpu::Cpu;
use crate::util::helper::{combine_to_u16, split_u16};

const V_RAM_SIZE: usize = 8192;
const W_RAM_SIZE: usize = 8192;
const OAM_SIZE: usize = 160;
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
    cartridge: Cartridge,
    v_ram: [u8; V_RAM_SIZE], // video ram
    w_ram: [u8; W_RAM_SIZE], // work ram
    oam: [u8; OAM_SIZE],     // object attribute memory
    h_ram: [u8; H_RAM_SIZE], // high ram
    pub v_ram_dirty: bool,

    pub current_ff_44: RefCell<u8>,
}

impl Bus {
    pub fn new(cartridge: Cartridge) -> Self {
        Bus {
            cartridge,
            v_ram: [0; V_RAM_SIZE],
            w_ram: [0; W_RAM_SIZE],
            oam: [0; OAM_SIZE],
            h_ram: [0; H_RAM_SIZE],
            v_ram_dirty: false,

            current_ff_44: RefCell::new(0),
        }
    }

    // https://gbdev.io/pandocs/Memory_Map.html
    pub fn read(&self, mapped_cpu: Option<&Cpu>, address: u16) -> u8 {
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
                // match mapped_cpu {
                //     Some(cpu) => {
                //         if cpu.dma.is_active() {
                //             return 0xFF;
                //         }
                //     }
                //     _ => (),
                // }
                let oam_address = (address - OAM_START) as usize;
                self.oam[oam_address]
            }
            0xFF44 => 0x90, // TODO: Remove this, this is for Gameboy Doctor
            IO_REGS_START..=IO_REGS_END => {
                if let Some(cpu) = mapped_cpu {
                    let (_, lower) = split_u16(address);
                    self.read_mapped_io_register(cpu, lower)
                } else {
                    0
                }
            }
            H_RAM_START..=H_RAM_END => {
                let h_ram_address = (address - H_RAM_START) as usize;
                self.h_ram[h_ram_address]
            }
            INTERRUPT_ENABLED => {
                if let Some(cpu) = mapped_cpu {
                    cpu.interrupt_handler.enabled()
                } else {
                    0
                }
            }
            _ => 0, // TODO: _ => unreachable!(),
        }
    }

    fn read_mapped_io_register(&self, cpu: &Cpu, offset: u8) -> u8 {
        match offset {
            0x04 => cpu.timer.divider(),
            0x05 => cpu.timer.counter(),
            0x06 => cpu.timer.modulo(),
            0x07 => cpu.timer.control(),

            0x40 => {
                eprintln!("reading from 40");
                0
            }

            0x46 => {
                if cpu.dma.is_active() {
                    1
                } else {
                    0
                }
            }

            0x0F => cpu.interrupt_handler.requested(),
            _ => {
                eprintln!("{} is not mapped yet!", offset);
                0
            }
        }
    }

    pub fn write(&mut self, cpu: &mut Cpu, address: u16, data: u8) {
        // println!("Writing to address: {:#x} data: {:#x}", address, data);
        match address {
            CART_START..=CART_END => {
                // eprintln!("Writting to cartridge at {:x}", address);
            }
            V_RAM_START..=V_RAM_END => {
                let v_ram_address = (address - V_RAM_START) as usize;
                self.v_ram[v_ram_address] = data;
                self.v_ram_dirty = true
            }
            W_RAM_START..=W_RAM_END => {
                let h_ram_address = (address - W_RAM_START) as usize;
                self.w_ram[h_ram_address] = data
            }
            OAM_START..=OAM_END => {
                let (_, lower) = split_u16(address);
                self.write_oam(lower, data);
            }
            IO_REGS_START..=IO_REGS_END => {
                let (_, lower) = split_u16(address);
                self.write_mapped_io_register(cpu, lower, data);
            }
            H_RAM_START..=H_RAM_END => {
                let h_ram_address = (address - H_RAM_START) as usize;
                self.h_ram[h_ram_address] = data
            }
            INTERRUPT_ENABLED => {
                // eprintln!("Writting to FFFF to enable {:b}", data);
                cpu.interrupt_handler.set_enabled(data);
            }
            _ => (), // TODO: unreachable
        }
    }

    pub fn write_oam(&mut self, offset: u8, data: u8) {
        let oam_address = combine_to_u16(0xFE, offset) - OAM_START;
        self.oam[oam_address as usize] = data
    }

    fn write_mapped_io_register(&self, cpu: &mut Cpu, offset: u8, data: u8) {
        match offset {
            0x04 => cpu.timer.reset_divider(),
            0x05 => cpu.timer.set_counter(data),
            0x06 => cpu.timer.set_modulo(data),
            0x07 => cpu.timer.set_control(data),

            0x46 => cpu.dma.start(data),

            0x0F => {
                // eprintln!("Set FF0F requested: {:b}", data);
                cpu.interrupt_handler.set_requested(data)
            }
            // _ => eprintln!("{} is not mapped yet!", offset),
            _ => (),
        }
    }
}
