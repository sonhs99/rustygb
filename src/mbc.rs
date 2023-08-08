use crate::device::IOHandler;
use crate::mmu::{MemoryRead, MemoryWrite};

pub struct Cartridge {
    rom: Vec<u8>,
    ram: Vec<u8>,
    reg: [u8; 4],
    rom_bank: u32,
    ram_bank: u32,
}

impl Cartridge {
    pub fn new(rom: Vec<u8>, ram: Vec<u8>) -> Cartridge {
        Cartridge {
            rom: rom,
            ram: ram,
            reg: [0, 1, 0, 0],
            rom_bank: 0x4000,
            ram_bank: 0x0000,
        }
    }
}

impl IOHandler for Cartridge {
    fn read(&mut self, mmu: &crate::mmu::MemoryBus, address: u16) -> MemoryRead {
        match address {
            0x0000..=0x3FFF => MemoryRead::Value(self.rom[address as usize]),
            0x4000..=0x7FFF => {
                MemoryRead::Value(self.rom[((address & 0x3FFF) as u32 + self.rom_bank) as usize])
            }
            0xA000..=0xBFFF => {
                MemoryRead::Value(self.ram[((address & 0x1FFF) as u32 + self.ram_bank) as usize])
            }
            _ => MemoryRead::PassThrough,
        }
    }
    fn write(&mut self, mmu: &crate::mmu::MemoryBus, address: u16, value: u8) -> MemoryWrite {
        match address {
            0x0000..=0x1FFF => self.reg[0] = value,
            0x2000..=0x3FFF => {
                // println!("rom bank changed {}", value);
                self.reg[1] = value;
                self.rom_bank = (if value != 0 { (value & 0x3F) as u32 } else { 1 }) << 14;
            }
            0x4000..=0x5FFF => {
                self.reg[2] = value;
                if value <= 3 {
                    self.ram_bank = (value as u32) << 13;
                }
            }
            0x6000..=0x7FFF => self.reg[3] = value,
            0xA000..=0xBFFF => {
                self.ram[((address & 0x1FFF) as u32 + self.ram_bank) as usize] = value;
            }
            _ => return MemoryWrite::PassThrough,
        }
        MemoryWrite::PassThrough
    }
}
