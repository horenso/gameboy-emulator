pub struct Bus {
    rom: Cartridge,
    v_ram: [u8; 8192],
    ram: [u8; 8192],
}

impl Bus {}

// https://gbdev.io/pandocs/Memory_Map.html
pub fn read_at_address(address: u16) -> u8 {
    match address {
        0x0000..0x8000 => read_from_rom(address),
        0x8001..0x9fff => v_ram[address - 0x8001],
        _ => todo!(format!("Access to {} not implemented", address)),
    }
}

fn read_from_rom(address: u16) -> u8 {
    println!("Reading from ROM");
    rom.data[address]
}
