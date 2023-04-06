use std::rc::Rc;

use crate::bus::Bus;
use crate::decode::*;
// use crate::helper::combine_to_u16;
use crate::instruction::{Inst, Reg};
use crate::registers::Registers;

pub struct Cpu {
    regs: Registers,
    bus: Rc<Bus>,
}

impl Cpu {
    pub fn new(bus: Rc<Bus>) -> Self {
        Cpu {
            regs: Registers::new(),
            bus,
        }
    }

    pub fn fetch_and_execute(&mut self) {
        let inst = self.fetch();
        self.execute(inst);
    }

    fn read(&mut self) -> u8 {
        let data = self.bus.read_at_address(self.regs.pc);
        self.regs.pc += 1;
        data
    }

    fn fetch(&mut self) -> Inst {
        let fetched = self.read();
        let mut inst = decode_unprefixed(fetched);
        if inst == Inst::Prefix {
            let fetched = self.read();
            inst = decode_prefixed(fetched);
        }
        println!("{:?}", inst);
        inst
    }

    pub fn execute(&mut self, inst: Inst) {
        let data8: u8 = 0;
        let data16: u16 = 0;
        // fetch additional data
        // match inst.addr_mode {
        //     AddrMode::Data8 | AddrMode::Addr8 => {
        //         data8 = self.read();
        //     }
        //     AddrMode::Data16 | AddrMode::Addr16 => {
        //         let d1 = self.read();
        //         let d2 = self.read();
        //         data16 = combine_to_u16(d1, d2);
        //     }
        //     _ => {}
        // }
        // match inst.inst_type {
        //     Inst::NoOp => {
        //         self.regs.pc += 1;
        //     }

        //     // Arithmetic
        //     Inst::Add
        //     _ => {
        //         panic!("I don't know how to execute {:?}!", inst.inst_type);
        //     }
        // }
    }

    fn execute_ld(&mut self) {}

    fn execute_add(&mut self) {}

    fn execute_adc(&mut self) {}

    fn execute_sub(&mut self) {}

    fn execute_sbc(&mut self) {}

    fn execute_and(&mut self) {}

    fn execute_xor(&mut self) {}

    fn execute_or(&mut self) {}

    fn execute_cp(&mut self) {}

    fn execute_cpl(&mut self) {}

    fn execute_inc(&mut self) {}

    fn execute_dec(&mut self) {}

    fn execute_daa(&mut self) {}
}
