use std::{
    cell::{Ref, RefCell, RefMut},
    rc::Rc,
};

use crate::mmu::{MemoryBus, MemoryHandler, MemoryRead, MemoryWrite};

pub struct Device<T>(Rc<RefCell<T>>, bool);

pub struct DevHandler<T>(Rc<RefCell<T>>, bool);

impl<T> Device<T> {
    pub fn new(inner: T) -> Self {
        Self::inner(inner, false)
    }

    pub fn mediate(inner: T) -> Self {
        Self::inner(inner, true)
    }

    fn inner(inner: T, debug: bool) -> Self {
        Self(Rc::new(RefCell::new(inner)), debug)
    }

    pub fn borrow<'a>(&'a self) -> Ref<'a, T> {
        self.0.borrow()
    }

    pub fn borrow_mut<'a>(&'a self) -> RefMut<'a, T> {
        self.0.borrow_mut()
    }
}

pub trait IOHandler {
    fn read(&mut self, mmu: &MemoryBus, address: u16) -> MemoryRead;
    fn write(&mut self, mmu: &MemoryBus, address: u16, value: u8) -> MemoryWrite;
}

impl<T: IOHandler> Device<T> {
    pub fn handler(&self) -> DevHandler<T> {
        DevHandler(self.0.clone(), self.1)
    }
}

impl<T: IOHandler> MemoryHandler for DevHandler<T> {
    fn read(&self, mmu: &MemoryBus, address: u16) -> MemoryRead {
        match self.0.try_borrow_mut() {
            Ok(mut inner) => inner.read(mmu, address),
            Err(e) => {
                if self.1 {
                    MemoryRead::PassThrough
                } else {
                    panic!("Recursive read from {:04X} : {}", address, e);
                }
            }
        }
    }
    fn write(&self, mmu: &MemoryBus, address: u16, value: u8) -> MemoryWrite {
        match self.0.try_borrow_mut() {
            Ok(mut inner) => inner.write(mmu, address, value),
            Err(e) => {
                if self.1 {
                    MemoryWrite::PassThrough
                } else {
                    panic!("Recursive write to {:04X} : {}", address, e);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestObject {
        value: [u8; 0x2000],
    }

    impl TestObject {
        pub fn new() -> Self {
            Self { value: [0; 0x2000] }
        }
    }

    impl IOHandler for TestObject {
        fn read(&mut self, mmu: &MemoryBus, address: u16) -> MemoryRead {
            if address >= 0x2000 && address <= 0x3FFF {
                MemoryRead::Value(self.value[(address & 0x1FFF) as usize])
            } else {
                MemoryRead::PassThrough
            }
        }
        fn write(&mut self, mmu: &MemoryBus, address: u16, value: u8) -> MemoryWrite {
            if address >= 0x2000 && address <= 0x3FFF {
                self.value[(address & 0x1FFF) as usize] = value;
                MemoryWrite::Value(value)
            } else {
                MemoryWrite::PassThrough
            }
        }
    }

    #[test]
    fn check_write_address() {
        let mut bus = MemoryBus::new();
        let test_obj = Device::new(TestObject::new());
        bus.add_handler((0x2000, 0x3FFF), test_obj.handler());
        bus.write_byte(0x2000, 0x42);
        assert_eq!(bus.read_byte(0x2000), Some(0x42));
        assert_eq!(test_obj.borrow().value[0], 0x42);
    }
}
