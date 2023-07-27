use rustygb::cpu::*;

use minifb::{Key, Window, WindowOptions};
use std::fs::File;
use std::io::BufReader;
use std::io::Read;

const IO_DIV: usize = 0x04;
const IO_LCDC: usize = 0x40;
const IO_SCY: usize = 0x42;
const IO_SCX: usize = 0x43;
const IO_LY: usize = 0x44;
const IO_BGP: usize = 0x47;
const IO_OBP0: usize = 0x48;
const IO_OBP1: usize = 0x49;
const IO_WY: usize = 0x4A;
const IO_WX: usize = 0x4B;
const IO_IF: usize = 0x0F;

const FRAME_WIDTH: usize = 144;
const FRAME_HEIGHT: usize = 160;

fn get_color(tile: u16, y_offset: u8, x_offset: u8, video_ram: &[u8; 8192]) -> u8 {
    let tile_data = tile * 16 + y_offset as u16 * 2;
    (video_ram[(tile_data + 1) as usize].wrapping_shr(x_offset as u32)) % 2 * 2
        + (video_ram[tile_data as usize].wrapping_shr(x_offset as u32))
}

fn read_rom(rom_name: &str) -> Vec<u8> {
    let mut file = File::open(rom_name).expect("File Not Found");
    let buffer = BufReader::new(file);
    let mut rom: Vec<u8> = Vec::new();
    for byte_or_error in buffer.bytes() {
        let byte = byte_or_error.unwrap();
        rom.push(byte);
    }
    rom
}

fn main() {
    let mut frame_buffer: Vec<u32> = vec![0; FRAME_WIDTH * FRAME_HEIGHT];
    let mut window = Window::new("test", FRAME_WIDTH, FRAME_HEIGHT, WindowOptions::default())
        .unwrap_or_else(|e| {
            panic!("{}", e);
        });

    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));
    window
        .update_with_buffer(&frame_buffer, FRAME_WIDTH, FRAME_HEIGHT)
        .unwrap();
    // while window.is_open() && !window.is_key_down(Key::Escape) {
    //     window
    //         .update_with_buffer(&frame_buffer, FRAME_WIDTH, FRAME_HEIGHT)
    //         .unwrap();
    // }

    let rom = read_rom("./roms/Pokemon - Red Version (E).gb");

    let palette: [u32; 12] = [
        0xFFFFFFFF, 0xFFFFA563, 0xFFFF0000, 0xFF000000, 0xFFFFFFFF, 0xFF8484FF, 0xFF3A3A94,
        0xFF000000, 0xFFFFFFFF, 0xFFFFA563, 0xFFFF0000, 0xFF000000,
    ];
    let mut cpu = CPU::new(rom, vec![0; 0x8000]);
    let mut ppu_dot = 0;

    cpu.bus.io[IO_DIV] = 0xAC;
    cpu.bus.io[IO_LCDC] = 0x91;
    loop {
        let elapsed_cycles = cpu.step();
        cpu.bus.io[IO_DIV] = cpu.bus.io[IO_DIV].wrapping_add((elapsed_cycles >> 8) as u8);
        for _ in 0..elapsed_cycles {
            let video_ram = &cpu.bus.video_ram;
            let io = &cpu.bus.io;
            let oam = &cpu.bus.oam;
            if io[IO_LCDC] & 0x80 != 0 {
                ppu_dot += 1;
                if ppu_dot == 456 {
                    if cpu.bus.io[IO_LY] < 144 {
                        for tmp in 0..160 {
                            let is_window: bool = io[IO_LCDC] & 32 != 0
                                && io[IO_LY] >= io[IO_WY]
                                && tmp >= io[IO_WX] - 7;
                            let x_offset = if is_window {
                                tmp - io[IO_WX] + 7
                            } else {
                                tmp + io[IO_SCX]
                            };
                            let y_offset = if is_window {
                                io[IO_LY] - io[IO_WY]
                            } else {
                                io[IO_LY] + io[IO_SCY]
                            };
                            let mut palette_index: usize = IO_BGP;
                            let tile =
                                video_ram[(((if io[IO_LCDC] & (if is_window { 64 } else { 8 }) != 0
                                {
                                    7 as u16
                                } else {
                                    6 as u16
                                }) << 10)
                                    + y_offset as u16 / 8 * 32
                                    + x_offset as u16 / 8)
                                    as usize] as u16;
                            let mut color = get_color(
                                if io[IO_LCDC] & 0x10 != 0 {
                                    tile
                                } else {
                                    256 + tile as i8 as i16 as u16
                                },
                                y_offset & 0x07,
                                7 - (x_offset & 7),
                                video_ram,
                            );
                            if io[IO_LCDC] & 0x02 != 0 {
                                for sprite in (0..160).step_by(4) {
                                    let sprite_x = tmp - oam[sprite + 1] + 8;
                                    let sprite_y = io[IO_LY] - oam[sprite] + 16;
                                    let sprite_color = get_color(
                                        oam[sprite + 2] as u16,
                                        sprite_y ^ (if oam[sprite + 3] & 64 != 0 { 7 } else { 0 }),
                                        sprite_x ^ (if oam[sprite + 3] & 32 != 0 { 0 } else { 7 }),
                                        video_ram,
                                    );
                                    if sprite_y < 8
                                        && sprite_y < 8
                                        && !(oam[sprite + 3] & 0x80 != 0 && color != 0)
                                        && sprite_color != 0
                                    {
                                        color = sprite_color;
                                        palette_index = if oam[sprite] & 0x10 != 0 {
                                            IO_OBP0
                                        } else {
                                            IO_OBP1
                                        };
                                    }
                                }
                                frame_buffer[(io[IO_LY] as usize)
                                    .wrapping_mul(160)
                                    .wrapping_add(tmp as usize)] =
                                    palette[((io[palette_index] >> (2 * color)) % 4) as usize
                                        + ((palette_index - IO_BGP) * 4 & 7)];
                            }
                        }
                    }
                    if cpu.bus.io[IO_LY] == 143 {
                        cpu.bus.io[IO_IF] |= 1;
                        window
                            .update_with_buffer(&frame_buffer, FRAME_WIDTH, FRAME_HEIGHT)
                            .unwrap();
                    }
                    cpu.bus.io[IO_LY] = (cpu.bus.io[IO_LY] + 1) % 154;
                    ppu_dot = 0;
                }
            } else {
                cpu.bus.io[IO_LY] = 0;
                ppu_dot = 0;
            }
        }
    }
}
