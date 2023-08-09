use crate::device::IOHandler;
use crate::mmu::{MemoryBus, MemoryRead, MemoryWrite};

struct Sound {
    // Global Reg
    nr50: u8,
    nr51: u8,
    nr52: u8,

    // Channel 1
    nr10: u8,
    nr11: u8,
    nr12: u8,
    nr13: u8,
    nr14: u8,

    // Channel 2
    nr21: u8,
    nr22: u8,
    nr23: u8,
    nr24: u8,

    // Channel 3
    nr30: u8,
    nr31: u8,
    nr32: u8,
    nr33: u8,
    nr34: u8,
    wave: [u8; 16],

    // Channel 4
    nr41: u8,
    nr42: u8,
    nr43: u8,
    nr44: u8,
}

impl Sound {
    pub fn new() -> Sound {
        Sound {
            nr50: 0,
            nr51: 0,
            nr52: 0,
            nr10: 0,
            nr11: 0,
            nr12: 0,
            nr13: 0,
            nr14: 0,
            nr21: 0,
            nr22: 0,
            nr23: 0,
            nr24: 0,
            nr30: 0,
            nr31: 0,
            nr32: 0,
            nr33: 0,
            nr34: 0,
            wave: [0; 16],
            nr41: 0,
            nr42: 0,
            nr43: 0,
            nr44: 0,
        }
    }

    pub fn step(&mut self, elasped_cycles: u16) {}
}

impl IOHandler for Sound {
    fn read(&mut self, _mmu: &MemoryBus, address: u16) -> MemoryRead {
        match address {
            0xFF24 => MemoryRead::Value(self.nr52),
            0xFF25 => MemoryRead::Value(self.nr51),
            0xFF26 => MemoryRead::Value(self.nr50),

            0xFF10 => MemoryRead::Value(self.nr10),
            0xFF11 => MemoryRead::Value(self.nr11 & 0xC0),
            0xFF12 => MemoryRead::Value(self.nr12),
            // 0xFF13 => MemoryRead::Value(self.nr13),
            0xFF14 => MemoryRead::Value(self.nr14 & 0x40),

            0xFF16 => MemoryRead::Value(self.nr21 & 0xC0),
            0xFF17 => MemoryRead::Value(self.nr22),
            // 0xFF18 => MemoryRead::Value(self.nr23),
            0xFF19 => MemoryRead::Value(self.nr24 & 0x40),

            0xFF1A => MemoryRead::Value(self.nr30),
            // 0xFF1B => MemoryRead::Value(self.nr31),
            0xFF1C => MemoryRead::Value(self.nr32),
            // 0xFF1D => MemoryRead::Value(self.nr33),
            0xFF1E => MemoryRead::Value(self.nr34 & 0x40),
            0xFF30..=0xFF3F => MemoryRead::Value(self.wave[(address & 0x000F) as usize]),

            // 0xFF20 => MemoryRead::Value(self.nr41),
            0xFF21 => MemoryRead::Value(self.nr42),
            0xFF22 => MemoryRead::Value(self.nr43),
            0xFF23 => MemoryRead::Value(self.nr44 & 0x7F),
            _ => MemoryRead::PassThrough,
        }
    }

    fn write(&mut self, _mmu: &MemoryBus, address: u16, value: u8) -> MemoryWrite {
        match address {
            0xFF24 => self.nr52 = self.nr52 | (value & 0x80),
            0xFF25 => self.nr51 = value,
            0xFF26 => self.nr50 = value,

            0xFF10 => self.nr10 = value,
            0xFF11 => self.nr11 = value,
            0xFF12 => self.nr12 = value,
            0xFF13 => self.nr13 = value,
            0xFF14 => self.nr14 = value,

            0xFF16 => self.nr21 = value,
            0xFF17 => self.nr22 = value,
            0xFF18 => self.nr23 = value,
            0xFF19 => self.nr24 = value,

            0xFF1A => self.nr30 = value,
            0xFF1B => self.nr31 = value,
            0xFF1C => self.nr32 = value,
            0xFF1D => self.nr33 = value,
            0xFF1E => self.nr34 = value,
            0xFF30..=0xFF3F => self.wave[(address & 0x000F) as usize] = value,

            0xFF20 => self.nr41 = value,
            0xFF21 => self.nr42 = value,
            0xFF22 => self.nr43 = value,
            0xFF23 => self.nr44 = value,
            _ => {}
        }
        return MemoryWrite::PassThrough;
    }
}
