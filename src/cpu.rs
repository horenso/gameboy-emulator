use std::io::Write;

use crate::bus::Bus;
use crate::decode::*;
use crate::helper::{combine_to_u16, split_u16, split_u32};
use crate::instruction::*;
use crate::registers::Registers;

pub struct Cpu {
    regs: Registers,
    bus: Bus,
    pub counter: u64, // count number of executed instructions
}

impl Cpu {
    pub fn new(bus: Bus) -> Self {
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
        self.regs.pc = self.regs.pc.wrapping_add(1);
        data
    }

    fn read_next_16bit(&mut self) -> u16 {
        let low = self.read_next_8bit();
        let high = self.read_next_8bit();
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

    fn get_reg8(&self, reg: &Reg8) -> u8 {
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

    fn set_reg8(&mut self, reg: &Reg8, data: u8) {
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

    fn set_reg16(&mut self, reg: &Reg16, data: u16) {
        match reg {
            Reg16::Af => self.regs.set_af(data),
            Reg16::Bc => self.regs.set_bc(data),
            Reg16::De => self.regs.set_de(data),
            Reg16::Hl => self.regs.set_hl(data),
            Reg16::Sp => self.regs.sp = data,
            Reg16::Pc => self.regs.pc = data,
            Reg16::HlIncr | Reg16::HlDecr | Reg16::SpPlusD => unreachable!(),
        };
    }

    fn get_reg16(&mut self, reg: &Reg16) -> u16 {
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
            Reg16::Pc => self.regs.pc,
            Reg16::SpPlusD => unreachable!(),
        }
    }

    fn load_indr(&mut self, reg: &Reg16) -> u8 {
        let addr = self.get_reg16(&reg);
        self.bus.read(addr)
    }

    fn save_indr(&mut self, reg: &Reg16, data: u8) {
        let addr = self.get_reg16(&reg);
        self.bus.write(addr, data);
    }

    fn get_8bit_operand(&mut self, operand: &Operand) -> u8 {
        match operand {
            Operand::D8 => self.read_next_8bit(),
            Operand::R8(reg) => self.get_reg8(&reg),
            Operand::IndR16(reg) => self.load_indr(reg),
            _ => unreachable!(),
        }
    }

    fn set_8bit_operand(&mut self, operand: &Operand, data: u8) {
        match operand {
            Operand::R8(reg) => self.set_reg8(reg, data),
            Operand::IndR16(reg) => {
                let addr = self.get_reg16(reg);
                self.bus.write(addr, data);
            }
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

    pub fn debug_print(&self, file: &mut impl Write) {
        let p0 = self.bus.read(self.regs.pc);
        let p1 = self.bus.read(self.regs.pc.wrapping_add(1));
        let p2 = self.bus.read(self.regs.pc.wrapping_add(2));
        let p3 = self.bus.read(self.regs.pc.wrapping_add(3));
        writeln!(
            file,
            concat!(
                "A:{:02X} F:{:02X} B:{:02X} C:{:02X} D:{:02X} E:{:02X} H:{:02X} L:{:02X} ",
                "SP:{:04X} PC:{:04X} ",
                "PCMEM:{:02X},{:02X},{:02X},{:02X}",
            ),
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
            p0,
            p1,
            p2,
            p3,
        )
        .unwrap();
    }

    pub fn execute(&mut self, inst: Inst) {
        self.counter += 1;
        // if self.counter % 10000 == 0 {
        // pristdout"{:08}: {:?}", self.counter, inst);
        // }
        match inst {
            Inst::Prefix => unreachable!(),

            Inst::NoOp => (),
            Inst::Halt => self.halt(),
            Inst::Stop => self.stop(),
            Inst::Di => self.di(),
            Inst::Ei => self.ei(),

            Inst::Ld8(dest, source) => self.ld8(dest, source),
            Inst::Ld16(dest, source) => self.ld16(dest, source),
            Inst::Push(reg) => self.push(reg),
            Inst::Pop(reg) => self.pop(reg),

            Inst::Jr(cond) => self.jr(cond),
            Inst::Jp(cond, dest) => self.jp(cond, dest),
            Inst::Call(cond) => self.call(cond),
            Inst::Ret(cond) => self.ret(cond),
            Inst::Reti => self.reti(),
            Inst::Rst(amount) => self.rst(amount),

            Inst::Add(operand) => self.add_a(operand, false),
            Inst::AddHl(reg) => self.add_hl(reg),
            Inst::AddSp => self.add_sp(),
            Inst::Adc(operand) => self.add_a(operand, true),
            Inst::Sub(operand) => self.sub_a(operand, false, true),
            Inst::Sbc(operand) => self.sub_a(operand, true, true),
            Inst::And(operand) => self.and(operand),
            Inst::Xor(operand) => self.xor(operand),
            Inst::Or(operand) => self.or(operand),
            Inst::Cp(operand) => self.sub_a(operand, false, false),
            Inst::Inc8(operand) => self.inc8(operand),
            Inst::Inc16(reg) => self.inc16(reg),
            Inst::Dec8(operand) => self.dec8(operand),
            Inst::Dec16(reg) => self.dec16(reg),

            Inst::Rotate(rotation, operand) => self.rotate(rotation, operand),
            Inst::Shift(shift, operand) => self.shift(shift, operand),
            Inst::Swap(operand) => self.swap(operand),
            Inst::TestBit(amount, operand) => self.test_bit(amount, operand),
            Inst::ResetBit(amount, operand) => self.reset_bit(amount, operand),
            Inst::SetBit(amount, operand) => self.set_bit(amount, operand),

            Inst::Daa => self.daa(),
            Inst::ComplementA => self.complement_a(),
            Inst::SetCarryFlag => self.set_carry(),
            Inst::ComplementCarryFlag => self.complement_carry_flag(),
        };
    }

    fn halt(&mut self) {}

    fn stop(&mut self) {
        panic!("STOPPPPP");
    }

    fn di(&mut self) {}

    fn ei(&mut self) {}

    fn ld8(&mut self, dest: Operand, source: Operand) {
        let data = match source {
            Operand::D8 => self.read_next_8bit(),
            Operand::A8 => {
                let addr = combine_to_u16(0xFF, self.read_next_8bit());
                self.bus.read(addr)
            }
            Operand::A16 => {
                let addr = self.read_next_16bit();
                self.bus.read(addr)
            }
            Operand::IndR16(reg) => self.load_indr(&reg),
            Operand::R8(reg) => self.get_reg8(&reg),
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
            Operand::IndR16(reg) => self.save_indr(&reg, data),
            Operand::R8(reg) => {
                self.set_reg8(&reg, data);
            }
            _ => unreachable!(),
        };
    }

    fn ld16(&mut self, dest: Operand, source: Operand) {
        let data = match source {
            Operand::D16 => self.read_next_16bit(),
            Operand::R16(reg) => self.get_reg16(&reg),
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
            Operand::R16(reg) => self.set_reg16(&reg, data),
            _ => unreachable!(),
        };
    }

    fn push(&mut self, reg: Reg16) {
        let (high, low) = split_u16(self.get_reg16(&reg));
        self.regs.sp -= 1;
        self.bus.write(self.regs.sp, high);
        self.regs.sp -= 1;
        self.bus.write(self.regs.sp, low);
    }

    fn pop(&mut self, reg: Reg16) {
        let low = self.bus.read(self.regs.sp);
        self.regs.sp += 1;
        let high = self.bus.read(self.regs.sp);
        self.regs.sp += 1;
        self.set_reg16(&reg, combine_to_u16(high, low));
    }

    fn jr(&mut self, cond: Cond) {
        let data = i32::from(self.read_next_8bit() as i8);
        if self.check_cond(cond) {
            let result = self.regs.pc as i32 + data;
            self.regs.pc = result as u16;
        }
    }

    fn jp(&mut self, cond: Cond, dest: Operand) {
        let addr = match dest {
            Operand::A16 => self.read_next_16bit(),
            Operand::R16(reg) => self.get_reg16(&reg),
            _ => unreachable!(),
        };
        if self.check_cond(cond) {
            self.regs.pc = addr;
        }
    }

    fn call(&mut self, cond: Cond) {
        let new_address = self.read_next_16bit();
        if self.check_cond(cond) {
            self.push(Reg16::Pc);
            self.regs.pc = new_address;
        }
    }

    fn ret(&mut self, cond: Cond) {
        if self.check_cond(cond) {
            self.pop(Reg16::Pc);
        }
    }

    fn reti(&mut self) {}

    fn rst(&mut self, amount: u8) {}

    fn add_a(&mut self, operand: Operand, with_carry: bool) {
        let left = self.regs.a as u16;
        let right = self.get_8bit_operand(&operand) as u16;
        let c = if with_carry && self.regs.carry_flag() {
            1
        } else {
            0
        };
        let sum = left + right + c;
        let (carry, result) = split_u16(sum);
        self.regs.a = result;
        self.regs.set_zero(result == 0);
        self.regs.set_subtract(false);
        self.regs
            .set_half_carry((left & 0xF) + (right & 0xF) + c > 0xF);
        self.regs.set_carry(carry > 0);
    }

    fn add_hl(&mut self, reg: Reg16) {
        let data = self.get_reg16(&reg) as u32;
        let hl = self.regs.hl() as u32;
        let sum = hl + data;
        let (carry, result) = split_u32(sum);
        self.regs.set_hl(result);
        self.regs.set_subtract(false);
        self.regs
            .set_half_carry((data & 0xFFF) + (hl & 0xFFF) > 0xFFF);
        self.regs.set_carry(carry > 0);
    }

    fn add_sp(&mut self) {
        let data = self.read_next_8bit() as i16;
        // TODO: What happens when we overflow below 0?
        let result = self.regs.sp as i16 + data;
        self.regs.sp = if result < 0 { 0 } else { result as u16 };
        self.regs.set_zero(false);
        self.regs.set_subtract(false);
        self.regs.set_half_carry(false); // TODO
        self.regs.set_carry(false); // TODO
    }

    fn sub_a(&mut self, operand: Operand, with_carry: bool, save_back: bool) {
        let left = self.regs.a as u16;
        let right = self.get_8bit_operand(&operand) as u16;
        let c: u16 = if with_carry && self.regs.carry_flag() {
            1
        } else {
            0
        };
        let mut diff = left.wrapping_sub(right);
        diff = diff.wrapping_sub(c);

        self.regs.set_zero(diff == 0);
        self.regs.set_subtract(true);
        self.regs.set_half_carry((left & 0xF) < ((right & 0xF) + c));
        self.regs.set_carry(left < right + c);
        if save_back {
            self.regs.a = diff as u8;
        }
    }

    fn and(&mut self, operand: Operand) {
        let data = self.get_8bit_operand(&operand);
        self.regs.a &= data;
        self.regs.set_zero(self.regs.a == 0);
        self.regs.set_subtract(false);
        self.regs.set_half_carry(true);
        self.regs.set_carry(false);
    }

    fn xor(&mut self, operand: Operand) {
        let data = self.get_8bit_operand(&operand);
        self.regs.a ^= data;
        self.regs.set_zero(self.regs.a == 0);
        self.regs.set_subtract(false);
        self.regs.set_half_carry(false);
        self.regs.set_carry(false);
    }

    fn or(&mut self, operand: Operand) {
        let data = self.get_8bit_operand(&operand);
        self.regs.a |= data;
        self.regs.set_zero(self.regs.a == 0);
        self.regs.set_subtract(false);
        self.regs.set_half_carry(false);
        self.regs.set_carry(false);
    }

    fn inc8(&mut self, operand: Operand) {
        let data = self.get_8bit_operand(&operand);
        let result = data.wrapping_add(1);
        self.regs.set_zero(result == 0);
        self.regs.set_subtract(false);
        self.regs.set_half_carry((data & 0x0F) + 1 > 0x0F);
        match operand {
            Operand::IndR16(reg) => {
                let addr = self.get_reg16(&reg);
                self.bus.write(addr, result as u8);
            }
            Operand::R8(reg) => self.set_reg8(&reg, result as u8),
            _ => unreachable!(),
        }
    }

    fn inc16(&mut self, reg: Reg16) {
        let data = self.get_reg16(&reg);
        let sum = data.wrapping_add(1);
        self.set_reg16(&reg, sum);
    }

    fn dec8(&mut self, operand: Operand) {
        let data = self.get_8bit_operand(&operand);
        let result = data.wrapping_sub(1);
        self.regs.set_zero(result == 0);
        self.regs.set_subtract(true);
        self.regs.set_half_carry(data & 0x0F == 0);
        match operand {
            Operand::IndR16(reg) => {
                let addr = self.get_reg16(&reg);
                self.bus.write(addr, result);
            }
            Operand::R8(reg) => self.set_reg8(&reg, result),
            _ => unreachable!(),
        }
    }

    fn dec16(&mut self, reg: Reg16) {
        let data = self.get_reg16(&reg) as u32;
        let sum = data + 1;
        let (carry, result) = split_u32(sum);
        self.set_reg16(&reg, result);
        self.regs.set_zero(result == 0);
        self.regs.set_subtract(true);
        self.regs.set_half_carry(carry & (1 << 11) == 1);
    }

    fn rotate(&mut self, direction: Rotation, operand: Operand) {
        let data = self.get_8bit_operand(&operand);
        let (result, carry) = match direction {
            Rotation::LeftThroughCarry => {
                let carry = data & 0b1000_0000 != 0;
                let mut rotated = data << 1;
                if self.regs.carry_flag() {
                    rotated |= 1;
                }
                (rotated, carry)
            }
            Rotation::LeftCircular => (data.rotate_left(1), data & 0b1000_0000 != 0),
            Rotation::RightThroughCarry => {
                let carry = data & 1 != 0;
                let mut rotated = data >> 1;
                if self.regs.carry_flag() {
                    rotated |= 0b1000_0000;
                }
                (rotated, carry)
            }
            Rotation::RightCircular => (data.rotate_right(1), data & 1 != 0),
        };
        match operand {
            Operand::R8(Reg8::A) => self.regs.set_zero(false),
            _ => self.regs.set_zero(result == 0),
        };
        self.regs.set_subtract(false);
        self.regs.set_half_carry(false);
        self.regs.set_carry(carry);
        self.set_8bit_operand(&operand, result);
    }

    fn shift(&mut self, shift: ShiftType, operand: Operand) {
        let data = self.get_8bit_operand(&operand);
        let (result, carry) = match shift {
            ShiftType::LeftArithmetic => ((data << 1), data & 0b1000_0000 != 0),
            ShiftType::RightArithmetic => ((data >> 1) & 0b1000_0000, data & 1 != 0),
            ShiftType::RightLogic => (data >> 1, (data & 1) != 0),
        };
        self.regs.set_zero(result == 0);
        self.regs.set_subtract(false);
        self.regs.set_half_carry(false);
        self.regs.set_carry(carry);
        self.set_8bit_operand(&operand, result);
    }

    fn swap(&mut self, operand: Operand) {
        let data = self.get_8bit_operand(&operand);
        let result = (data >> 4) | (data << 4);
        self.regs.set_zero(result == 0);
        self.regs.set_subtract(false);
        self.regs.set_half_carry(false);
        self.regs.set_carry(false);
        self.set_8bit_operand(&operand, result);
    }

    fn test_bit(&mut self, amount: u8, operand: Operand) {
        let data = self.get_8bit_operand(&operand);
        let result = data | (1 << amount);
        self.regs.set_zero(result == 0);
        self.regs.set_subtract(false);
        self.regs.set_half_carry(true);
    }

    fn reset_bit(&mut self, amount: u8, operand: Operand) {
        let data = self.get_8bit_operand(&operand);
        let result = data & !(1 << amount);
        self.regs.set_zero(result == 0);
        self.regs.set_subtract(false);
        self.regs.set_half_carry(true);
        self.set_8bit_operand(&operand, result);
    }

    fn set_bit(&mut self, amount: u8, operand: Operand) {
        let data = self.get_8bit_operand(&operand);
        let result = data | (1 << amount);
        self.regs.set_zero(result == 0);
        self.regs.set_subtract(false);
        self.regs.set_half_carry(true);
        self.set_8bit_operand(&operand, result);
    }

    fn daa(&mut self) {
        if self.regs.subtraction_flag() {
            if self.regs.carry_flag() {
                self.regs.a = self.regs.a.wrapping_sub(0x60);
            }
            if self.regs.half_carry_flag() {
                self.regs.a = self.regs.a.wrapping_sub(0x6);
            }
        } else {
            if self.regs.carry_flag() || self.regs.a > 0x99 {
                self.regs.a = self.regs.a.wrapping_add(0x60);
                self.regs.set_carry(true);
            }
            if self.regs.half_carry_flag() || (self.regs.a & 0x0F) > 0x09 {
                self.regs.a = self.regs.a.wrapping_add(0x6);
            }
        }
        self.regs.set_zero(self.regs.a == 0);
        self.regs.set_half_carry(false);
    }

    fn complement_a(&mut self) {
        self.regs.a = !self.regs.a;
        self.regs.set_subtract(true);
        self.regs.set_half_carry(true);
    }

    fn set_carry(&mut self) {
        self.regs.set_carry(true);
    }

    fn complement_carry_flag(&mut self) {
        self.regs.set_subtract(false);
        self.regs.set_half_carry(false);
        self.regs.set_carry(!self.regs.carry_flag())
    }
}
