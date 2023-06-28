#[derive(Clone, Debug)]
pub enum Interrupt {
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
    enabled: u8,   // IE: interrupt enable
    requested: u8, // IF: interrupt flag
}

impl InterruptHandler {
    pub fn new() -> InterruptHandler {
        InterruptHandler {
            master_enabled: false,
            enabled: 0,
            requested: 0,
        }
    }

    pub fn handle_interrupts(&mut self) -> Option<Interrupt> {
        if !(self.master_enabled && self.is_interrupt_pending()) {
            return Option::None;
        }

        for interrupt in INTERRUPT_PRIORITY {
            let result = self.handle_interrupt(interrupt);
            if result.is_some() {
                return result;
            }
        }
        Option::None
    }

    pub fn is_interrupt_pending(&self) -> bool {
        self.requested & self.enabled != 0
    }

    fn handle_interrupt(&mut self, interrupt: &Interrupt) -> Option<Interrupt> {
        if self.enabled & interrupt.bit() == 0 {
            return Option::None;
        }
        self.master_enabled = false;
        self.requested &= !interrupt.bit();
        Option::Some(interrupt.clone())
    }

    pub fn enabled(&self) -> u8 {
        self.enabled
    }

    pub fn set_enabled(&mut self, data: u8) {
        self.enabled = data & 0b0001_1111;
    }

    pub fn requested(&self) -> u8 {
        self.requested
    }

    pub fn set_requested(&mut self, data: u8) {
        self.requested = data & 0b0001_1111;
    }

    pub fn request_interrupt(&mut self, interrupt: Interrupt) {
        self.requested |= interrupt.bit();
    }
}
