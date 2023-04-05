#[derive(PartialEq, Debug)]
pub enum Inst {
    Prefix, // 0xCB is a prefix 2 byte encoded instructions
    Ld(Operand, Operand),
    Push(Reg16),
    Pop(Reg16),

    // Control
    NoOp,
    Halt,
    Stop,
    Di,
    Ei,

    // Jumps
    Jr(Operand),
    Jp(Operand),
    Call(Cond),
    Ret(Cond),
    Reti, // return from interrupt handler
    Rst(u8),

    // Arithmetic
    Add(Operand),
    AddHl(Reg16),
    AddSp,        // Add immediate 8-bit signed value to SP
    Adc(Operand), // add with carry
    Sub(Operand),
    Sbc(Operand), // subtract with carry
    And(Operand),
    Xor(Operand),
    Or(Operand),
    Cp(Operand), // compare
    Inc(Operand),
    Dec(Operand),

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

    // Assorted operations on accumulator or flags
    Rlca,
    Rrca,
    Rla,
    Rra,
    Daa, // decimal adjust A
    Cpl,
    Scf,
    Ccf,
}

#[derive(PartialEq, Debug)]
pub enum Operand {
    D8,
    D16,
    A8,
    R8(Reg8),
    R16(Reg16),
    ImmR16(Reg16),
}

#[derive(PartialEq, Debug)]
pub enum Cond {
    Always,
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
    Af,
    Bc,
    De,
    Hl,
    HlIncr,
    HlDecr,
    Sp,
    SpPlusD,
}

#[derive(PartialEq, Debug)]
pub enum Reg {
    R8(Reg8),
    R16(Reg16),
}
