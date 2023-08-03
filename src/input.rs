use crate::device::IOHandler;
use crate::mmu::{MemoryBus, MemoryRead, MemoryWrite};

pub struct Pad {
    cross_button: u8,
    ab_button: u8,
    cross_select: bool,
    ab_select: bool,
}

impl Pad {
    pub fn new() -> Pad {
        Pad {
            cross_button: 0x00,
            ab_button: 0x00,
            cross_select: false,
            ab_select: false,
        }
    }

    pub fn step<T>(&mut self, callback: T)
    where
        T: Fn() -> (u8, u8),
    {
        let (cross_input, ab_input) = callback();
        self.cross_button = cross_input;
        self.ab_button = ab_input;
    }
}

impl IOHandler for Pad {
    fn read(&mut self, mmu: &MemoryBus, address: u16) -> MemoryRead {
        if self.cross_select {
            MemoryRead::Value(!(0x10 | self.cross_button))
        } else if self.ab_select {
            MemoryRead::Value(!(0x20 | self.ab_button))
        } else {
            MemoryRead::Value(0xFF)
        }
    }
    fn write(&mut self, mmu: &MemoryBus, address: u16, value: u8) -> MemoryWrite {
        if address == 0xFF00 {
            if value & 0x10 == 0 {
                self.cross_select = true;
                self.ab_select = false;
            } else if value & 0x20 == 0 {
                self.cross_select = false;
                self.ab_select = true;
            } else {
                self.cross_select = false;
                self.ab_select = false;
            }
            MemoryWrite::Value(value)
        } else {
            MemoryWrite::PassThrough
        }
    }
}
