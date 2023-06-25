#[derive(Debug)]
pub struct Timer {
    divider: u16, // DIV: divider register
    counter: u8,  // TIMA: timer counter
    modulo: u8,   // TMA: timer modulo
    control: u8,  // TAC: timer control

    is_enabled: bool,
    mask: u16,
}

impl Timer {
    pub fn new() -> Timer {
        Timer {
            // Appearenly this is the divider value after the boot rom (PC=0x0100)
            // according to the Cycle Accurate Game Boy Docs
            divider: 0,
            counter: 0,
            modulo: 0,
            control: 5, // enabled and speed 1

            is_enabled: true,
            // To mask off the part of the divider that we don't care about
            // e.g if the speed is 0 we care about every 1024th cycle
            // therefore only about the uppermost 0xFC00 bits.
            mask: 0xFC00,
        }
    }

    pub fn tick_timer(&mut self, cycles: u8) -> bool {
        let prev_divider = self.divider;
        self.divider = self.divider.wrapping_add(cycles as u16);
        if !self.is_enabled {
            return false;
        }

        let update_timer = (self.divider & self.mask) != (prev_divider & self.mask);

        if update_timer {
            if self.counter == 0xFF {
                self.counter = self.modulo;
                return true;
            } else {
                self.counter += 1;
            }
        }
        false
    }

    pub fn divider(&self) -> u8 {
        (self.divider >> 8) as u8
    }

    pub fn reset_divider(&mut self) {
        self.divider = 0;
    }

    pub fn counter(&self) -> u8 {
        self.counter
    }

    pub fn set_counter(&mut self, data: u8) {
        self.counter = data;
    }

    pub fn modulo(&self) -> u8 {
        self.modulo
    }

    pub fn set_modulo(&mut self, data: u8) {
        self.modulo = data;
    }

    pub fn control(&self) -> u8 {
        self.control
    }

    pub fn set_control(&mut self, data: u8) {
        self.control = data;
        // control is only a 3-bit register
        // self.control = data & 0b111;
        self.is_enabled = data & 0b100 != 0;
        self.mask = match data & 0b11 {
            0 => 0xFC00, // >= 1024
            1 => 0xFFF0, // >= 16
            2 => 0xFFC0, // >= 64
            3 => 0xFF00, // >= 256
            _ => unreachable!(),
        };
    }
}
