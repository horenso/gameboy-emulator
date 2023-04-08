use std::fs::File;
use std::io::Write;
use std::rc::Rc;

use crate::bus::Bus;
use crate::decode::*;
// use crate::helper::combine_to_u16;
use crate::helper::{combine_to_u16, split_u16};
use crate::instruction::*;
use crate::registers::Registers;

pub struct Cpu {
    regs: Registers,
    bus: Rc<Bus>,
}

impl Cpu {
    pub fn new(bus: Rc<Bus>) -> Self {
        Cpu {
            regs: Registers::new(),
            bus,
        }
    }

    pub fn fetch_and_execute(&mut self) {
        let inst = self.fetch();
        self.execute(inst);
    }

    fn read_next_8bit(&mut self) -> u8 {
        let data = self.bus.read(self.regs.pc);
        self.regs.pc += 1;
        data
    }

    fn read_next_16bit(&mut self) -> u16 {
        let high = self.read_next_8bit();
        let low = self.read_next_8bit();
        combine_to_u16(high, low)
    }

    fn fetch(&mut self) -> Inst {
        let fetched = self.read_next_8bit();
        let mut inst = decode_unprefixed(fetched);
        if inst == Inst::Prefix {
            let fetched = self.read_next_8bit();
            inst = decode_prefixed(fetched);
        }
        println!("Fetched instruction: {:?}", inst);
        inst
    }

    fn get_reg8(&self, reg: Reg8) -> u8 {
        match reg {
            Reg8::A => self.regs.a,
            Reg8::B => self.regs.b,
            Reg8::C => self.regs.c,
            Reg8::D => self.regs.d,
            Reg8::E => self.regs.e,
            Reg8::H => self.regs.h,
            Reg8::L => self.regs.l,
        }
    }

    fn set_reg8(&mut self, reg: Reg8, data: u8) {
        match reg {
            Reg8::A => self.regs.a = data,
            Reg8::B => self.regs.b = data,
            Reg8::C => self.regs.c = data,
            Reg8::D => self.regs.d = data,
            Reg8::E => self.regs.e = data,
            Reg8::H => self.regs.h = data,
            Reg8::L => self.regs.l = data,
        };
    }

    fn set_reg16(&mut self, reg: Reg16, data: u16) {
        match reg {
            Reg16::Af => self.regs.set_af(data),
            Reg16::Bc => self.regs.set_bc(data),
            Reg16::De => self.regs.set_de(data),
            Reg16::Hl => self.regs.set_hl(data),
            _ => unreachable!(),
        };
    }

    fn get_reg16(&mut self, reg: Reg16) -> u16 {
        match reg {
            Reg16::Af => self.regs.af(),
            Reg16::Bc => self.regs.bc(),
            Reg16::De => self.regs.de(),
            Reg16::Hl => self.regs.hl(),
            Reg16::HlIncr => {
                let hl = self.regs.hl();
                self.regs.incr_hl();
                hl
            }
            Reg16::HlDecr => {
                let hl = self.regs.hl();
                self.regs.decr_hl();
                hl
            }
            Reg16::Sp => self.regs.sp,
            _ => unreachable!(),
        }
    }

    fn check_cond(&self, cond: Cond) -> bool {
        match cond {
            Cond::Always => true,
            Cond::Zero => self.regs.zero_flag(),
            Cond::NotZero => !(self.regs.zero_flag()),
            Cond::Carry => self.regs.carry_flag(),
            Cond::NotCarry => !(self.regs.carry_flag()),
        }
    }

    pub fn debug_print(&self, file: &mut File) {
        let p0 = self.bus.read(self.regs.pc);
        let p1 = self.bus.read(self.regs.pc + 1);
        let p2 = self.bus.read(self.regs.pc + 2);
        let p3 = self.bus.read(self.regs.pc + 3);
        writeln!(file, "A:{:02X} F:{:02X} B:{:02X} C:{:02X} D:{:02X} E:{:02X} H:{:02X} L:{:02X} SP:{:02X} PC:{:02X} PCMEM:{:02X},{:02X},{:02X},{:02X}",
            self.regs.a,
            self.regs.f,
            self.regs.b,
            self.regs.c,
            self.regs.d,
            self.regs.e,
            self.regs.h,
            self.regs.l,
            self.regs.sp,
            self.regs.pc,
            p0, p1, p2, p3).unwrap();
    }

    pub fn execute(&mut self, inst: Inst) {
        match inst {
            Inst::NoOp => (),
            Inst::Ld8(dest, source) => self.ld8(dest, source),
            Inst::Ld16(dest, source) => self.ld16(dest, source),
            Inst::Jr(cond) => self.jr(cond),
            Inst::Jp(cond, dest) => self.jp(cond, dest),
            _ => {
                println!("Instruction {:?} not implemented!", inst);
                todo!();
            }
        };
    }

    fn ld8(&mut self, dest: Operand, source: Operand) {
        let data = match source {
            Operand::D8 => self.read_next_8bit(),
            Operand::A8 => {
                let addr = combine_to_u16(0xFF, self.read_next_8bit());
                self.bus.read(addr)
            }
            Operand::IndR16(reg16) => {
                let addr = self.get_reg16(reg16);
                self.bus.read(addr)
            }
            Operand::R8(reg8) => self.get_reg8(reg8),
            _ => unreachable!(),
        };
        match dest {
            Operand::A8 => {
                let addr = combine_to_u16(0xFF, self.read_next_8bit());
                self.bus.write(addr, data);
            }
            Operand::A16 => {
                let addr = self.read_next_16bit();
                self.bus.write(addr, data);
            }
            Operand::IndR16(reg16) => {
                let addr = self.get_reg16(reg16);
                self.bus.write(addr, data);
            }
            Operand::R8(reg8) => {
                self.set_reg8(reg8, data);
            }
            _ => unreachable!(),
        };
    }

    fn ld16(&mut self, dest: Operand, source: Operand) {
        let data = match source {
            Operand::D16 => self.read_next_16bit(),
            Operand::R16(reg16) => self.get_reg16(reg16),
            _ => unreachable!(),
        };
        match dest {
            Operand::A8 => {
                let addr = combine_to_u16(0xFF, self.read_next_8bit());
                let (high, low) = split_u16(data);
                self.bus.write(addr, low);
                self.bus.write(addr + 1, high);
            }
            Operand::A16 => {
                let addr = self.read_next_16bit();
                let (high, low) = split_u16(data);
                self.bus.write(addr, low);
                self.bus.write(addr + 1, high);
            }
            Operand::R16(reg16) => {
                self.set_reg16(reg16, data);
            }
            _ => unreachable!(),
        };
    }

    fn jr(&mut self, cond: Cond) {
        if !self.check_cond(cond) {
            return;
        }
        let data = self.read_next_8bit() as i16;
        self.regs.sp = (self.regs.sp as i16).wrapping_add(data) as u16;
    }

    fn jp(&mut self, cond: Cond, dest: Operand) {
        if !self.check_cond(cond) {
            return;
        }
        let addr = match dest {
            Operand::A16 => self.read_next_16bit(),
            Operand::R16(reg16) => self.get_reg16(reg16),
            _ => unreachable!(),
        };
        self.regs.sp = addr;
    }
}
