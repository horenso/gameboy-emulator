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
}
