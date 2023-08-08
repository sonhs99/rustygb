use rustygb::{
    cpu::CPU,
    cycle::Clock,
    device::Device,
    dma::DMA,
    gpu::{FrameBuffer, FRAME_HEIGHT, FRAME_WIDTH, GPU},
    input::Pad,
    mbc::Cartridge,
    mmu::MemoryBus,
};

use minifb::{Key, Scale, Window, WindowOptions};
use std::env;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;

fn read_rom(rom_name: &str) -> Vec<u8> {
    let file = File::open(rom_name).expect("File Not Found");
    let buffer = BufReader::new(file);
    let mut rom: Vec<u8> = Vec::new();
    for byte_or_error in buffer.bytes() {
        let byte = byte_or_error.unwrap();
        rom.push(byte);
    }
    rom
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        panic!("usage: rustygb [FileName]");
    }
    let mut window = Window::new(
        "test",
        FRAME_WIDTH,
        FRAME_HEIGHT,
        WindowOptions {
            resize: false,
            scale: Scale::X4,
            ..WindowOptions::default()
        },
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    let rom = read_rom(&args[1]);

    let mut count = 0;
    let mut cpu = CPU::new();
    let mut bus = MemoryBus::new();

    let gpu = Device::new(GPU::new());
    let cartridge = Device::new(Cartridge::new(rom, vec![0; 0x8000]));
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

    while window.is_open() {
        let elasped_cycle = cpu.step(&mut bus);
        clock.borrow_mut().step(&mut bus, elasped_cycle);
        gpu.borrow_mut().step(
            elasped_cycle,
            &mut bus,
            &mut |frame_buffer: &FrameBuffer| {
                window
                    .update_with_buffer(&frame_buffer.pixels, FRAME_WIDTH, FRAME_HEIGHT)
                    .unwrap();
            },
        );
        dma.borrow_mut().step(&mut bus);
        if count == 0xFFF {
            count = 0;
            input.borrow_mut().step(|| -> (u8, u8) {
                let (mut cross_input, mut ab_input) = (0, 0);
                for key in window.get_keys() {
                    match key {
                        Key::Up => cross_input |= 0x04,
                        Key::Down => cross_input |= 0x08,
                        Key::Left => cross_input |= 0x02,
                        Key::Right => cross_input |= 0x01,
                        Key::Z => ab_input |= 0x01,
                        Key::X => ab_input |= 0x02,
                        Key::A => ab_input |= 0x08,
                        Key::B => ab_input |= 0x04,
                        _ => {}
                    }
                }
                (cross_input, ab_input)
            });
        } else {
            count += 1;
        }
    }
}
