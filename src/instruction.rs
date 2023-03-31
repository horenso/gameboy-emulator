#[derive(PartialEq, Debug)]
pub enum InstType {
    Unknown,
    Prefix, // 0xCB is a prefex 2 byte encoded instructions

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
    Ldh, // load heigh 0xFFXX\

    Jp,

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
    NotZ,
    Z,
    NotC,
    C,
}

#[derive(PartialEq, Debug)]
pub enum Reg {
    None,
    A,
    F,
    B,
    C,
    D,
    E,
    H,
    L,
    Af, // TODO: Do I need this?
    Bc,
    De,
    Hl,
    Sp,
    Pc,
}

#[derive(Debug)]
pub struct Instruction {
    pub opcode: u8,
    pub inst_type: InstType,
    pub addr_mode: AddrMode,
    pub reg_1: Reg,
    pub reg_2: Reg,
    pub cond: Cond,
}

impl Instruction {
    pub fn from_opcode(opcode: u8) -> Self {
        let mut inst_type = InstType::Unknown;
        let mut addr_mode = AddrMode::None;
        let mut reg_1 = Reg::None;
        let mut reg_2 = Reg::None;
        let mut cond = Cond::None;
        match opcode {
            0x00 => {
                inst_type = InstType::NoOp;
            }
            0x40..=0x75 => {
                inst_type = InstType::Ld;
            }
            0x76 => {
                inst_type = InstType::Halt;
            }
            0x77..=0x7F => {
                inst_type = InstType::Ld;
            }

            0x80..=0xBF => {
                return Self::arithmetic_logic(opcode);
            }

            0xC3 => {
                inst_type = InstType::Jp;
                addr_mode = AddrMode::Addr16;
                cond = Cond::NotZ;
            }
            _ => {
                println!("Unsupported opcode: {}", opcode);
                panic!("Dunno what to do!");
            }
        }

        Instruction {
            opcode,
            inst_type,
            addr_mode,
            reg_1,
            reg_2,
            cond,
        }
    }

    pub fn prefixed_from_obcode(opcode: u8) -> Self {
        let inst_type = match opcode {
            0x00..=0x07 => InstType::Rlc,
            0x08..=0x0F => InstType::Rrc,
            0x10..=0x17 => InstType::Rl,
            0x18..=0x1F => InstType::Rr,
            0x20..=0x27 => InstType::Sla,
            0x28..=0x2F => InstType::Sra,
            0x30..=0x37 => InstType::Swap,
            0x38..=0x3F => InstType::Srl,
            0x40..=0x7F => InstType::Bit,
            0x80..=0xBF => InstType::Res,
            0xC0..=0xFF => InstType::Set,
        };
        let reg_1 = match opcode % 8 {
            0 => Reg::B,
            1 => Reg::C,
            2 => Reg::D,
            3 => Reg::E,
            4 => Reg::H,
            5 => Reg::L,
            6 => Reg::Hl,
            7 => Reg::A,
            _ => unreachable!(),
        };
        Instruction {
            opcode,
            inst_type,
            addr_mode: AddrMode::None,
            reg_1,
            reg_2: Reg::None,
            cond: Cond::None,
        }
    }

    fn arithmetic_logic(opcode: u8) -> Self {
        let inst_type = match opcode {
            0x80..=0x87 => InstType::Add,
            0x88..=0x8F => InstType::Adc,
            0x90..=0x97 => InstType::Sub,
            0x98..=0x9F => InstType::Sbc,
            0xA0..=0xA7 => InstType::And,
            0xA8..=0xAF => InstType::Xor,
            0xB0..=0xB7 => InstType::Or,
            0xB8..=0xBF => InstType::Cp,
            _ => unreachable!(),
        };
        let reg_1 = Reg::A;
        let reg_2 = match opcode % 8 {
            0 => Reg::B,
            1 => Reg::C,
            2 => Reg::D,
            3 => Reg::E,
            4 => Reg::H,
            5 => Reg::L,
            6 => Reg::Hl,
            7 => Reg::A,
            _ => unreachable!(),
        };
        Instruction {
            opcode,
            inst_type,
            addr_mode: AddrMode::None,
            reg_1,
            reg_2,
            cond: Cond::None,
        }
    }
}
