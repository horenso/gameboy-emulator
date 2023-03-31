use crate::bus::Bus;
use crate::instruction::{InstType, Instruction};
use crate::registers::Registers;

pub struct Cpu {
    regs: Registers,
    cur_inst: Option<Instruction>,
}

impl Cpu {
    pub fn new() -> Self {
        Cpu {
            regs: Registers::new(),
            cur_inst: None,
        }
    }

    pub fn fetch_and_execute(&mut self, bus: &Bus) {
        let inst = self.fetch(bus);
        self.execute(inst);
    }

    fn fetch(&self, bus: &Bus) -> Instruction {
        let fetched = bus.read_at_address(self.regs.pc);
        let inst = Instruction::from_opcode(fetched);
        println!("{:?}", inst);
        inst
    }

    pub fn execute(&mut self, inst: Instruction) {
        match inst.inst_type {
            InstType::NoOp => {
                self.regs.pc += 1;
            }
            _ => {
                panic!("I don't know how to execute {:?}!", inst.inst_type);
            }
        }
    }
}
