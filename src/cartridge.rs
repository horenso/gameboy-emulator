use std::{fs::File, io::Read};

pub struct Cartridge {
    pub data: Vec<u8>,
}

impl Cartridge {
    pub fn load_from_file(cartridge_path: &str) -> Result<Cartridge, String> {
        let mut file = File::open(cartridge_path).map_err(|e| e.to_string())?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).map_err(|e| e.to_string())?;
        Ok(Cartridge { data: buffer })
    }

    pub fn read(&self, index: usize) -> u8 {
        if index < self.data.len() {
            self.data[index]
        } else {
            println!("Reading outside of cartridge at {}!", index);
            0
        }
    }
}
