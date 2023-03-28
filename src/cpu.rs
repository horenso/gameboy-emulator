pub struct Cpu {
    pub Af: u16, // Accumulator / Flags
    pub Be: u16,
    pub Hl: u16,

    pub Sp: u16,
    pub Pc: u16,
}

impl Cpu {
    pub fn new() -> Self {
        Cpu {
            AF = 0,
            BE = 0,
            HL = 0,
            SP = 0,
            PC = 0x0100 // The entrypoint after the Nintendo logo
        }
    }

    fn setA(self: Cpu, value: u8) {
        u16_value = self.Af = self.Af | value; 
    }

    pub fn fetch(self: Cpu, bus: &Bus) {

    }
}
