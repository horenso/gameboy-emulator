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
        println!("Reading bus at {:#x}", address);
        match address {
            0x0000..=0x8000 => self.read_from_rom(address),
            0x8001..=0x9fff => {
                let v_ram_address = (address - 0x8001) as usize;
                self.v_ram[v_ram_address]
            }
            0xFF44 => 0x90, // TODO: Remove this, this is for Gameboy Doctor
            _ => {
                return 0; // TODO: hack
                println!("Access to {:#x} not implemented", address);
                panic!();
            }
        }
    }

    pub fn write(&self, address: u16, data: u8) {
        println!("Writting to address: {:#x} data: {:#x}", address, data);
    }

    fn read_from_rom(&self, address: u16) -> u8 {
        self.rom.read(address as usize)
    }
}
