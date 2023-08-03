use crate::{
    device::IOHandler,
    mmu::{MemoryBus, MemoryRead, MemoryWrite},
};

pub struct Clock {
    cycles: u16,
    counter: u8,
    TMA: u8,
    TAC: u8,
}

impl Clock {
    pub fn new() -> Clock {
        Clock {
            cycles: 0,
            counter: 0,
            TMA: 0,
            TAC: 0,
        }
    }

    pub fn step(&mut self, mmu: &mut MemoryBus, elapsed_cycle: u16) {
        self.cycles = self.cycles.wrapping_add(elapsed_cycle);
        if self.TAC & 0x04 == 0x04 {
            let new_cycles = (elapsed_cycle
                / match self.TAC & 0x03 {
                    0 => 1024,
                    1 => 16,
                    2 => 64,
                    3 => 256,
                    _ => unreachable!(),
                }) as u8;
            if 0xFF - self.counter < new_cycles {
                self.counter = self.TMA;
                mmu.set_if(mmu.get_if() | 0x04);
            } else {
                self.counter += new_cycles;
            }
        }
    }
}

impl IOHandler for Clock {
    fn read(&mut self, mmu: &MemoryBus, address: u16) -> MemoryRead {
        match address {
            0xFF04 => MemoryRead::Value((self.cycles >> 8) as u8),
            0xFF05 => MemoryRead::Value(self.counter),
            0xFF06 => MemoryRead::Value(self.TMA),
            0xFF07 => MemoryRead::Value(self.TAC),
            _ => MemoryRead::PassThrough,
        }
    }
    fn write(&mut self, mmu: &MemoryBus, address: u16, value: u8) -> MemoryWrite {
        match address {
            0xFF04 => self.cycles &= 0x00FF,
            0xFF05 => self.counter = value,
            0xFF06 => self.TMA = value,
            0xFF07 => self.TAC = value,
            _ => {}
        }
        MemoryWrite::PassThrough
    }
}
