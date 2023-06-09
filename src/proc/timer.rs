#[derive(Debug)]
pub struct Timer {
    divider: u16,
    counter: u8,
    modulo: u8,
    control: u8,
}

impl Timer {
    pub fn new() -> Timer {
        Timer {
            divider: 0,
            counter: 0,
            modulo: 0,
            control: 0,
        }
    }

    pub fn tick(&mut self, cycles: u8) {
        self.divider = self.divider.wrapping_add(1);
        eprintln!("Divider {}", self.divider);
        if self.divider == 0 {
            eprintln!("Divider overflowed!");
        }
    }

    pub fn devider(&self) -> u8 {
        self.divider as u8
    }

    pub fn reset_devider(&mut self) {
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
    }
}
