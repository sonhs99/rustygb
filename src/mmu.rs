pub struct MemoryBus {
    rom: Vec<u8>,
    extram: Vec<u8>,
    pub io: [u8; 128],
    high_ram: [u8; 128],
    pub video_ram: [u8; 8192],
    work_ram: [u8; 16384],
    pub oam: [u8; 160],
    rom1: u32,
    extmbank: u32,
}

impl MemoryBus {
    pub fn new(rom: Vec<u8>, save: Vec<u8>) -> MemoryBus {
        MemoryBus {
            rom: rom,
            extram: save,
            io: [0; 128],
            high_ram: [0; 128],
            video_ram: [0; 8192],
            work_ram: [0; 16384],
            oam: [0; 160],
            rom1: 0x8000,
            extmbank: 0x0000,
        }
    }
    pub fn read_byte(&self, address: u16) -> Option<u8> {
        match address {
            0x0000..=0x3FFF => Some(self.rom[address as usize]),
            0x4000..=0x7FFF => {
                // println!(
                //     "ROM Index = {:#04X}, Real Address = {:#04X}",
                //     self.rom1,
                //     self.rom1 + (address as u32 & 0x3FFF)
                // );
                Some(self.rom[(self.rom1 + (address as u32 & 0x3FFF)) as usize])
            }
            0x8000..=0x9FFF => Some(self.video_ram[(address & 0x1FFF) as usize]),
            0xA000..=0xBFFF => {
                Some(self.extram[(self.extmbank + (address as u32 & 0x1FFF)) as usize])
            }
            0xC000..=0xDFFF => Some(self.work_ram[(address & 0x3FFF) as usize]),
            0xE000..=0xFDFF => Some(self.work_ram[(address & 0x3FFF) as usize]),
            0xFE00..=0xFE9F => Some(self.oam[(address & 0x00FF) as usize]),
            0xFF00..=0xFF7F => {
                match address {
                    0xFF00 => {
                        // joypad
                        if (!self.io[0x00] & 0x10) != 0 {
                            return Some(!(0x10));
                        } else if (!self.io[0x00] & 0x10) != 0 {
                            return Some(!(0x20));
                        }
                        Some(255)
                    }
                    _ => Some(self.io[(address & 0x007F) as usize]),
                }
            }
            0xFF80..=0xFFFF => Some(self.high_ram[(address & 0x007F) as usize]),
            _ => None,
        }
    }
    pub fn write_byte(&mut self, address: u16, value: u8) -> Option<()> {
        match address {
            0x2000..=0x3FFF => {
                self.rom1 = (if value != 0 { (value & 0x3F) as u32 } else { 1 }) << 14;
            }
            0x4000..=0x5FFF => {
                if value <= 3 {
                    self.extmbank = (value as u32) << 13;
                }
            }
            0x8000..=0x9FFF => self.video_ram[(address & 0x1FFF) as usize] = value,
            0xA000..=0xBFFF => {
                self.extram[(self.extmbank + (address as u32 & 0x1FFF)) as usize] = value
            }
            0xC000..=0xDFFF => self.work_ram[(address & 0x3FFF) as usize] = value,
            0xE000..=0xFDFF => self.work_ram[(address & 0x3FFF) as usize] = value,
            0xFE00..=0xFE9F => self.oam[(address & 0x00FF) as usize] = value,
            0xFF00..=0xFF7F => {
                match address {
                    0xFF46 => {
                        // DMA
                        for idx in (0..160).rev() {
                            self.oam[idx] = self.read_byte(value as u16 * 0x100 | idx as u16)?;
                        }
                    }
                    _ => {}
                }
                self.io[(address & 0x007F) as usize] = value;
            }
            0xFF80..=0xFFFF => self.high_ram[(address & 0x007F) as usize] = value,
            _ => return None,
        }
        Some(())
    }

    pub fn get_ie(&self) -> u8 {
        self.high_ram[0x7F]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_unused_address() {
        let bus = MemoryBus::new(vec![0; 0x100000], vec![0; 0x8000]);
        let mut count = 0;
        for addr in 0x0000..=0xFFFF {
            if let None = bus.read_byte(addr) {
                println!("fail : {:#04X}", addr);
                count += 1;
            }
        }
        assert_eq!(count, 96);
    }

    #[test]
    fn check_write_address() {
        let mut bus = MemoryBus::new(vec![0; 0x100000], vec![0; 0x8000]);
        let magic_number: u8 = 0x42;

        assert_eq!(bus.write_byte(0x0000, magic_number), None);
        assert_eq!(bus.write_byte(0x6000, magic_number), None);

        assert_eq!(bus.write_byte(0x8000, magic_number), Some(()));
        assert_eq!(bus.video_ram[0], magic_number);
        println!("video ram [pass]");

        assert_eq!(bus.write_byte(0xA000, magic_number), Some(()));
        assert_eq!(bus.extram[(bus.extmbank + 0) as usize], magic_number);
        println!("exram ram [pass]");

        assert_eq!(bus.write_byte(0xC000, magic_number), Some(()));
        assert_eq!(bus.work_ram[0], magic_number);
        println!("work ram [pass]");

        assert_eq!(bus.write_byte(0xFF00, magic_number), Some(()));
        assert_eq!(bus.io[0], magic_number);
        println!("io [pass]");

        assert_eq!(bus.write_byte(0xFF80, magic_number), Some(()));
        assert_eq!(bus.high_ram[0], magic_number);
        println!("high ram [pass]");
    }
}
