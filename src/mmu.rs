use std::{collections::HashMap, rc::Rc};

pub struct MemoryBus {
    memory: [u8; 0x10000],
    handlers: HashMap<u16, Vec<Rc<dyn MemoryHandler>>>,
}

pub enum MemoryRead {
    Value(u8),
    PassThrough,
}

pub enum MemoryWrite {
    Value(u8),
    PassThrough,
    Block,
}

pub trait MemoryHandler {
    fn read(&self, mmu: &MemoryBus, address: u16) -> MemoryRead;
    fn write(&self, mmu: &MemoryBus, address: u16, value: u8) -> MemoryWrite;
}

impl MemoryBus {
    pub fn new() -> MemoryBus {
        MemoryBus {
            memory: [0; 0x10000],
            handlers: HashMap::new(),
        }
    }
    pub fn add_handler<T>(&mut self, range: (u16, u16), handler: T)
    where
        T: MemoryHandler + 'static,
    {
        let handler = Rc::new(handler);
        for addr in range.0..=range.1 {
            if self.handlers.contains_key(&addr) {
                match self.handlers.get_mut(&addr) {
                    Some(v) => v.push(handler.clone()),
                    None => {}
                }
            } else {
                self.handlers.insert(addr, vec![handler.clone()]);
            }
        }
    }
    pub fn read_byte(&self, address: u16) -> Option<u8> {
        if let Some(handlers) = self.handlers.get(&address) {
            for handler in handlers {
                match handler.read(self, address) {
                    MemoryRead::Value(val) => return Some(val),
                    MemoryRead::PassThrough => {}
                }
            }
        }
        if address >= 0xE000 && address <= 0xFDFF {
            Some(self.memory[(address - 0x2000) as usize])
        } else {
            Some(self.memory[address as usize])
        }
    }
    pub fn write_byte(&mut self, address: u16, value: u8) -> Option<()> {
        if let Some(handlers) = self.handlers.get(&address) {
            for handler in handlers {
                match handler.write(self, address, value) {
                    MemoryWrite::Value(val) => {
                        self.memory[address as usize] = val;
                        return Some(());
                    }
                    MemoryWrite::PassThrough => {}
                    MemoryWrite::Block => return Some(()),
                }
            }
        }
        if address >= 0xE000 && address <= 0xFDFF {
            self.memory[(address - 0x2000) as usize] = value;
        } else {
            self.memory[address as usize] = value;
        }
        Some(())
    }

    pub fn get_ie(&self) -> u8 {
        self.memory[0xffff]
    }

    pub fn get_if(&self) -> u8 {
        self.memory[0xff0f]
    }

    pub fn set_if(&mut self, value: u8) {
        self.memory[0xff0f] = value;
    }
}
