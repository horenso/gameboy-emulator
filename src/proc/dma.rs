use std::cell::Cell;

use crate::{bus::Bus, util::helper::combine_to_u16};

use super::cpu::Cpu;

/// Direct Memory Access
/// Transfers 160 bytes from XX00-XX9F => FE00-FE9F,
/// where XX can be set at the start of the transfer.
#[derive(Debug)]
pub(crate) struct Dma {
    is_active: bool,
    upper: u8, // the XX in the source XX00-XX9F
    lower: u8, // the current offset from 00 to including 9F
    start_delay: u8,
}

impl Dma {
    pub fn new() -> Self {
        Dma {
            is_active: false,
            upper: 0,
            lower: 0,
            start_delay: 0,
        }
    }

    pub fn start(&mut self, upper: u8) {
        self.is_active = true;
        self.start_delay = 2;
        self.upper = upper;
        self.lower = 0;
        eprintln!("starting dma for {:x}00-{:x}9F", upper, upper);
    }

    pub fn tick(&mut self, bus: &mut Bus) {
        if !self.is_active {
            return;
        }
        if self.start_delay > 0 {
            self.start_delay -= 1;
            return;
        }
        let addr = combine_to_u16(self.upper, self.lower);
        let data = bus.read(Option::None, addr);
        bus.write_oam(self.lower, data);
        self.lower += 1;
        self.is_active = self.lower < 160;
        if !self.is_active {
            eprintln!("dma transfered to {:x}", self.upper);
        }
    }

    pub fn is_active(&self) -> bool {
        self.is_active
    }
}
