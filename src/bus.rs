use crate::cartridge::Cartridge;

pub struct Bus {
    rom: Cartridge,
    v_ram: [u8; 8192],
    ram: [u8; 8192],
}

impl Bus {
    pub fn new(rom: Cartridge) -> Self {
        Bus {
            rom,
            v_ram: [0; 8192],
            ram: [0; 8192],
        }
    }

    // https://gbdev.io/pandocs/Memory_Map.html
    pub fn read(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x8000 => self.read_from_rom(address),
            0x8001..=0x9fff => {
                let v_ram_address = (address - 0x8001) as usize;
                self.v_ram[v_ram_address]
            }
            _ => {
                println!("Access to {} not implemented", address);
                panic!();
            }
        }
    }

    fn read_from_rom(&self, address: u16) -> u8 {
        println!("Reading from ROM");
        self.rom.data[address as usize]
    }
}
