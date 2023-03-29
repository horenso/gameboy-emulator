#[derive(PartialEq, Debug)]
pub enum InstType {
    Unknown,
    // Control
    NoOp,
    Halt,
    Stop,

    Load,
    Jump,
}

#[derive(PartialEq, Debug)]
pub enum AddrMode {
    None,
    Data8,
    Data16,
    Reg,
    Addr8,
    Addr16,
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
    E,
    H,
    L,
    Af,
    Be,
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
            0xC3 => {
                inst_type = InstType::Jump;
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
}
