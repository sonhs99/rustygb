use crate::{
    device::IOHandler,
    mmu::{MemoryBus, MemoryRead, MemoryWrite},
};

pub struct DMA {
    reg: u8,
    active: bool,
}

impl DMA {
    pub fn new() -> DMA {
        DMA {
            reg: 0,
            active: false,
        }
    }

    pub fn step(&mut self, bus: &mut MemoryBus) {
        if self.active {
            self.active = false;
            let src = (self.reg as u16) << 8;
            // println!("Start DMA Transfer from {:04X}", src);
            for idx in 0..160 {
                let value = bus.read_byte(src + idx).unwrap();
                bus.write_byte(0xFE00 + idx, value);
            }
        }
    }
}

impl IOHandler for DMA {
    fn read(&mut self, mmu: &crate::mmu::MemoryBus, address: u16) -> MemoryRead {
        MemoryRead::PassThrough
    }
    fn write(&mut self, mmu: &MemoryBus, address: u16, value: u8) -> MemoryWrite {
        if address == 0xFF46 {
            self.active = true;
            self.reg = value;
        }
        MemoryWrite::Block
    }
}
