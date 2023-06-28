const OAM_SIZE: usize = 160;

#[derive(Debug)]
pub struct Oam {
    data: [u8; OAM_SIZE], // object attribute memory
}

impl Oam {
    pub fn new() -> Self {
        Oam {
            data: [0; OAM_SIZE],
        }
    }

    pub fn read(&self, offset: u8) -> u8 {
        self.data[offset as usize]
    }

    pub fn write(&mut self, offset: u8, data: u8) {
        self.data[offset as usize] = data
    }
}
