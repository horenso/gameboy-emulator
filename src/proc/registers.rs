use crate::util::helper::{combine_to_u16, split_u16};

pub enum Flag {
    Zero = 0b1000_0000,
    Subtraction = 0b0100_0000,
    HalfCarry = 0b0010_0000,
    Carry = 0b0001_0000,
}

#[derive(Default, Debug)]
pub struct Registers {
    pub a: u8,
    pub f: u8,

    pub b: u8,
    pub c: u8,

    pub d: u8,
    pub e: u8,

    pub h: u8,
    pub l: u8,

    pub sp: u16,
    pub pc: u16,
}

impl Registers {
    pub fn new() -> Self {
        Registers {
            a: 0x0001,
            f: 0x00B0,
            b: 0x0000,
            c: 0x0013,
            d: 0x0000,
            e: 0x00D8,
            h: 0x0001,
            l: 0x004D,
            pc: 0x00FF, // The entrypoint after the Nintendo internal ROM
            sp: 0xFFFE,
        }
    }

    pub fn set_f(&mut self, value: u8) {
        // Only the upper four bits (the flags) can be set
        self.f = value & 0xF0;
    }

    pub fn set_af(&mut self, value: u16) {
        let (a, f) = split_u16(value);
        self.a = a;
        self.set_f(f);
    }

    pub fn set_bc(&mut self, value: u16) {
        (self.b, self.c) = split_u16(value)
    }

    pub fn set_de(&mut self, value: u16) {
        (self.d, self.e) = split_u16(value)
    }

    pub fn set_hl(&mut self, value: u16) {
        (self.h, self.l) = split_u16(value)
    }

    pub fn af(&self) -> u16 {
        combine_to_u16(self.a, self.f)
    }

    pub fn bc(&self) -> u16 {
        combine_to_u16(self.b, self.c)
    }

    pub fn de(&self) -> u16 {
        combine_to_u16(self.d, self.e)
    }

    pub fn hl(&self) -> u16 {
        combine_to_u16(self.h, self.l)
    }

    pub fn incr_hl(&mut self) {
        // TODO: Do this more efficiently maybe
        self.set_hl(self.hl().wrapping_add(1));
    }

    pub fn decr_hl(&mut self) {
        self.set_hl(self.hl().wrapping_sub(1));
    }

    pub fn zero_flag(&self) -> bool {
        self.f & Flag::Zero as u8 != 0
    }

    pub fn subtraction_flag(&self) -> bool {
        self.f & Flag::Subtraction as u8 != 0
    }

    pub fn half_carry_flag(&self) -> bool {
        self.f & Flag::HalfCarry as u8 != 0
    }

    pub fn carry_flag(&self) -> bool {
        self.f & Flag::Carry as u8 != 0
    }

    pub fn set_flag_zero(&mut self, zero: bool) {
        if zero {
            self.f |= Flag::Zero as u8;
        } else {
            self.f &= !(Flag::Zero as u8);
        }
    }

    pub fn set_flag_subtract(&mut self, subtract: bool) {
        if subtract {
            self.f |= Flag::Subtraction as u8;
        } else {
            self.f &= !(Flag::Subtraction as u8);
        }
    }

    pub fn set_flag_half_carry(&mut self, half_carry: bool) {
        if half_carry {
            self.f |= Flag::HalfCarry as u8;
        } else {
            self.f &= !(Flag::HalfCarry as u8);
        }
    }

    pub fn set_flag_carry(&mut self, carry: bool) {
        if carry {
            self.f |= Flag::Carry as u8;
        } else {
            self.f &= !(Flag::Carry as u8);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_accessors() {
        let mut regs = Registers::new();

        // AF
        regs.a = 0xEE;
        regs.f = 0x23;
        assert_eq!(regs.af(), 0xEE23);

        regs.set_af(0xABCD);
        assert_eq!(regs.a, 0xAB);
        // The lower 4 bits of F are always unset
        assert_eq!(regs.f, 0xC0);

        // BC
        regs.b = 0xBB;
        regs.c = 0xCC;
        assert_eq!(regs.bc(), 0xBBCC);

        regs.set_bc(0x1234);
        assert_eq!(regs.b, 0x12);
        assert_eq!(regs.c, 0x34);

        // DE
        regs.d = 0x34;
        regs.e = 0x54;
        assert_eq!(regs.de(), 0x3454);

        regs.set_de(0x6543);
        assert_eq!(regs.d, 0x65);
        assert_eq!(regs.e, 0x43);

        // HL
        regs.h = 0xAC;
        regs.l = 0x1F;
        assert_eq!(regs.hl(), 0xAC1F);

        regs.set_hl(0xFE12);
        assert_eq!(regs.h, 0xFE);
        assert_eq!(regs.l, 0x12);
    }
}
