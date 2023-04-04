pub fn decode_unprefixed(opcode: u8) -> Inst {
    let x = opcode >> 6;
    let y = (opcode & 0b00111000) >> 3;
    let z = opcode & 0b00000111;

    match x {
        0 => Self::x0(y, z),
        1 if y == 0 && z == 0 => Inst::Halt,
        1 => Inst::Ld(ld_operand(y, true), ld_operand(z, true)),
        2 => arithmetic_logic(y, z, false),
        3 => unreachable!(), // TODO
        _ => unreachable!(),
    }
}

fn ld_operand(code: u8, immediate: bool) -> LdOp {
    match code {
        0 => LdOp::R8(Reg8::B),
        1 => LdOp::R8(Reg8::C),
        2 => LdOp::R8(Reg8::D),
        3 => LdOp::R8(Reg8::E),
        4 => LdOp::R8(Reg8::H),
        5 => LdOp::R8(Reg8::L),
        6 => LdOp::R16(Reg::Hl, Offset::No, immediate),
        7 => LdOp::R8(Reg::A),
        _ => unreachable!(),
    }
}

fn x0(y: u8, z: u8) -> Inst {
    match z {
        0 => match y {
            0 => Inst {
                inst_type: Inst::NoOp,
                addr_mode: AddrMode::None,
                reg_1: Reg::None,
                reg_2: Reg::None,
                cond: Cond::None,
            },
            1 => Inst {
                inst_type: Inst::Halt,
                addr_mode: AddrMode::Addr16,
                reg_1: Reg::Sp,
                reg_2: Reg::None,
                cond: Cond::None,
            },
            2 => Inst {
                inst_type: Inst::Stop,
                addr_mode: AddrMode::None,
                reg_1: Reg::None,
                reg_2: Reg::None,
                cond: Cond::None,
            },
            3 => Inst {
                inst_type: Inst::Jr,
                addr_mode: AddrMode::Reg8,
                reg_1: Reg::None,
                reg_2: Reg::None,
                cond: Cond::None,
            },
            4..=7 => Inst {
                inst_type: Inst::Jr,
                addr_mode: AddrMode::Reg8,
                reg_1: Reg::None,
                reg_2: Reg::None,
                cond: Self::get_cond(y - 4),
            },
            _ => unreachable!(),
        },
        1 if y & 1 == 0 => Inst {
            inst_type: Inst::Ld,
            addr_mode: AddrMode::Data16,
            reg_1: Reg::None,
            reg_2: Reg::None,
            cond: Cond::None,
        },
        1 => Inst {
            inst_type: Inst::Add,
            addr_mode: AddrMode::Data16,
            reg_1: Reg::None,
            reg_2: Reg::None,
            cond: Cond::None,
        },
        _ => unreachable!(),
    }
}

fn get_cond(code: u8) -> Cond {
    match code {
        0 => Cond::NZ,
        1 => Cond::Z,
        2 => Cond::NC,
        3 => Cond::C,
        _ => unreachable!(),
    }
}

fn arithmetic_logic(y: u8, z: u8, immediate: bool) -> Self {
    let inst_type = match y {
        0 => Inst::Add,
        1 => Inst::Adc,
        2 => Inst::Sub,
        3 => Inst::Sbc,
        4 => Inst::And,
        5 => Inst::Xor,
        6 => Inst::Or,
        7 => Inst::Cp,
        _ => unreachable!(),
    };
    Inst {
        inst_type,
        addr_mode: if immediate {
            AddrMode::None
        } else {
            AddrMode::Data8
        },
        reg_1: Reg::A,
        reg_2: if immediate {
            Self::get_reg(z)
        } else {
            Reg::None
        },
        cond: Cond::None,
    }
}

pub fn decode_prefixed(opcode: u8) -> Inst {
    let x = opcode >> 6;
    let y = (opcode & 0b00111000) >> 3;
    let z = opcode & 0b00000111;

    let inst_type = match opcode {
        0x00..=0x07 => Inst::Rlc(),
        0x08..=0x0F => Inst::Rrc(),
        0x10..=0x17 => Inst::Rl(),
        0x18..=0x1F => Inst::Rr(),
        0x20..=0x27 => Inst::Sla(),
        0x28..=0x2F => Inst::Sra(),
        0x30..=0x37 => Inst::Swap(),
        0x38..=0x3F => Inst::Srl(),
        0x40..=0x7F => Inst::Bit(),
        0x80..=0xBF => Inst::Res(),
        0xC0..=0xFF => Inst::Set(),
    };
    Inst {
        inst_type,
        addr_mode: AddrMode::None,
        reg_1: Reg::A,
        reg_2: Reg::None,
        cond: Cond::None,
    }
}
