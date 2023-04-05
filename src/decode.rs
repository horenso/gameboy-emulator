use crate::instruction::*;

pub fn decode_unprefixed(opcode: u8) -> Inst {
    let x = opcode >> 6;
    let y = (opcode & 0b00111000) >> 3;
    let z = opcode & 0b00000111;

    match x {
        0 => match z {
            0 => match y {
                0 => Inst::NoOp,
                1 => Inst::Ld(Operand::D8, Operand::R16(Reg16::Sp)),
                2 => Inst::Stop,
                3 => Inst::Jr(Operand::A8),
                4..=7 => Inst::Jr(operand_imm16(y - 4)),
                _ => unreachable!(),
            },
            1 if y & 1 == 0 => Inst::Ld(operand_imm16(y >> 1), Operand::D16),
            1 => Inst::AddHl(rp_table(y >> 1)),
            2 => match y {
                0 => Inst::Ld(Operand::ImmR16(Reg16::Bc), Operand::R8(Reg8::A)),
                1 => Inst::Ld(Operand::ImmR16(Reg16::De), Operand::R8(Reg8::A)),
                2 => Inst::Ld(Operand::ImmR16(Reg16::HlIncr), Operand::R8(Reg8::A)),
                3 => Inst::Ld(Operand::ImmR16(Reg16::HlDecr), Operand::R8(Reg8::A)),
                4 => Inst::Ld(Operand::R8(Reg8::A), Operand::ImmR16(Reg16::Bc)),
                5 => Inst::Ld(Operand::R8(Reg8::A), Operand::ImmR16(Reg16::De)),
                6 => Inst::Ld(Operand::R8(Reg8::A), Operand::ImmR16(Reg16::HlIncr)),
                7 => Inst::Ld(Operand::R8(Reg8::A), Operand::ImmR16(Reg16::HlDecr)),
                _ => unreachable!(),
            },
            3 if y & 1 == 0 => Inst::Inc(Operand::R16(rp_table(y >> 1))),
            3 => Inst::Dec(Operand::R16(rp_table(y >> 1))),
            4 => Inst::Inc(operand(y)),
            5 => Inst::Dec(operand(y)),
            6 => Inst::Ld(operand(y), Operand::D8),
            7 => match y {
                0 => Inst::Rlca,
                1 => Inst::Rrca,
                2 => Inst::Rla,
                3 => Inst::Rra,
                4 => Inst::Daa,
                5 => Inst::Cpl,
                6 => Inst::Scf,
                7 => Inst::Ccf,
                _ => unreachable!(),
            },
            _ => unreachable!(),
        },
        1 if y == 0 && z == 0 => Inst::Halt,
        1 => Inst::Ld(operand(y), operand(z)),
        2 => arithmetic_logic(y, z, false),
        3 => match z {
            0 => match y {
                0..=3 => Inst::Ret(cond(y)),
                4 => Inst::Ld(Operand::D8, Operand::R8(Reg8::A)),
                5 => Inst::AddSp,
                6 => Inst::Ld(Operand::R8(Reg8::A), Operand::A8),
                7 => Inst::Ld(Operand::R16(Reg16::Hl), Operand::R16(Reg16::SpPlusD)),
                _ => unreachable!(),
            },
            1 if y & 1 == 0 => Inst::Pop(rp2_table(y >> 1)),
            1 => match y >> 1 {
                0 => Inst::Ret(Cond::Always),
                1 => Inst::Reti,
                2 => Inst::Jp(Operand::R16(Reg16::Hl)),
                3 => Inst::Ld(Operand::R16(Reg16::Sp), Operand::R16(Reg16::Hl)),
                4 => match y {
                    0..=3 => Inst::Call(cond(y)),
                    4..=7 => Inst::NoOp,
                    _ => unreachable!(),
                },
                5 if y & 1 == 0 => Inst::Push(rp2_table(y >> 1)),
                5 if y >> 1 == 0 => Inst::Call(Cond::Always),
                5 => Inst::NoOp,
                6 => arithmetic_logic(y, z, true),
                7 => Inst::Rst(y * 8),
                _ => unreachable!(),
            },
            _ => unreachable!(), // TODO
        },
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

fn rp_table(code: u8) -> Reg16 {
    match code {
        0 => Reg16::Bc,
        1 => Reg16::De,
        2 => Reg16::Hl,
        3 => Reg16::Sp,
        _ => unreachable!(),
    }
}

fn rp2_table(code: u8) -> Reg16 {
    match code {
        0 => Reg16::Bc,
        1 => Reg16::De,
        2 => Reg16::Hl,
        3 => Reg16::Af,
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

fn arithmetic_logic(y: u8, z: u8, immediate: bool) -> Inst {
    let operand = if immediate { Operand::D8 } else { operand(z) };
    match y {
        0 => Inst::Add(operand),
        1 => Inst::Adc(operand),
        2 => Inst::Sub(operand),
        3 => Inst::Sbc(operand),
        4 => Inst::And(operand),
        5 => Inst::Xor(operand),
        6 => Inst::Or(operand),
        7 => Inst::Cp(operand),
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
