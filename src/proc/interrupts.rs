enum Interrupt {
    VBlank,
    LcdStat,
    Timer,
    Serial,
    Joypad,
}

impl Interrupt {
    pub fn bit(&self) -> u8 {
        match self {
            Self::VBlank => 1,
            Self::LcdStat => 2,
            Self::Timer => 4,
            Self::Serial => 8,
            Self::Joypad => 16,
        }
    }

    pub fn address(&self) -> u16 {
        match self {
            Self::VBlank => 0x40,
            Self::LcdStat => 0x48,
            Self::Timer => 0x50,
            Self::Serial => 0x58,
            Self::Joypad => 0x60,
        }
    }
}

// Interrupts are handled in the following priority
static INTERRUPT_PRIORITY: &[Interrupt] = &[
    Interrupt::VBlank,
    Interrupt::LcdStat,
    Interrupt::Timer,
    Interrupt::Serial,
    Interrupt::Joypad,
];

#[derive(Debug)]
pub struct InterruptHandler {
    pub(crate) master_enabled: bool,
    pub(crate) enabled: u8,
    pub(crate) requested: u8,
}

impl InterruptHandler {
    pub fn new() -> InterruptHandler {
        InterruptHandler {
            master_enabled: false,
            enabled: 0,
            requested: 0,
        }
    }

    pub fn handle_interrupts(&mut self) -> Option<u16> {
        if !self.master_enabled || self.requested & self.enabled == 0 {
            return Option::None;
        }

        for interrupt in INTERRUPT_PRIORITY {
            let result = self.handle_interrupt(interrupt);
            if let Some(..) = result {
                return result;
            }
        }
        Option::None
    }

    fn handle_interrupt(&mut self, interrupt: &Interrupt) -> Option<u16> {
        if self.enabled & interrupt.bit() == 0 {
            return Option::None;
        }
        self.master_enabled = false;
        self.requested &= !interrupt.bit();
        Option::Some(interrupt.address())
    }
}
