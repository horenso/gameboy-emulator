use crate::helper::{combine_to_u16, split_u16};

#[derive(Default)]
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
            pc: 0x0100, // The entrypoint after the Nintendo internal ROM
            sp: 0xFFFE,
        }
    }

    pub fn set_af(&mut self, value: u16) {
        (self.a, self.f) = split_u16(value);
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
        self.set_hl(self.hl() + 1);
    }

    pub fn decr_hl(&mut self) {
        self.set_hl(self.hl() - 1);
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
        assert_eq!(regs.f, 0xCD);

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
