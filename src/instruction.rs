enum InstructionType {
    NoOp,
    Load,
    Jump,
}

enum Condition {
    None,
    NotZ,
    Z,
    NotC,
    C,
}

enum Register {
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
}

pub struct Instruction {
    opcode: u8,
    addr_mode: Mode,
    reg_1: Register,
    reg_2: Register,
    condition: Condition,
}
