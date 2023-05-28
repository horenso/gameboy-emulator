use std::io::Write;

use crate::bus::Bus;
use crate::decode::*;
use crate::helper::{combine_to_u16, split_u16, split_u32};
use crate::instruction::*;
use crate::registers::Registers;

pub struct Cpu {
    regs: Registers,
    interrupt_master_enabled: bool,
    interrupt_flags: u8,
    pub counter: u64, // count number of executed instructions
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu {
            regs: Registers::new(),
            interrupt_master_enabled: false,
            interrupt_flags: 0,
            counter: 0,
        }
    }

    pub fn fetch_and_execute(&mut self, bus: &mut Bus) {
        let inst = self.fetch(bus);
        self.execute(bus, inst);
    }

    fn read_next_8bit(&mut self, bus: &Bus) -> u8 {
        let data = bus.read(self.regs.pc);
        self.regs.pc = self.regs.pc.wrapping_add(1);
        data
    }

    fn read_next_16bit(&mut self, bus: &Bus) -> u16 {
        let low = self.read_next_8bit(bus);
        let high = self.read_next_8bit(bus);
        combine_to_u16(high, low)
    }

    fn fetch(&mut self, bus: &mut Bus) -> Inst {
        let fetched = self.read_next_8bit(bus);
        let mut inst = decode_unprefixed(fetched);
        if inst == Inst::Prefix {
            let fetched = self.read_next_8bit(bus);
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
            Reg16::HlIncr | Reg16::HlDecr => unreachable!(),
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
        }
    }

    fn load_indr(&mut self, bus: &Bus, reg: &Reg16) -> u8 {
        let addr = self.get_reg16(&reg);
        bus.read(addr)
    }

    fn save_indr(&mut self, bus: &mut Bus, reg: &Reg16, data: u8) {
        let addr = self.get_reg16(&reg);
        bus.write(addr, data);
    }

    fn get_8bit_operand(&mut self, bus: &Bus, operand: &Operand) -> u8 {
        match operand {
            Operand::D8 => self.read_next_8bit(bus),
            Operand::R8(reg) => self.get_reg8(&reg),
            Operand::IndR16(reg) => self.load_indr(bus, reg),
            _ => unreachable!(),
        }
    }

    fn set_8bit_operand(&mut self, bus: &mut Bus, operand: &Operand, data: u8) {
        match operand {
            Operand::R8(reg) => self.set_reg8(reg, data),
            Operand::IndR16(reg) => {
                let addr = self.get_reg16(reg);
                bus.write(addr, data);
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

    pub fn debug_print(&self, bus: &Bus, file: &mut impl Write) {
        let p0 = bus.read(self.regs.pc);
        let p1 = bus.read(self.regs.pc.wrapping_add(1));
        let p2 = bus.read(self.regs.pc.wrapping_add(2));
        let p3 = bus.read(self.regs.pc.wrapping_add(3));
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

    pub fn execute(&mut self, bus: &mut Bus, inst: Inst) {
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

            Inst::Ld8(dest, source) => self.ld8(bus, dest, source),
            Inst::Ld16(dest, source) => self.ld16(bus, dest, source),
            Inst::LdHlSp => self.ld_hl_sp_plus_offset(bus),
            Inst::Push(reg) => self.push(bus, reg),
            Inst::Pop(reg) => self.pop(bus, reg),

            Inst::Jr(cond) => self.jr(bus, cond),
            Inst::Jp(cond, dest) => self.jp(bus, cond, dest),
            Inst::Call(cond) => self.call(bus, cond),
            Inst::Ret(cond) => self.ret(bus, cond),
            Inst::Reti => self.reti(bus),
            Inst::Rst(offset) => self.rst(bus, offset),

            Inst::Add(operand) => self.add_a(bus, operand, false),
            Inst::AddHl(reg) => self.add_hl(reg),
            Inst::AddSp => self.add_sp(bus),
            Inst::Adc(operand) => self.add_a(bus, operand, true),
            Inst::Sub(operand) => self.sub_a(bus, operand, false, true),
            Inst::Sbc(operand) => self.sub_a(bus, operand, true, true),
            Inst::And(operand) => self.and(bus, operand),
            Inst::Xor(operand) => self.xor(bus, operand),
            Inst::Or(operand) => self.or(bus, operand),
            Inst::Cp(operand) => self.sub_a(bus, operand, false, false),
            Inst::Inc8(operand) => self.inc8(bus, operand),
            Inst::Inc16(reg) => self.inc16(reg),
            Inst::Dec8(operand) => self.dec8(bus, operand),
            Inst::Dec16(reg) => self.dec16(reg),

            Inst::Rotate(rot, operand, set_zero) => self.rotate(bus, rot, operand, set_zero),
            Inst::Shift(shift, operand) => self.shift(bus, shift, operand),
            Inst::Swap(operand) => self.swap(bus, operand),
            Inst::TestBit(index, operand) => self.test_bit(bus, index, operand),
            Inst::ResetBit(index, operand) => self.reset_bit(bus, index, operand),
            Inst::SetBit(index, operand) => self.set_bit(bus, index, operand),

            Inst::DecimalAdjustA => self.decimal_adjust_a(),
            Inst::ComplementA => self.complement_a(),
            Inst::SetCarryFlag => self.set_flag_carry(),
            Inst::ComplementCarryFlag => self.complement_carry_flag(),
        };
    }

    fn halt(&self) {}

    fn stop(&self) {
        // println!("STOPPPPP");
    }

    fn di(&self) {
        // println!("diiiii");
    }

    fn ei(&self) {
        // println!("eiiiii");
    }

    fn ld8(&mut self, bus: &mut Bus, dest: Operand, source: Operand) {
        let data = match source {
            Operand::D8 => self.read_next_8bit(bus),
            Operand::A8 => {
                let addr = combine_to_u16(0xFF, self.read_next_8bit(bus));
                bus.read(addr)
            }
            Operand::A16 => {
                let addr = self.read_next_16bit(bus);
                bus.read(addr)
            }
            Operand::IndHighPlusC => bus.read(combine_to_u16(0xFF, self.regs.c)),
            Operand::IndR16(reg) => self.load_indr(bus, &reg),
            Operand::R8(reg) => self.get_reg8(&reg),
            _ => unreachable!(),
        };
        match dest {
            Operand::A8 => {
                let addr = combine_to_u16(0xFF, self.read_next_8bit(bus));
                bus.write(addr, data);
            }
            Operand::A16 => {
                let addr = self.read_next_16bit(bus);
                bus.write(addr, data);
            }
            Operand::IndHighPlusC => {
                let addr = combine_to_u16(0xFF, self.regs.c);
                bus.write(addr, data)
            }
            Operand::IndR16(reg) => self.save_indr(bus, &reg, data),
            Operand::R8(reg) => {
                self.set_reg8(&reg, data);
            }
            _ => unreachable!(),
        };
    }

    fn ld16(&mut self, bus: &mut Bus, dest: Operand, source: Operand) {
        let data = match source {
            Operand::D16 => self.read_next_16bit(bus),
            Operand::R16(reg) => self.get_reg16(&reg),
            _ => unreachable!(),
        };
        match dest {
            Operand::A8 => {
                let addr = combine_to_u16(0xFF, self.read_next_8bit(bus));
                let (high, low) = split_u16(data);
                bus.write(addr, low);
                bus.write(addr + 1, high);
            }
            Operand::A16 => {
                let addr = self.read_next_16bit(bus);
                let (high, low) = split_u16(data);
                bus.write(addr, low);
                bus.write(addr + 1, high);
            }
            Operand::R16(reg) => self.set_reg16(&reg, data),
            _ => unreachable!(),
        };
    }

    fn ld_hl_sp_plus_offset(&mut self, bus: &mut Bus) {
        let i8_offset = self.read_next_8bit(bus) as i8;
        let i32_offset = i8_offset as i32;
        let u16_offset = i8_offset as u16;
        let sp = self.regs.sp;
        let result = (sp as i32 + i32_offset) as u16;
        self.regs.set_hl(result);

        self.regs.set_flag_zero(false);
        self.regs.set_flag_subtract(false);
        self.regs
            .set_flag_half_carry(((sp ^ u16_offset ^ (result & 0xFFFF)) & 0x10) == 0x10);
        self.regs
            .set_flag_carry(((sp ^ u16_offset ^ (result & 0xFFFF)) & 0x100) == 0x100);
    }

    fn push(&mut self, bus: &mut Bus, reg: Reg16) {
        let (high, low) = split_u16(self.get_reg16(&reg));
        self.regs.sp -= 1;
        bus.write(self.regs.sp, high);
        self.regs.sp -= 1;
        bus.write(self.regs.sp, low);
    }

    fn pop(&mut self, bus: &mut Bus, reg: Reg16) {
        let low = bus.read(self.regs.sp);
        self.regs.sp += 1;
        let high = bus.read(self.regs.sp);
        self.regs.sp += 1;
        self.set_reg16(&reg, combine_to_u16(high, low));
    }

    fn jr(&mut self, bus: &Bus, cond: Cond) {
        let data = i32::from(self.read_next_8bit(bus) as i8);
        if self.check_cond(cond) {
            let result = self.regs.pc as i32 + data;
            self.regs.pc = result as u16;
        }
    }

    fn jp(&mut self, bus: &Bus, cond: Cond, dest: Operand) {
        let addr = match dest {
            Operand::A16 => self.read_next_16bit(bus),
            Operand::R16(reg) => self.get_reg16(&reg),
            _ => unreachable!(),
        };
        if self.check_cond(cond) {
            self.regs.pc = addr;
        }
    }

    fn call(&mut self, bus: &mut Bus, cond: Cond) {
        let new_address = self.read_next_16bit(bus);
        if self.check_cond(cond) {
            self.push(bus, Reg16::Pc);
            self.regs.pc = new_address;
        }
    }

    fn ret(&mut self, bus: &mut Bus, cond: Cond) {
        if self.check_cond(cond) {
            self.pop(bus, Reg16::Pc);
        }
    }

    fn reti(&mut self, bus: &mut Bus) {
        // TODO: Enable interrupts here
        self.ret(bus, Cond::Always);
    }

    fn rst(&mut self, bus: &mut Bus, offset: u8) {
        self.push(bus, Reg16::Pc);
        self.regs.pc = combine_to_u16(0, offset);
    }

    fn add_a(&mut self, bus: &mut Bus, operand: Operand, with_carry: bool) {
        let left = self.regs.a as u16;
        let right = self.get_8bit_operand(bus, &operand) as u16;
        let c = if with_carry && self.regs.carry_flag() {
            1
        } else {
            0
        };
        let sum = left + right + c;
        let (carry, result) = split_u16(sum);
        self.regs.a = result;
        self.regs.set_flag_zero(result == 0);
        self.regs.set_flag_subtract(false);
        self.regs
            .set_flag_half_carry((left & 0xF) + (right & 0xF) + c > 0xF);
        self.regs.set_flag_carry(carry > 0);
    }

    fn add_hl(&mut self, reg: Reg16) {
        let data = self.get_reg16(&reg) as u32;
        let hl = self.regs.hl() as u32;
        let sum = hl + data;
        let (carry, result) = split_u32(sum);
        self.regs.set_hl(result);
        self.regs.set_flag_subtract(false);
        self.regs
            .set_flag_half_carry((data & 0xFFF) + (hl & 0xFFF) > 0xFFF);
        self.regs.set_flag_carry(carry > 0);
    }

    fn add_sp(&mut self, bus: &mut Bus) {
        let data = self.read_next_8bit(bus) as i8;
        let data = data as i32;
        let u16_data = data as u16;
        let sp = self.regs.sp;
        let result = (sp as i32).wrapping_add(data);
        let u16_result = result as u16;
        self.regs.sp = u16_result;
        self.regs.set_flag_zero(false);
        self.regs.set_flag_subtract(false);
        self.regs
            .set_flag_half_carry(((sp ^ u16_data ^ (u16_result & 0xFFFF)) & 0x10) == 0x10);
        self.regs
            .set_flag_carry(((sp ^ u16_data ^ (u16_result & 0xFFFF)) & 0x100) == 0x100);
    }

    fn sub_a(&mut self, bus: &mut Bus, operand: Operand, with_carry: bool, save_back: bool) {
        let left = self.regs.a as u16;
        let right = self.get_8bit_operand(bus, &operand) as u16;
        let c: u16 = if with_carry && self.regs.carry_flag() {
            1
        } else {
            0
        };
        let mut diff = left.wrapping_sub(right);
        diff = diff.wrapping_sub(c);
        let result = diff as u8;

        self.regs.set_flag_zero(result == 0);
        self.regs.set_flag_subtract(true);
        self.regs
            .set_flag_half_carry((left & 0xF) < ((right & 0xF) + c));
        self.regs.set_flag_carry(left < right + c);
        if save_back {
            self.regs.a = result;
        }
    }

    fn and(&mut self, bus: &mut Bus, operand: Operand) {
        let data = self.get_8bit_operand(bus, &operand);
        self.regs.a &= data;
        self.regs.set_flag_zero(self.regs.a == 0);
        self.regs.set_flag_subtract(false);
        self.regs.set_flag_half_carry(true);
        self.regs.set_flag_carry(false);
    }

    fn xor(&mut self, bus: &mut Bus, operand: Operand) {
        let data = self.get_8bit_operand(bus, &operand);
        self.regs.a ^= data;
        self.regs.set_flag_zero(self.regs.a == 0);
        self.regs.set_flag_subtract(false);
        self.regs.set_flag_half_carry(false);
        self.regs.set_flag_carry(false);
    }

    fn or(&mut self, bus: &mut Bus, operand: Operand) {
        let data = self.get_8bit_operand(bus, &operand);
        self.regs.a |= data;
        self.regs.set_flag_zero(self.regs.a == 0);
        self.regs.set_flag_subtract(false);
        self.regs.set_flag_half_carry(false);
        self.regs.set_flag_carry(false);
    }

    fn inc8(&mut self, bus: &mut Bus, operand: Operand) {
        let data = self.get_8bit_operand(bus, &operand);
        let result = data.wrapping_add(1);
        self.regs.set_flag_zero(result == 0);
        self.regs.set_flag_subtract(false);
        self.regs.set_flag_half_carry((data & 0x0F) + 1 > 0x0F);
        match operand {
            Operand::IndR16(reg) => {
                let addr = self.get_reg16(&reg);
                bus.write(addr, result as u8);
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

    fn dec8(&mut self, bus: &mut Bus, operand: Operand) {
        let data = self.get_8bit_operand(bus, &operand);
        let result = data.wrapping_sub(1);
        self.regs.set_flag_zero(result == 0);
        self.regs.set_flag_subtract(true);
        self.regs.set_flag_half_carry(data & 0x0F == 0);
        match operand {
            Operand::IndR16(reg) => {
                let addr = self.get_reg16(&reg);
                bus.write(addr, result);
            }
            Operand::R8(reg) => self.set_reg8(&reg, result),
            _ => unreachable!(),
        }
    }

    fn dec16(&mut self, reg: Reg16) {
        let data = self.get_reg16(&reg);
        let result = data.wrapping_sub(1);
        self.set_reg16(&reg, result);
    }

    fn rotate(&mut self, bus: &mut Bus, direction: Rotation, operand: Operand, set_zero: bool) {
        let data = self.get_8bit_operand(bus, &operand);
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
        self.regs.set_flag_zero(set_zero && result == 0);
        self.regs.set_flag_subtract(false);
        self.regs.set_flag_half_carry(false);
        self.regs.set_flag_carry(carry);
        self.set_8bit_operand(bus, &operand, result);
    }

    fn shift(&mut self, bus: &mut Bus, shift: ShiftType, operand: Operand) {
        let data = self.get_8bit_operand(bus, &operand);
        let (result, carry) = match shift {
            ShiftType::LeftArithmetic => ((data << 1), data & 0b1000_0000 != 0),
            ShiftType::RightArithmetic => ((data >> 1) | (data & 0b1000_0000), data & 1 != 0),
            ShiftType::RightLogic => (data >> 1, data & 1 != 0),
        };
        self.regs.set_flag_zero(result == 0);
        self.regs.set_flag_subtract(false);
        self.regs.set_flag_half_carry(false);
        self.regs.set_flag_carry(carry);
        self.set_8bit_operand(bus, &operand, result);
    }

    fn swap(&mut self, bus: &mut Bus, operand: Operand) {
        let data = self.get_8bit_operand(bus, &operand);
        let result = (data >> 4) | (data << 4);
        self.regs.set_flag_zero(result == 0);
        self.regs.set_flag_subtract(false);
        self.regs.set_flag_half_carry(false);
        self.regs.set_flag_carry(false);
        self.set_8bit_operand(bus, &operand, result);
    }

    fn test_bit(&mut self, bus: &Bus, index: u8, operand: Operand) {
        let data = self.get_8bit_operand(bus, &operand);
        let result = data & (1 << index);
        self.regs.set_flag_zero(result == 0);
        self.regs.set_flag_subtract(false);
        self.regs.set_flag_half_carry(true);
    }

    fn reset_bit(&mut self, bus: &mut Bus, index: u8, operand: Operand) {
        let data = self.get_8bit_operand(bus, &operand);
        let result = data & !(1 << index);
        self.set_8bit_operand(bus, &operand, result);
    }

    fn set_bit(&mut self, bus: &mut Bus, index: u8, operand: Operand) {
        let data = self.get_8bit_operand(bus, &operand);
        let result = data | (1 << index);
        self.set_8bit_operand(bus, &operand, result);
    }

    fn decimal_adjust_a(&mut self) {
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
                self.regs.set_flag_carry(true);
            }
            if self.regs.half_carry_flag() || (self.regs.a & 0x0F) > 0x09 {
                self.regs.a = self.regs.a.wrapping_add(0x6);
            }
        }
        self.regs.set_flag_zero(self.regs.a == 0);
        self.regs.set_flag_half_carry(false);
    }

    fn complement_a(&mut self) {
        self.regs.a = !self.regs.a;
        self.regs.set_flag_subtract(true);
        self.regs.set_flag_half_carry(true);
    }

    fn set_flag_carry(&mut self) {
        self.regs.set_flag_subtract(false);
        self.regs.set_flag_half_carry(false);
        self.regs.set_flag_carry(true);
    }

    fn complement_carry_flag(&mut self) {
        self.regs.set_flag_subtract(false);
        self.regs.set_flag_half_carry(false);
        self.regs.set_flag_carry(!self.regs.carry_flag())
    }
}
