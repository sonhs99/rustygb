use core::cell::RefCell;

use alloc::rc::Rc;

use crate::FrameBuffer;

pub trait Hardware {
    fn is_active(&mut self) -> bool;
    fn draw_framebuffer(&mut self, frame_buffer: &FrameBuffer);
    fn get_keys(&mut self) -> (u8, u8);
    fn update(&mut self);
}

pub struct HardwareHandle(Rc<RefCell<dyn Hardware>>);

impl HardwareHandle {
    pub fn new<T: Hardware + 'static>(inner: T) -> HardwareHandle {
        HardwareHandle(Rc::new(RefCell::new(inner)))
    }
    pub fn get(&self) -> &Rc<RefCell<dyn Hardware>> {
        &self.0
    }
}
