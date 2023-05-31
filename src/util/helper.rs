pub fn split_u16(value: u16) -> (u8, u8) {
    let high = (value >> 8) as u8;
    let low = (value & 0xFF) as u8;
    (high, low)
}

pub fn split_u32(value: u32) -> (u16, u16) {
    let high = (value >> 16) as u16;
    let low = (value & 0xFFFF) as u16;
    (high, low)
}

pub fn combine_to_u16(high: u8, low: u8) -> u16 {
    (high as u16) << 8 | low as u16
}
