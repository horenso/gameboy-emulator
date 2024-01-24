use std::{fs::File, io::Read};

#[derive(Debug)]
pub enum MemoryBankController {
    RomOnly,
    MBC1,
}

#[derive(Debug)]
pub enum BankingMode {
    Rom,
    Ram,
}

#[derive(Debug)]
pub struct Cartridge {
    pub data: Vec<u8>,
    pub title: String,
    pub mbc: MemoryBankController,
    pub rom_banks: u8, // number of 32 KiB ROM banks
    pub ram_banks: u8, // number of 8 KiB RAM banks

    // MBC1 related, this may be refactored to it's own struct
    is_ram_enabled: bool,
    banking_mode: BankingMode,
}

fn read_string(data: &Vec<u8>, start_index: usize, length: usize) -> String {
    let mut string = String::new();
    for i in start_index..start_index + length {
        let byte = *data.get(i).unwrap();
        if byte == 0 {
            break;
        }
        string.push(byte as char);
    }
    string
}

impl Cartridge {
    pub fn load_from_file(cartridge_path: &str) -> Result<Cartridge, String> {
        let mut file = File::open(cartridge_path).map_err(|e| e.to_string())?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).map_err(|e| e.to_string())?;

        let title = read_string(&buffer, 0x0134, 16);
        let mbc = match *buffer.get(0x0147).unwrap() {
            0 => MemoryBankController::RomOnly,
            _ => MemoryBankController::MBC1,
        };
        let rom_banks = 1 << *buffer.get(0x0148).unwrap();
        let ram_banks = match *buffer.get(0x0149).unwrap() {
            0 | 1 => 0,
            2 => 1,
            3 => 4,
            4 => 16,
            5 => 8,
            _ => 0,
        };
        eprintln!(
            "Cartridge info: {} {:?} {} {}",
            title, mbc, rom_banks, ram_banks
        );
        Ok(Cartridge {
            data: buffer,
            title,
            mbc,
            rom_banks,
            ram_banks,
        })
    }

    pub fn read(&self, index: usize) -> u8 {
        if index < self.data.len() {
            self.data[index]
        } else {
            eprintln!("Reading outside of cartridge at {}!", index);
            0
        }
    }

    pub fn write(&mut self, address: u16, data: u8) {
        eprintln!("Writting to cartridge at {:x} {:x}", address, data);
    }

    pub fn print_info(&self) {
        eprintln!("Title: {}", self.title);
        // eprintln!("Cartridge type: {:02X}", self.cartridge_type);
        eprintln!("32 KiB ROM banks: {:02X}", self.rom_banks);
        eprintln!("8 KiB RAM banks: {:02X}", self.ram_banks);
    }
}
