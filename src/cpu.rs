use std::fs::File;
use std::io::Write;
use std::rc::Rc;

use crate::bus::Bus;
use crate::decode::*;
use crate::helper::{combine_to_u16, split_u16};
use crate::instruction::*;
use crate::registers::Registers;

pub struct Cpu {
    regs: Registers,
    bus: Rc<Bus>,
    counter: u64, // count number of executed instructions
}

impl Cpu {
    pub fn new(bus: Rc<Bus>) -> Self {
        Cpu {
            regs: Registers::new(),
            bus,
            counter: 0,
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

    fn load_indr(&mut self, reg: Reg16) -> u8 {
        let addr = self.get_reg16(reg);
        self.bus.read(addr)
    }

    fn save_indr(&mut self, reg: Reg16, data: u8) {
        let addr = self.get_reg16(reg);
        self.bus.write(addr, data);
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
        self.counter += 1;
        println!("Executing instruction: {:?} {}", inst, self.counter);
        match inst {
            Inst::Prefix => unreachable!(),

            Inst::NoOp => (),
            Inst::Halt => self.halt(),
            Inst::Stop => self.stop(),
            Inst::Di => self.di(),
            Inst::Ei => self.ei(),

            Inst::Ld8(dest, source) => self.ld8(dest, source),
            Inst::Ld16(dest, source) => self.ld16(dest, source),
            Inst::Push(reg16) => self.push(reg16),
            Inst::Pop(reg16) => self.pop(reg16),

            Inst::Jr(cond) => self.jr(cond),
            Inst::Jp(cond, dest) => self.jp(cond, dest),
            Inst::Call(cond) => self.call(cond),
            Inst::Ret(cond) => self.ret(cond),
            Inst::Reti => self.reti(),
            Inst::Rst(amount) => self.rst(amount),

            Inst::Add(operand) => self.add_a(operand),
            Inst::AddHl(reg16) => self.add_hl(reg16),
            Inst::AddSp => self.add_sp(),
            Inst::Adc(operand) => self.adc(operand),
            Inst::Sub(operand) => self.sub(operand),
            Inst::Sbc(operand) => self.sbc(operand),
            Inst::And(operand) => self.and(operand),
            Inst::Xor(operand) => self.xor(operand),
            Inst::Or(operand) => self.or(operand),
            Inst::Cp(operand) => self.cp(operand),
            Inst::Inc(operand) => self.inc(operand),
            Inst::Dec(operand) => self.dec(operand),

            Inst::Rlc(operand) => self.rlc(operand),
            Inst::Rrc(operand) => self.rrc(operand),
            Inst::Rl(operand) => self.rl(operand),
            Inst::Rr(operand) => self.rr(operand),
            Inst::Sla(operand) => self.sla(operand),
            Inst::Sra(operand) => self.sra(operand),
            Inst::Swap(operand) => self.swap(operand),
            Inst::Srl(operand) => self.srl(operand),
            Inst::Bit(amount, operand) => self.bit(amount, operand),
            Inst::Res(amount, operand) => self.res(amount, operand),
            Inst::Set(amount, operand) => self.set(amount, operand),

            Inst::Rlca => self.rlca(),
            Inst::Rrca => self.rrca(),
            Inst::Rla => self.rla(),
            Inst::Rra => self.rra(),
            Inst::Daa => self.daa(),
            Inst::Cpl => self.cpl(),
            Inst::Scf => self.scf(),
            Inst::Ccf => self.ccf(),
        };
    }

    fn halt(&mut self) {}

    fn stop(&mut self) {}

    fn di(&mut self) {}

    fn ei(&mut self) {}

    fn ld8(&mut self, dest: Operand, source: Operand) {
        let data = match source {
            Operand::D8 => self.read_next_8bit(),
            Operand::A8 => {
                let addr = combine_to_u16(0xFF, self.read_next_8bit());
                self.bus.read(addr)
            }
            Operand::IndR16(reg16) => self.load_indr(reg16),
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
            Operand::IndR16(reg16) => self.save_indr(reg16, data),
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
            Operand::R16(reg16) => self.set_reg16(reg16, data),
            _ => unreachable!(),
        };
    }

    fn push(&mut self, reg: Reg16) {}

    fn pop(&mut self, reg: Reg16) {}

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

    fn call(&mut self, cond: Cond) {
        if !self.check_cond(cond) {
            return;
        }
        // TODO: call logic
    }

    fn ret(&mut self, cond: Cond) {}

    fn reti(&mut self) {}

    fn rst(&mut self, amount: u8) {}

    fn add_a(&mut self, operand: Operand) {
        let data = match operand {
            Operand::D8 => self.read_next_8bit(),
            Operand::R8(reg8) => self.get_reg8(reg8),
            Operand::IndR16(reg16) => self.load_indr(reg16),
            _ => unreachable!(),
        };
        let sum: u16 = self.regs.a as u16 + data as u16;
        let (result, carry) = split_u16(sum);
        self.regs.a = result;
        self.regs.set_zero(result == 0);
        self.regs.set_subtract(false);
        self.regs.set_half_carry(carry & 0b0000_1000 == 0);
        self.regs.set_carry(carry & 0b1000_0000 == 0);
    }

    fn add_hl(&mut self, reg: Reg16) {
        let data = self.get_reg16(reg);
        let sum = self.regs.hl() + data;
        // TODO: Set flags
    }

    fn add_sp(&mut self) {}

    fn adc(&mut self, operand: Operand) {}

    fn sub(&mut self, operand: Operand) {}

    fn sbc(&mut self, operand: Operand) {}

    fn and(&mut self, operand: Operand) {}

    fn xor(&mut self, operand: Operand) {}

    fn or(&mut self, operand: Operand) {}

    fn cp(&mut self, operand: Operand) {}

    fn inc(&mut self, operand: Operand) {}

    fn dec(&mut self, operand: Operand) {}

    fn rlc(&mut self, operand: Operand) {}

    fn rrc(&mut self, operand: Operand) {}

    fn rl(&mut self, operand: Operand) {}

    fn rr(&mut self, operand: Operand) {}

    fn sla(&mut self, operand: Operand) {}

    fn sra(&mut self, operand: Operand) {}

    fn swap(&mut self, operand: Operand) {}

    fn srl(&mut self, operand: Operand) {}

    fn bit(&mut self, amount: u8, operand: Operand) {}

    fn res(&mut self, amount: u8, operand: Operand) {}

    fn set(&mut self, amount: u8, operand: Operand) {}

    fn rlca(&mut self) {}

    fn rrca(&mut self) {}

    fn rla(&mut self) {}

    fn rra(&mut self) {}

    fn daa(&mut self) {}

    fn cpl(&mut self) {}

    fn scf(&mut self) {}

    fn ccf(&mut self) {}
}
