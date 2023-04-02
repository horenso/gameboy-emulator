#[derive(PartialEq, Debug)]
#[allow(non_camel_case_types)] // LdRR instead of LD_r_r is much less readable
pub enum Inst {
    // 0xCB is a prefex 2 byte encoded instructions
    Prefix,

    // 8-bit loads
    LD_r_r(Reg8, Reg8),
    LD_r_d8(Reg8, u8),
    LD_r_HL(Reg8),
    LD_HL_r(Reg8),
    LD_HL_d8(u8),
    LD_A_BC,
    LD_A_DE,
    LD_A_d16(u16),
    LD_BC_A,
    LD_DE_A,
    LD_nnnn_A(u16),
    LDH_n_A(u8),
    LDH_A_n(u8),

    // Control
    NoOp,
    Halt,
    Stop,
    Di,
    Ei,

    // Loads
    Ld,
    Ldi, // load increment
    Ldd, // load decrement
    Ldh, // load heigh 0xFFxx

    Jp,
    Jr, // Jump relative

    // Arithmetic
    Add,
    Adc, // add with carry
    Sub,
    Sbc, // subtract with carry
    And,
    Xor,
    Or,
    Cp,  // compare
    Cpl, // Flip bits (xor 0xFF)
    Inc,
    Dec,
    Daa, // decimal adjust A

    // Rotations and shifts
    Rlc,
    Rrc,
    Rl,
    Rr,
    Sla,
    Sra,
    Swap,
    Srl,
    Bit,
    Res,
    Set,
}

#[derive(PartialEq, Debug)]
pub enum AddrMode {
    None,

    // Immediate values
    Data8,
    Data16,

    // 16-bit address
    Addr8,  // implied 0xFFxx prefix
    Addr16, // full address

    //
    Reg8,
}

#[derive(PartialEq, Debug)]
pub enum Cond {
    None,
    NZ,
    Z,
    NC,
    C,
}

#[derive(PartialEq, Debug)]
enum Reg8 {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
}

enum Reg16 {
    BC,
    DE,
    HL,
    SP,
}

#[derive(Debug)]
pub struct Instruction {
    pub inst_type: Inst,
    pub addr_mode: AddrMode,
    pub reg_1: Reg,
    pub reg_2: Reg,
    pub cond: Cond,
}

impl Instruction {
    pub fn from_opcode(opcode: u8) -> Self {
        let x = opcode >> 6;
        let y = (opcode & 0b00111000) >> 3;
        let z = opcode & 0b00000111;

        match x {
            0 => Self::x0(y, z),
            1 if y == 0 && z == 0 => Instruction {
                inst_type: Inst::Halt,
                addr_mode: AddrMode::None,
                reg_1: Reg::None,
                reg_2: Reg::None,
                cond: Cond::None,
            },
            1 => Instruction {
                inst_type: Inst::Ld,
                addr_mode: AddrMode::Data8,
                reg_1: Self::get_reg(y),
                reg_2: Self::get_reg(z),
                cond: Cond::None,
            },
            2 => return Self::arithmetic_logic(y, z, false),
            3 => unreachable!(), // TODO
            _ => unreachable!(),
        }
    }

    fn x0(y: u8, z: u8) -> Instruction {
        match z {
            0 => match y {
                0 => Instruction {
                    inst_type: Inst::NoOp,
                    addr_mode: AddrMode::None,
                    reg_1: Reg::None,
                    reg_2: Reg::None,
                    cond: Cond::None,
                },
                1 => Instruction {
                    inst_type: Inst::Halt,
                    addr_mode: AddrMode::Addr16,
                    reg_1: Reg::Sp,
                    reg_2: Reg::None,
                    cond: Cond::None,
                },
                2 => Instruction {
                    inst_type: Inst::Stop,
                    addr_mode: AddrMode::None,
                    reg_1: Reg::None,
                    reg_2: Reg::None,
                    cond: Cond::None,
                },
                3 => Instruction {
                    inst_type: Inst::Jr,
                    addr_mode: AddrMode::Reg8,
                    reg_1: Reg::None,
                    reg_2: Reg::None,
                    cond: Cond::None,
                },
                4..=7 => Instruction {
                    inst_type: Inst::Jr,
                    addr_mode: AddrMode::Reg8,
                    reg_1: Reg::None,
                    reg_2: Reg::None,
                    cond: Self::get_cond(y - 4),
                },
                _ => unreachable!(),
            },
            1 if y & 1 == 0 => Instruction {
                inst_type: Inst::Ld,
                addr_mode: AddrMode::Data16,
                reg_1: Reg::None,
                reg_2: Reg::None,
                cond: Cond::None,
            },
            1 => Instruction {
                inst_type: Inst::Add,
                addr_mode: AddrMode::Data16,
                reg_1: Reg::None,
                reg_2: Reg::None,
                cond: Cond::None,
            },
            _ => unreachable!(),
        }
    }

    fn get_reg(code: u8) -> Reg {
        match code {
            0 => Reg::B,
            1 => Reg::C,
            2 => Reg::D,
            3 => Reg::E,
            4 => Reg::H,
            5 => Reg::L,
            6 => Reg::Hl,
            7 => Reg::A,
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

    pub fn prefixed_from_obcode(opcode: u8) -> Self {
        let inst_type = match opcode {
            0x00..=0x07 => Inst::Rlc,
            0x08..=0x0F => Inst::Rrc,
            0x10..=0x17 => Inst::Rl,
            0x18..=0x1F => Inst::Rr,
            0x20..=0x27 => Inst::Sla,
            0x28..=0x2F => Inst::Sra,
            0x30..=0x37 => Inst::Swap,
            0x38..=0x3F => Inst::Srl,
            0x40..=0x7F => Inst::Bit,
            0x80..=0xBF => Inst::Res,
            0xC0..=0xFF => Inst::Set,
        };
        Instruction {
            inst_type,
            addr_mode: AddrMode::None,
            reg_1: Reg::A,
            reg_2: Reg::None,
            cond: Cond::None,
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
        Instruction {
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
}
