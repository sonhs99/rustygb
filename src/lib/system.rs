use crate::{
    cpu::CPU,
    cycle::Clock,
    device::Device,
    dma::DMA,
    gpu::{FrameBuffer, GPU},
    hardware::{Hardware, HardwareHandle},
    input::Pad,
    mbc::Cartridge,
    mmu::MemoryBus,
};

pub struct System {
    cpu: CPU,
    bus: MemoryBus,
    gpu: Device<GPU>,
    cartrigde: Device<Cartridge>,
    dma: Device<DMA>,
    clock: Device<Clock>,
    input: Device<Pad>,
    hardware: HardwareHandle,
}

impl System {
    pub fn new<T>(cart: Cartridge, hardware: T) -> System
    where
        T: Hardware + 'static,
    {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new();

        let hardware = HardwareHandle::new(hardware);

        let gpu = Device::new(GPU::new());
        let cartridge = Device::new(cart);
        let dma = Device::new(DMA::new());
        let clock = Device::new(Clock::new());
        let input = Device::new(Pad::new());

        bus.add_handler((0x0000, 0x7FFF), cartridge.handler());
        bus.add_handler((0x8000, 0x9FFF), gpu.handler());
        bus.add_handler((0xA000, 0xBFFF), cartridge.handler());
        bus.add_handler((0xFE00, 0xFE9F), gpu.handler());

        bus.add_handler((0xFF00, 0xFF00), input.handler());
        bus.add_handler((0xFF04, 0xFF07), clock.handler());
        bus.add_handler((0xFF40, 0xFF45), gpu.handler());
        bus.add_handler((0xFF46, 0xFF46), dma.handler());
        bus.add_handler((0xFF47, 0xFF4B), gpu.handler());

        System {
            cpu: cpu,
            bus: bus,
            gpu: gpu,
            cartrigde: cartridge,
            dma: dma,
            clock: clock,
            input: input,
            hardware: hardware,
        }
    }

    pub fn step(&mut self) -> u32 {
        let elasped_cycle = self.cpu.step(&mut self.bus);
        self.clock.borrow_mut().step(&mut self.bus, elasped_cycle);
        self.gpu
            .borrow_mut()
            .step(elasped_cycle, &mut self.bus, &self.hardware);
        self.dma.borrow_mut().step(&mut self.bus);
        self.input.borrow_mut().step(&self.hardware);
        self.hardware.get().borrow_mut().update();
        1
    }

    pub fn is_active(&mut self) -> bool {
        self.hardware.get().borrow_mut().is_active()
    }
}

pub fn run<T>(cart: Cartridge, hardware: T)
where
    T: Hardware + 'static,
{
    let mut system = System::new(cart, hardware);

    while system.is_active() {
        system.step();
    }
}
