use std::rc::Rc;

use crate::bus::Bus;
use crate::decode::*;
// use crate::helper::combine_to_u16;
use crate::instruction::*;
use crate::registers::Registers;
use crate::helper::combine_to_u16;

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

    fn read_8bit(&mut self) -> u8 {
        let data = self.bus.read(self.regs.pc);
        self.regs.pc += 1;
        data
    }

    fn read_16bit(&mut self) -> u16 {
        let high = self.read_8bit(&mut self);
        let low = self.read_8bit(&mut self);
        combine_to_u16(high, low)
    }

    fn fetch(&mut self) -> Inst {
        let fetched = self.read_8bit();
        let mut inst = decode_unprefixed(fetched);
        if inst == Inst::Prefix {
            let fetched = self.read_8bit();
            inst = decode_prefixed(fetched);
        }
        println!("{:?}", inst);
        inst
    }

    fn get_8bit(&self, reg: Reg8) -> u8 {
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

    fn get_16bit(&self, reg: Reg16) -> u16 {
        match reg {
            Reg16::Af => self.regs.af(),
            Reg16::Bc => self.regs.bc(),
            Reg16::De => self.regs.de(),
            Reg16::Hl => self.regs.hl(),
            Reg16::HlIncr => {
                let hl = self.regs.hl();
                self.regs.incr_hl();
                hl
                },
            Reg16::HlDecr => {
                let hl = self.regs.hl();
                self.regs.decr_hl();
                hl
            }
            Reg16::Sp => self.regs.sp(),
            _ => unreachable!(),
        }
    }

    pub fn execute(&mut self, inst: Inst) {
        let data8: u8 = 0;
        let data16: u16 = 0;
        match inst {
            Inst::Ld(o1, o2) => self.execute_ld(o1, o2),
            _ => todo!("Not implemented!"),
        };
    }

    fn execute_ld(&mut self, dest: Operand, source: Operand) {
        match source {
            Operand::D8 => {
                self.read_8bit(),
            },
            Operand::R8(reg) => self.regs.get_8bit(reg),
        }


    fn execute_push(reg: Reg16) {}

    fn execute_pop(reg: Reg16) {}
}
