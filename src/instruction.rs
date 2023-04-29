#[derive(PartialEq, Debug)]
pub enum Inst {
    Prefix, // 0xCB is a prefix for 2 byte instructions

    // Control
    NoOp, // no operation
    Halt,
    Stop,
    Di, // disable interrupts
    Ei, // enable interrupts

    // Load and stack operations
    Ld8(Operand, Operand),
    Ld16(Operand, Operand),
    Push(Reg16),
    Pop(Reg16),

    // Jumps
    Jp(Cond, Operand), // absolute jump
    Jr(Cond),          // relatve jump
    Call(Cond),        // call a function
    Ret(Cond),         // return if condition
    Reti,              // return and enable interrupts
    Rst(u8),           // shorthand call to specific locations

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
    Inc8(Operand),
    Inc16(Reg16),
    Dec8(Operand),
    Dec16(Reg16),

    // Rotations and shifts
    Rotate(Rotation, Operand),
    Shift(ShiftType, Operand),
    Swap(Operand),
    TestBit(u8, Operand),
    ResetBit(u8, Operand),
    SetBit(u8, Operand),

    // Assorted operations on accumulator or flags
    Daa, // decimal adjust A
    ComplementA,
    SetCarryFlag,
    ComplementCarryFlag,
}

#[derive(PartialEq, Debug)]
pub enum Operand {
    D8,            // immediate 8-bit data
    D16,           // immediate 16-bit data
    A8,            // immediate 8-bit address where 0xFF.. is implied
    A16,           // immediate 16-bit address
    R8(Reg8),      // 8-bit register
    R16(Reg16),    // 16-bit register
    IndR16(Reg16), // address that is stored the 16-bit register
}

#[derive(PartialEq, Debug)]
pub enum Rotation {
    LeftThroughCarry, // The RR instuctions
    LeftCircular,     // The RRC instructions
    RightThroughCarry,
    RightCircular,
}

#[derive(PartialEq, Debug)]
pub enum ShiftType {
    LeftArithmetic,
    RightArithmetic,
    RightLogic,
}

#[derive(PartialEq, Debug)]
pub enum Cond {
    Always,
    NotZero,
    Zero,
    NotCarry,
    Carry,
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
    Pc,
}
