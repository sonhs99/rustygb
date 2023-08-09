use crate::{
    device::IOHandler,
    mmu::{MemoryBus, MemoryRead, MemoryWrite},
};

pub struct Clock {
    cycles: u16,
    counter: u8,
    TMA: u8,
    TAC: u8,
    inc: u8,
}

impl Clock {
    pub fn new() -> Clock {
        Clock {
            cycles: 44288,
            counter: 0,
            TMA: 0,
            TAC: 0,
            inc: 0,
        }
    }

    pub fn step(&mut self, mmu: &mut MemoryBus, elapsed_cycle: u16) {
        self.cycles = self.cycles.wrapping_add(elapsed_cycle);
        if self.TAC & 0x04 == 0x04 {
            let divider = match self.TAC & 0x03 {
                0 => 1024,
                1 => 16,
                2 => 64,
                3 => 256,
                _ => unreachable!(),
            } as u16;
            // println!("{}", divider);
            let new_cycles = (elapsed_cycle / divider) as u8;
            if 0xFF - self.counter < new_cycles {
                let rest = new_cycles - (0xFF - self.counter);
                self.counter = self.TMA + rest;
                mmu.set_if(mmu.get_if() | 0x04);
                // println!("{}", mmu.get_if());
            } else {
                self.counter += new_cycles;
            }
            // println!("{} {} {}", new_cycles, self.counter, self.TMA);
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
