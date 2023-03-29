use std::{fs::File, io::Read};

pub struct Cartridge {
    pub data: Vec<u8>,
}

impl Cartridge {
    pub fn load_from_file(cartridge_path: String) -> Result<Cartridge, String> {
        let mut file = File::open(cartridge_path).map_err(|e| e.to_string())?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).map_err(|e| e.to_string())?;
        Ok(Cartridge { data: buffer })
    }
}
