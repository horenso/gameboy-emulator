use crate::instruction::*;

pub fn decode_unprefixed(opcode: u8) -> Inst {
    let x = opcode >> 6;
    let y = (opcode & 0b00111000) >> 3;
    let z = opcode & 0b00000111;

    match x {
        0 => match z {
            0 => match y {
                0 => Inst::NoOp,
                1 => Inst::Ld(Operand::U8, Operand::R16(Reg16::Sp)),
                2 => Inst::Stop,
                3 => Inst::Jr(Operand::I8),
                4..=7 => Inst::Jr(operand_imm16(y - 4)),
                _ => unreachable!(),
            },
            1 if y & 1 == 0 => Inst::Ld(operand_imm16(y >> 1), Operand::U16),
            1 => Inst::AddHl(reg16(y >> 1)),
            2 => match y {
                0 => Inst::Ld(Operand::ImmR16(Reg16::Bc), Operand::R8(Reg8::A)),
                1 => Inst::Ld(Operand::ImmR16(Reg16::De), Operand::R8(Reg8::A)),
                2 => Inst::Ld(Operand::ImmR16(Reg16::HlIncr), Operand::R8(Reg8::A)),
                3 => Inst::Ld(Operand::ImmR16(Reg16::HlDecr), Operand::R8(Reg8::A)),
                4 => Inst::Ld(Operand::R8(Reg8::A), Operand::ImmR16(Reg16::Bc),
                5 => Inst::Ld(Operand::R8(Reg8::A), Operand::ImmR16(Reg16::De),
                6 => Inst::Ld(Operand::R8(Reg8::A), Operand::ImmR16(Reg16::HlIncr),
                7 => Inst::Ld(Operand::R8(Reg8::A), Operand::ImmR16(Reg16::HlDecr),
            },
            _ => unreachable!(),
        },
        1 if y == 0 && z == 0 => Inst::Halt,
        1 => Inst::Ld(operand(y), operand(z)),
        2 => arithmetic_logic(y, z),
        3 => unreachable!(), // TODO
        _ => unreachable!(),
    }
}

fn operand(code: u8) -> Operand {
    match code {
        0 => Operand::R8(Reg8::B),
        1 => Operand::R8(Reg8::C),
        2 => Operand::R8(Reg8::D),
        3 => Operand::R8(Reg8::E),
        4 => Operand::R8(Reg8::H),
        5 => Operand::R8(Reg8::L),
        6 => Operand::ImmR16(Reg16::Hl),
        7 => Operand::R8(Reg8::A),
        _ => unreachable!(),
    }
}

fn operand_imm16(code: u8) -> Operand {
    match code {
        0 => Operand::ImmR16(Reg16::Bc),
        1 => Operand::ImmR16(Reg16::De),
        2 => Operand::ImmR16(Reg16::Hl),
        3 => Operand::ImmR16(Reg16::Sp),
        _ => unreachable!(),
    }
}

fn reg16(code: u8) -> Reg16 {
    match code {
        0 => Reg16::Bc,
        1 => Reg16::De,
        2 => Reg16::Hl,
        3 => Reg16::Sp,
        _ => unreachable!(),
    }
}

fn cond(code: u8) -> Cond {
    match code {
        0 => Cond::Nz,
        1 => Cond::Z,
        2 => Cond::Nc,
        3 => Cond::C,
        _ => unreachable!(),
    }
}

fn arithmetic_logic(y: u8, z: u8) -> Inst {
    match y {
        0 => Inst::Add(operand(z)),
        1 => Inst::Adc(operand(z)),
        2 => Inst::Sub(operand(z)),
        3 => Inst::Sbc(operand(z)),
        4 => Inst::And(operand(z)),
        5 => Inst::Xor(operand(z)),
        6 => Inst::Or(operand(z)),
        7 => Inst::Cp(operand(z)),
        _ => unreachable!(),
    }
}

pub fn decode_prefixed(opcode: u8) -> Inst {
    let z = opcode & 0b00000111;
    let y = (opcode & 0b00111000) >> 3;
    match opcode {
        0x00..=0x07 => Inst::Rlc(operand(y)),
        0x08..=0x0F => Inst::Rrc(operand(y)),
        0x10..=0x17 => Inst::Rl(operand(y)),
        0x18..=0x1F => Inst::Rr(operand(y)),
        0x20..=0x27 => Inst::Sla(operand(y)),
        0x28..=0x2F => Inst::Sra(operand(y)),
        0x30..=0x37 => Inst::Swap(operand(y)),
        0x38..=0x3F => Inst::Srl(operand(y)),
        0x40..=0x7F => Inst::Bit(y, operand(y)),
        0x80..=0xBF => Inst::Res(y, operand(y)),
        0xC0..=0xFF => Inst::Set(y, operand(y)),
    }
}

#[cfg(test)]
mod tests {
    use super::{decode_prefixed, *};

    #[test]
    fn test_decode_unprefixed() {}

    #[test]
    fn test_decode_prefixed() {
        assert_eq!(decode_prefixed(0x62), Inst::Bit(4, Operand::R8(Reg8::D)));
    }
}
