#[derive(PartialEq, Debug)]
pub enum Inst {
    // 0xCB is a prefix 2 byte encoded instructions
    Prefix,
    Ld(Operand, Operand),
    Ldh(Operand, Operand),
    Push(Reg16),
    Pop(Reg16),

    // Control
    NoOp,
    Halt,
    Stop,
    Di,
    Ei,
    Jp,

    // Jumps
    Jr(Operand),

    // Arithmetic
    Add(Operand),
    AddHl(Reg16),
    Adc(Operand), // add with carry
    Sub(Operand),
    Sbc(Operand), // subtract with carry
    And(Operand),
    Xor(Operand),
    Or(Operand),
    Cp(Operand),  // compare
    Cpl(Operand), // Flip bits (xor 0xFF)
    Inc(Operand),
    Dec(Operand),

    Daa, // decimal adjust A

    // Rotations and shifts
    Rlc(Operand),
    Rrc(Operand),
    Rl(Operand),
    Rr(Operand),
    Sla(Operand),
    Sra(Operand),
    Swap(Operand),
    Srl(Operand),
    Bit(u8, Operand),
    Res(u8, Operand),
    Set(u8, Operand),
}

#[derive(PartialEq, Debug)]
pub enum Operand {
    I8,
    U8,
    U16,
    R8(Reg8),
    R16(Reg16),
    ImmR16(Reg16, Offset),
    A8,
    A16,
}

#[derive(PartialEq, Debug)]
pub enum ArthLogOp {
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
    HlIncr,
    HlDecr,
    Sp,
}

#[derive(PartialEq, Debug)]
pub enum Reg {
    R8(Reg8),
    R16(Reg16),
}
