use crate::instruction::*;

pub fn decode_unprefixed(opcode: u8) -> Inst {
    let x = opcode >> 6;
    let y = (opcode & 0b00111000) >> 3;
    let z = opcode & 0b00000111;
    let p = y >> 1;
    let q = y & 1;

    match x {
        0 => match z {
            0 => match y {
                0 => Inst::NoOp,
                1 => Inst::Ld16(Operand::A16, Operand::R16(Reg16::Sp)),
                2 => Inst::Stop,
                3 => Inst::Jr(Cond::Always),
                4..=7 => Inst::Jr(cond(y - 4)),
                _ => unreachable!(),
            },
            1 if q == 0 => Inst::Ld16(Operand::R16(rp_table(p)), Operand::D16),
            1 => Inst::AddHl(rp_table(p)),
            2 => match y {
                0 => Inst::Ld8(Operand::IndR16(Reg16::Bc), Operand::R8(Reg8::A)),
                1 => Inst::Ld8(Operand::R8(Reg8::A), Operand::IndR16(Reg16::Bc)),
                2 => Inst::Ld8(Operand::IndR16(Reg16::De), Operand::R8(Reg8::A)),
                3 => Inst::Ld8(Operand::R8(Reg8::A), Operand::IndR16(Reg16::De)),
                4 => Inst::Ld8(Operand::IndR16(Reg16::HlIncr), Operand::R8(Reg8::A)),
                5 => Inst::Ld8(Operand::R8(Reg8::A), Operand::IndR16(Reg16::HlIncr)),
                6 => Inst::Ld8(Operand::IndR16(Reg16::HlDecr), Operand::R8(Reg8::A)),
                7 => Inst::Ld8(Operand::R8(Reg8::A), Operand::IndR16(Reg16::HlDecr)),
                _ => unreachable!(),
            },
            3 if q == 0 => Inst::Inc16(rp_table(p)),
            3 => Inst::Dec16(rp_table(p)),
            4 => Inst::Inc8(operand(y)),
            5 => Inst::Dec8(operand(y)),
            6 => Inst::Ld8(operand(y), Operand::D8),
            7 => match y {
                0 => Inst::Rotate(Rotation::LeftCircular, Operand::R8(Reg8::A)),
                1 => Inst::Rotate(Rotation::RightCircular, Operand::R8(Reg8::A)),
                2 => Inst::Rotate(Rotation::Left, Operand::R8(Reg8::A)),
                3 => Inst::Rotate(Rotation::Right, Operand::R8(Reg8::A)),
                4 => Inst::Daa,
                5 => Inst::Cpl,
                6 => Inst::Scf,
                7 => Inst::Ccf,
                _ => unreachable!(),
            },
            _ => unreachable!(),
        },
        1 if y == 0 && z == 0 => Inst::Halt,
        1 => Inst::Ld8(operand(y), operand(z)),
        2 => arithmetic_logic(y, z, false),
        3 => match z {
            0 => match y {
                0..=3 => Inst::Ret(cond(y)),
                4 => Inst::Ld8(Operand::A8, Operand::R8(Reg8::A)),
                5 => Inst::AddSp,
                6 => Inst::Ld8(Operand::R8(Reg8::A), Operand::A8),
                7 => Inst::Ld8(Operand::R16(Reg16::Hl), Operand::R16(Reg16::SpPlusD)),
                _ => unreachable!(),
            },
            1 if q == 0 => Inst::Pop(rp2_table(p)),
            1 => match p {
                0 => Inst::Ret(Cond::Always),
                1 => Inst::Reti,
                2 => Inst::Jp(Cond::Always, Operand::R16(Reg16::Hl)),
                3 => Inst::Ld16(Operand::R16(Reg16::Sp), Operand::R16(Reg16::Hl)),
                _ => unreachable!(),
            },
            2 => match y {
                0..=3 => Inst::Jp(cond(y), Operand::A16),
                4 => Inst::Ld8(Operand::A8, Operand::R8(Reg8::A)),
                5 => Inst::Ld8(Operand::A16, Operand::R8(Reg8::A)),
                6 => Inst::Ld8(Operand::R8(Reg8::A), Operand::A8),
                7 => Inst::Ld8(Operand::R8(Reg8::A), Operand::A16),
                _ => unreachable!(),
            },
            3 => match y {
                0 => Inst::Jp(Cond::Always, Operand::A16), // TODO: Is this A16?
                1 => Inst::Prefix,
                2..=5 => Inst::NoOp,
                6 => Inst::Di,
                7 => Inst::Ei,
                _ => unreachable!(),
            },
            4 => match y {
                0..=3 => Inst::Call(cond(y)),
                4..=7 => Inst::NoOp,
                _ => unreachable!(),
            },
            5 if q == 0 => Inst::Push(rp2_table(p)),
            5 if p == 0 => Inst::Call(Cond::Always),
            5 => Inst::NoOp,
            6 => arithmetic_logic(y, z, true),
            7 => Inst::Rst(y * 8),
            _ => unreachable!(),
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
        6 => Operand::IndR16(Reg16::Hl),
        7 => Operand::R8(Reg8::A),
        _ => unreachable!(),
    }
}

fn operand_imm16(code: u8) -> Operand {
    match code {
        0 => Operand::IndR16(Reg16::Bc),
        1 => Operand::IndR16(Reg16::De),
        2 => Operand::IndR16(Reg16::Hl),
        3 => Operand::IndR16(Reg16::Sp),
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
        0 => Cond::NotZero,
        1 => Cond::Zero,
        2 => Cond::NotCarry,
        3 => Cond::Carry,
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
    let y = (opcode & 0b00111000) >> 3;
    let z = opcode & 0b00000111;
    let operand = operand(z);
    match opcode {
        0x00..=0x07 => Inst::Rotate(Rotation::LeftCircular, operand),
        0x08..=0x0F => Inst::Rotate(Rotation::RightCircular, operand),
        0x10..=0x17 => Inst::Rotate(Rotation::Left, operand),
        0x18..=0x1F => Inst::Rotate(Rotation::Right, operand),
        0x20..=0x27 => Inst::Shift(ShiftType::LeftArithmetic, operand),
        0x28..=0x2F => Inst::Shift(ShiftType::RightArithmetic, operand),
        0x30..=0x37 => Inst::Swap(operand),
        0x38..=0x3F => Inst::Shift(ShiftType::RightLogic, operand),
        0x40..=0x7F => Inst::TestBit(y, operand),
        0x80..=0xBF => Inst::ResetBit(y, operand),
        0xC0..=0xFF => Inst::SetBit(y, operand),
    }
}

#[cfg(test)]
mod tests {
    use super::{decode_prefixed, *};

    #[test]
    fn test_decode_unprefixed() {}

    #[test]
    fn test_decode_prefixed() {
        assert_eq!(
            decode_prefixed(0x62),
            Inst::TestBit(4, Operand::R8(Reg8::D))
        );
    }
}
