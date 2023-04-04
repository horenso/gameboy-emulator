#[derive(PartialEq, Debug)]
pub enum Inst {
    // 0xCB is a prefix 2 byte encoded instructions
    Prefix,
    Ld(LdOp, LdOp),
    Ldh(LdOp, LdOp),
    Push(Reg16),
    Pop(Reg16),

    // Control
    NoOp,
    Halt,
    Stop,
    Di,
    Ei,
    Jp,
    Jr, // Jump relative

    // Arithmetic
    Add(Reg8),
    Adc(Reg8), // add with carry
    Sub(Reg8),
    Sbc(Reg8), // subtract with carry
    And(Reg8),
    Xor(Reg8),
    Or(Reg8),
    Cp(Reg8),  // compare
    Cpl(Reg8), // Flip bits (xor 0xFF)
    Inc(Reg8),
    Dec(Reg8),

    Daa, // decimal adjust A

    // Rotations and shifts
    Rlc(Reg8),
    Rrc(Reg8),
    Rl(Reg8),
    Rr(Reg8),
    Sla(Reg8),
    Sra(Reg8),
    Swap(Reg8),
    Srl(Reg8),
    Bit(Reg8, u8),
    Res(Reg8, u8),
    Set(Reg8, u8),
}

#[derive(PartialEq, Debug)]
pub enum LdOp {
    D8,
    D16,
    R8(Reg8),
    R16 {
        reg: Reg16,
        offset: Offset,
        immediate: bool,
    },
    A8,
    A16,
}

#[derive(PartialEq, Debug)]
pub enum Cond {
    Nz,
    Z,
    Nc,
    C,
}

#[derive(PartialEq, Debug)]
pub enum Offset {
    Decrement,
    No,
    Increment,
}

#[derive(PartialEq, Debug)]
pub enum Reg8 {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
}

#[derive(PartialEq, Debug)]
pub enum Reg16 {
    Bc,
    De,
    Hl,
    Sp,
}

#[derive(PartialEq, Debug)]
pub enum Reg {
    R8(Reg8),
    R16(Reg16),
}

impl Instruction {
