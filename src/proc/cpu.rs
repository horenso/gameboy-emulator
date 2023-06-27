use super::decode::{decode_prefixed, decode_unprefixed};
use super::dma::Dma;
use super::instruction::*;
use super::interrupts::InterruptHandler;
use super::registers::Registers;
use super::timer::Timer;
use crate::bus::Bus;
use crate::proc::interrupts::Interrupt;
use crate::util::helper::{combine_to_u16, split_u16, split_u32};
use std::io::Write;

#[derive(Debug)]
pub struct Cpu {
    pub(super) regs: Registers,
    pub(crate) interrupt_handler: InterruptHandler,
    pub(crate) timer: Timer,
    pub(crate) dma: Dma,

    pub(crate) is_halted: bool,
    pub(crate) counter: u64, // count number of executed instructions
    pub(crate) cycles: u64,
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu {
            regs: Registers::new(),
            interrupt_handler: InterruptHandler::new(),
            timer: Timer::new(),
            dma: Dma::new(),
            is_halted: false,
            counter: 0,
            cycles: 0,
        }
    }

    pub fn fetch_and_execute(&mut self, bus: &mut Bus) {
        if self.is_halted {
            eprintln!(
                "{}: CPU is halted: {:?} {:?}",
                self.counter, self.timer, self.interrupt_handler
            );
            self.tick_machine_cycles(1, bus);
        } else {
            let inst = self.fetch(bus);
            self.execute(bus, inst);
        };
        self.counter += 1;

        if self.interrupt_handler.is_interrupt_pending() {
            self.is_halted = false;
            let maybe_interrupt = self.interrupt_handler.handle_interrupts();
            if let Some(interrupt) = maybe_interrupt {
                eprintln!("INTERRUPT HAPPENED: {:?}", interrupt);
                let jump_address = interrupt.address();
                self.push_and_set_pc(bus, jump_address); // takes 3 machine cycles
                self.tick_machine_cycles(2, bus);
                self.interrupt_handler.master_enabled = false;
            }
        }
    }

    fn tick_machine_cycles(&mut self, machine_cycles: u8, bus: &mut Bus) {
        for _ in 0..machine_cycles {
            for _ in 0..4 {
                if self.timer.tick() {
                    self.interrupt_handler.request_interrupt(Interrupt::Timer);
                }
                self.cycles += 1;
            }
            self.dma.tick(bus);
        }
    }

    fn read_next_8bit(&mut self, bus: &mut Bus) -> u8 {
        let data = bus.read(Option::Some(self), self.regs.pc);
        self.regs.pc = self.regs.pc.wrapping_add(1);
        self.tick_machine_cycles(1, bus);
        data
    }

    fn read_next_16bit(&mut self, bus: &mut Bus) -> u16 {
        let low = self.read_next_8bit(bus);
        let high = self.read_next_8bit(bus);
        combine_to_u16(high, low)
    }

    fn read_8bit(&mut self, bus: &mut Bus, addr: u16) -> u8 {
        let data = bus.read(Option::Some(self), addr);
        self.tick_machine_cycles(1, bus);
        data
    }

    fn write_8bit(&mut self, bus: &mut Bus, addr: u16, data: u8) {
        bus.write(self, addr, data);
        self.tick_machine_cycles(1, bus);
    }

    fn write_16bit(&mut self, bus: &mut Bus, addr: u16, data: u16) {
        let (high, low) = split_u16(data);
        bus.write(self, addr, low);
        bus.write(self, addr + 1, high);
        self.tick_machine_cycles(1, bus);
    }

    fn fetch(&mut self, bus: &mut Bus) -> Inst {
        let mut fetched = self.read_next_8bit(bus);
        let mut inst = decode_unprefixed(fetched);
        if inst == Inst::Prefix {
            fetched = self.read_next_8bit(bus);
            inst = decode_prefixed(fetched);
            // eprint!("pre ");
        }
        // eprint!("{:#04X} ", fetched);
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

    fn load_indr(&mut self, bus: &mut Bus, reg: &Reg16) -> u8 {
        let addr = self.get_reg16(reg);
        self.read_8bit(bus, addr)
    }

    fn save_indr(&mut self, bus: &mut Bus, reg: &Reg16, data: u8) {
        let addr = self.get_reg16(reg);
        self.write_8bit(bus, addr, data);
    }

    fn get_8bit_operand(&mut self, bus: &mut Bus, operand: &Operand) -> u8 {
        match operand {
            Operand::D8 => self.read_next_8bit(bus),
            Operand::R8(reg) => self.get_reg8(reg),
            Operand::IndR16(reg) => self.load_indr(bus, reg),
            _ => unreachable!(),
        }
    }

    fn set_8bit_operand(&mut self, bus: &mut Bus, operand: &Operand, data: u8) {
        match operand {
            Operand::R8(reg) => self.set_reg8(reg, data),
            Operand::IndR16(reg) => {
                let addr = self.get_reg16(reg);
                self.write_8bit(bus, addr, data);
            }
            _ => unreachable!(),
        }
    }

    fn check_cond(&mut self, cond: &Cond) -> bool {
        match cond {
            Cond::Always => true,
            Cond::Zero => self.regs.zero_flag(),
            Cond::NotZero => !(self.regs.zero_flag()),
            Cond::Carry => self.regs.carry_flag(),
            Cond::NotCarry => !(self.regs.carry_flag()),
        }
    }

    pub fn debug_print(&self, bus: &Bus, file: &mut impl Write) {
        let p0 = bus.read(Option::Some(self), self.regs.pc);
        let p1 = bus.read(Option::Some(self), self.regs.pc.wrapping_add(1));
        let p2 = bus.read(Option::Some(self), self.regs.pc.wrapping_add(2));
        let p3 = bus.read(Option::Some(self), self.regs.pc.wrapping_add(3));
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
        match inst {
            Inst::Prefix => unreachable!(),

            Inst::NoOp => (),
            Inst::Halt => self.halt(),
            Inst::Stop => self.stop(),
            Inst::Di => self.disable_int(),
            Inst::Ei => self.enable_int(),

            Inst::Ld8(dest, source) => self.load8(bus, dest, source),
            Inst::Ld16(dest, source) => self.load16(bus, dest, source),
            Inst::LdHlSp => self.ld_hl_sp_plus_offset(bus),
            Inst::Push(reg) => self.push_stack(bus, reg),
            Inst::Pop(reg) => self.pop_stack(bus, reg),

            Inst::JumpAddr(cond) => self.jump_address(bus, cond),
            Inst::JumpHl => self.jump_hl(),
            Inst::JumpRelative(cond) => self.jump_relative(bus, cond),
            Inst::Call(cond) => self.call(bus, cond),
            Inst::Ret(cond) => self.ret(bus, cond),
            Inst::Reti => self.reti(bus),
            Inst::Rst(offset) => self.rst(bus, offset),

            Inst::Add(operand) => self.add_a(bus, operand, false),
            Inst::AddHl(reg) => self.add_hl(bus, reg),
            Inst::AddSp => self.add_sp(bus),
            Inst::Adc(operand) => self.add_a(bus, operand, true),
            Inst::Sub(operand) => self.sub_a(bus, operand, false, true),
            Inst::Sbc(operand) => self.sub_a(bus, operand, true, true),
            Inst::And(operand) => self.and(bus, operand),
            Inst::Xor(operand) => self.xor(bus, operand),
            Inst::Or(operand) => self.or(bus, operand),
            Inst::Cp(operand) => self.sub_a(bus, operand, false, false),
            Inst::Inc8(operand) => self.inc8(bus, operand),
            Inst::Inc16(reg) => self.inc16(bus, reg),
            Inst::Dec8(operand) => self.dec8(bus, operand),
            Inst::Dec16(reg) => self.dec16(bus, reg),

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

    fn halt(&mut self) {
        eprintln!("HALT!!!");
        self.is_halted = true;
    }

    fn stop(&self) {
        // TODO: What do we do here?
        eprintln!("STOPPPPP");
    }

    fn disable_int(&mut self) {
        // eprintln!("di!");
        self.interrupt_handler.master_enabled = false;
    }

    fn enable_int(&mut self) {
        // eprintln!("ei!");
        self.interrupt_handler.master_enabled = true;
    }

    fn load8(&mut self, bus: &mut Bus, dest: Operand, source: Operand) {
        let data = match source {
            Operand::D8 => self.read_next_8bit(bus),
            Operand::A8 => {
                let addr = combine_to_u16(0xFF, self.read_next_8bit(bus));
                self.read_8bit(bus, addr)
            }
            Operand::A16 => {
                let addr = self.read_next_16bit(bus);
                self.read_8bit(bus, addr)
            }
            Operand::IndHighPlusC => self.read_8bit(bus, combine_to_u16(0xFF, self.regs.c)),
            Operand::IndR16(reg) => self.load_indr(bus, &reg),
            Operand::R8(reg) => self.get_reg8(&reg),
            _ => unreachable!(),
        };
        match dest {
            Operand::A8 => {
                let addr = combine_to_u16(0xFF, self.read_next_8bit(bus));
                self.write_8bit(bus, addr, data);
            }
            Operand::A16 => {
                let addr = self.read_next_16bit(bus);
                self.write_8bit(bus, addr, data);
            }
            Operand::IndHighPlusC => {
                let addr = combine_to_u16(0xFF, self.regs.c);
                self.write_8bit(bus, addr, data);
            }
            Operand::IndR16(reg) => self.save_indr(bus, &reg, data),
            Operand::R8(reg) => {
                self.set_reg8(&reg, data);
            }
            _ => unreachable!(),
        };
    }

    fn load16(&mut self, bus: &mut Bus, dest: Operand, source: Operand) {
        let data = match source {
            Operand::D16 => self.read_next_16bit(bus),
            Operand::R16(reg) => {
                let result = self.get_reg16(&reg);
                self.tick_machine_cycles(1, bus);
                result
            }
            _ => unreachable!(),
        };
        match dest {
            Operand::A8 => {
                let addr = combine_to_u16(0xFF, self.read_next_8bit(bus));
                self.write_16bit(bus, addr, data);
            }
            Operand::A16 => {
                let addr = self.read_next_16bit(bus);
                self.write_16bit(bus, addr, data);
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
            .set_flag_half_carry(((sp ^ u16_offset ^ result) & 0x10) == 0x10);
        self.regs
            .set_flag_carry(((sp ^ u16_offset ^ result) & 0x100) == 0x100);
        self.tick_machine_cycles(1, bus);
    }

    fn push_stack(&mut self, bus: &mut Bus, reg: Reg16) {
        let (high, low) = split_u16(self.get_reg16(&reg));
        self.regs.sp -= 1;
        self.write_8bit(bus, self.regs.sp, high);
        self.regs.sp -= 1;
        self.write_8bit(bus, self.regs.sp, low);
        self.tick_machine_cycles(1, bus);
    }

    fn pop_stack(&mut self, bus: &mut Bus, reg: Reg16) {
        let low = self.read_8bit(bus, self.regs.sp);
        self.regs.sp += 1;
        let high = self.read_8bit(bus, self.regs.sp);
        self.regs.sp += 1;
        self.set_reg16(&reg, combine_to_u16(high, low));
    }

    fn jump_address(&mut self, bus: &mut Bus, cond: Cond) {
        let addr = self.read_next_16bit(bus);
        if self.check_cond(&cond) {
            self.regs.pc = addr;
            self.tick_machine_cycles(1, bus);
        }
    }

    fn jump_hl(&mut self) {
        self.regs.pc = self.regs.hl()
    }

    fn jump_relative(&mut self, bus: &mut Bus, cond: Cond) {
        let data = (self.read_next_8bit(bus) as i8) as i32;
        if self.check_cond(&cond) {
            let result = self.regs.pc as i32 + data;
            self.regs.pc = result as u16;
            self.tick_machine_cycles(1, bus);
        }
    }

    fn push_and_set_pc(&mut self, bus: &mut Bus, address: u16) {
        self.push_stack(bus, Reg16::Pc);
        self.regs.pc = address;
    }

    fn call(&mut self, bus: &mut Bus, cond: Cond) {
        let new_address = self.read_next_16bit(bus);
        if self.check_cond(&cond) {
            self.push_and_set_pc(bus, new_address);
        }
    }

    fn ret(&mut self, bus: &mut Bus, cond: Cond) {
        self.tick_machine_cycles(1, bus);
        if self.check_cond(&cond) {
            self.pop_stack(bus, Reg16::Pc);
            if cond != Cond::Always {
                self.tick_machine_cycles(1, bus);
            }
        }
    }

    fn reti(&mut self, bus: &mut Bus) {
        self.ret(bus, Cond::Always);
        self.interrupt_handler.master_enabled = true;
    }

    fn rst(&mut self, bus: &mut Bus, offset: u8) {
        self.push_and_set_pc(bus, combine_to_u16(0, offset));
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

    fn add_hl(&mut self, bus: &mut Bus, reg: Reg16) {
        let data = self.get_reg16(&reg) as u32;
        let hl = self.regs.hl() as u32;
        let sum = hl + data;
        let (carry, result) = split_u32(sum);
        self.regs.set_hl(result);
        self.regs.set_flag_subtract(false);
        self.regs
            .set_flag_half_carry((data & 0xFFF) + (hl & 0xFFF) > 0xFFF);
        self.regs.set_flag_carry(carry > 0);
        self.tick_machine_cycles(1, bus);
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
            .set_flag_half_carry(((sp ^ u16_data ^ u16_result) & 0x10) == 0x10);
        self.regs
            .set_flag_carry(((sp ^ u16_data ^ u16_result) & 0x100) == 0x100);
        self.tick_machine_cycles(2, bus);
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
                self.write_8bit(bus, addr, result);
            }
            Operand::R8(reg) => self.set_reg8(&reg, result),
            _ => unreachable!(),
        }
    }

    fn inc16(&mut self, bus: &mut Bus, reg: Reg16) {
        let data = self.get_reg16(&reg);
        let sum = data.wrapping_add(1);
        self.set_reg16(&reg, sum);
        self.tick_machine_cycles(1, bus);
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
                self.write_8bit(bus, addr, result);
            }
            Operand::R8(reg) => self.set_reg8(&reg, result),
            _ => unreachable!(),
        }
    }

    fn dec16(&mut self, bus: &mut Bus, reg: Reg16) {
        let data = self.get_reg16(&reg);
        let result = data.wrapping_sub(1);
        self.set_reg16(&reg, result);
        self.tick_machine_cycles(1, bus);
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

    fn test_bit(&mut self, bus: &mut Bus, index: u8, operand: Operand) {
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
