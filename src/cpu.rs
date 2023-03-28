#[derive(Debug, Default)]
pub struct Cpu {
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

fn split_u16(value: u16) -> (u8, u8) {
    let high = (value >> 8) as u8;
    let low = (value & 0xFF) as u8;
    (high, low)
}

fn combine_to_u16(high: u8, low: u8) -> u16 {
    (high as u16) << 8 | low as u16
}

impl Cpu {
    pub fn new() -> Self {
        Cpu {
            pc: 0x0100, // The entrypoint after the Nintendo internal ROM
            ..Default::default()
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cpu_register_accessors() {
        let mut cpu = Cpu::new();

        // AF
        cpu.a = 0xEE;
        cpu.f = 0x23;
        assert_eq!(cpu.af(), 0xEE23);

        cpu.set_af(0xABCD);
        assert_eq!(cpu.a, 0xAB);
        assert_eq!(cpu.f, 0xCD);

        // BC
        cpu.b = 0xBB;
        cpu.c = 0xCC;
        assert_eq!(cpu.bc(), 0xBBCC);

        cpu.set_bc(0x1234);
        assert_eq!(cpu.b, 0x12);
        assert_eq!(cpu.c, 0x34);

        // DE
        cpu.d = 0x34;
        cpu.e = 0x54;
        assert_eq!(cpu.de(), 0x3454);

        cpu.set_de(0x6543);
        assert_eq!(cpu.d, 0x65);
        assert_eq!(cpu.e, 0x43);

        // HL
        cpu.h = 0xAC;
        cpu.l = 0x1F;
        assert_eq!(cpu.hl(), 0xAC1F);

        cpu.set_hl(0xFE12);
        assert_eq!(cpu.h, 0xFE);
        assert_eq!(cpu.l, 0x12);
    }
}
