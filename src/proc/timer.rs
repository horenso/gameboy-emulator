#[derive(Debug)]
pub struct Timer {
    modulo: u8,
    control: u8,
    count: u8,
    divider: u8,
}

impl Timer {
    pub fn new() -> Timer {
        Timer {
            modulo: 0,
            control: 0,
            count: 0,
            divider: 0,
        }
    }

    pub fn devider(&self) -> u8 {
        self.divider
    }

    pub fn reset_devider(&mut self) {
        self.divider = 0;
    }
}
