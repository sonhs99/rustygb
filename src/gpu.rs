use crate::device::IOHandler;
use crate::mmu::{MemoryBus, MemoryRead, MemoryWrite};

pub type RenderCallback = fn(&FrameBuffer);

pub const FRAME_WIDTH: usize = 160;
pub const FRAME_HEIGHT: usize = 144;

const BGP: usize = 0;
const OBP0: usize = 1;
const OBP1: usize = 2;

#[repr(C)]
#[derive(Clone, Copy)]
struct Sprite {
    y: u8,
    x: u8,
    tile_index: u8,
    attribute: u8,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct Tile {
    pixels: [u8; 16],
}

enum Pixel {
    Black,
    Dark,
    Bright,
    White,
}

pub struct FrameBuffer {
    pub pixels: [u32; FRAME_HEIGHT * FRAME_WIDTH],
}

pub struct GPU {
    tiles: [Tile; 384],
    tile_map: [u8; 2048],
    oam: [Sprite; 40],
    LCDC: u8,
    STAT: u8,
    LY: u8,
    LYC: u8,
    WY: u8,
    WX: u8,
    SCY: u8,
    SCX: u8,
    BGP: u8,
    OBP0: u8,
    OBP1: u8,
    ppu_dot: u16,

    palette: [[u32; 4]; 3],

    frame_buffer: FrameBuffer,
}

impl Tile {
    pub fn color(&self, x_offset: u8, y_offset: u8) -> u8 {
        let low = (self.pixels[(y_offset * 2) as usize].wrapping_shr(x_offset as u32)) & 0x01;
        let high = (self.pixels[(y_offset * 2 + 1) as usize].wrapping_shr(x_offset as u32)) & 0x01;
        high * 2 + low
    }
}

impl GPU {
    pub fn new() -> GPU {
        GPU {
            tiles: [Tile { pixels: [0; 16] }; 384],
            tile_map: [0; 2048],
            oam: [Sprite {
                x: 0,
                y: 0,
                tile_index: 0,
                attribute: 0,
            }; 40],
            LCDC: 0x91,
            STAT: 0,
            LY: 0,
            LYC: 0,
            WY: 0,
            WX: 0,
            SCY: 0,
            SCX: 0,
            BGP: 0,
            OBP0: 0,
            OBP1: 0,
            ppu_dot: 0,
            frame_buffer: FrameBuffer {
                pixels: [0; FRAME_HEIGHT * FRAME_WIDTH],
            },
            palette: [
                [0xFFFFFFFF, 0xFFFFA563, 0xFFFF0000, 0xFF000000],
                [0xFFFFFFFF, 0xFF8484FF, 0xFF3A3A94, 0xFF000000],
                [0xFFFFFFFF, 0xFFFFA563, 0xFFFF0000, 0xFF000000],
            ],
        }
    }
    pub fn step<T>(&mut self, elapsed_cycles: u16, bus: &mut MemoryBus, callback: &mut T)
    where
        T: FnMut(&FrameBuffer),
    {
        for _ in 0..elapsed_cycles {
            if self.LCDC & 0x80 != 0 {
                self.ppu_dot += 1;
                if self.ppu_dot == 456 {
                    if self.LY < 144 {
                        self.render();
                    }
                    if self.LY == 143 {
                        bus.set_if(bus.get_if() | 1);
                        callback(&self.frame_buffer);
                    }
                    self.LY = (self.LY + 1) % 154;
                    self.ppu_dot = 0;
                }
            } else {
                self.LY = 0;
                self.ppu_dot = 0;
            }
        }
    }

    pub fn render(&mut self) {
        for tmp in 0..160 {
            let is_window: bool = self.LCDC & 0x20 != 0 && self.LY >= self.WY && tmp >= self.WX - 7;
            let (x_offset, y_offset) = if is_window {
                (tmp + 7 - self.WX, self.LY - self.WY)
            } else {
                (tmp.wrapping_add(self.SCX), self.LY.wrapping_add(self.SCY))
            };
            let mut palette_index = BGP;
            let mut palette = self.BGP;
            let tile_offset = if self.LCDC & (if is_window { 0x40 } else { 0x08 }) != 0 {
                0x0400
            } else {
                0x0000
            };
            let tile_index = self.tile_map
                [(tile_offset + y_offset as u16 / 8 * 32 + x_offset as u16 / 8) as usize];
            let tile_index = if self.LCDC & 0x10 != 0 {
                tile_index as u16
            } else {
                (256 + tile_index as i8 as i16) as u16
            };
            let tile = self.tiles[tile_index as usize];
            let mut color = tile.color(7 - (x_offset & 0x07), y_offset & 0x07);
            if self.LCDC & 0x02 != 0 {
                for sprite in &self.oam {
                    let sprite_x = tmp.wrapping_sub(sprite.x).wrapping_add(8);
                    let sprite_y = self.LY.wrapping_sub(sprite.y).wrapping_add(16);
                    let sprite_color = self.tiles[sprite.tile_index as usize].color(
                        sprite_x ^ (if sprite.attribute & 0x20 != 0 { 0 } else { 7 }),
                        sprite_y ^ (if sprite.attribute & 0x40 != 0 { 7 } else { 0 }),
                    );
                    if sprite_x < 8
                        && sprite_y < 8
                        && !(sprite.attribute & 0x80 != 0 && color != 0)
                        && sprite_color != 0
                    {
                        color = sprite_color;
                        (palette_index, palette) = if sprite.attribute & 0x10 != 0 {
                            (OBP1, self.OBP1)
                        } else {
                            (OBP0, self.OBP0)
                        };
                    }
                }
            }
            self.frame_buffer.pixels[(self.LY as usize)
                .wrapping_mul(160)
                .wrapping_add(tmp as usize)] = self.palette[palette_index]
                [((palette.wrapping_shr(2 * color as u32)) % 4) as usize];
        }
    }
}

impl IOHandler for GPU {
    fn read(&mut self, mmu: &MemoryBus, address: u16) -> MemoryRead {
        match address {
            0x8000..=0x97FF => unsafe {
                MemoryRead::Value(
                    *(&self.tiles[0] as *const Tile as *const u8)
                        .offset((address & 0x1FFF) as isize),
                )
            },
            0x9800..=0x9FFF => MemoryRead::Value(self.tile_map[(address - 0x9800) as usize]),
            0xFE00..=0xFE9F => unsafe {
                MemoryRead::Value(
                    *(&self.oam[0] as *const Sprite as *const u8)
                        .offset((address & 0x00FF) as isize),
                )
            },
            0xFF40 => MemoryRead::Value(self.LCDC),
            0xFF41 => MemoryRead::Value(self.STAT),
            0xFF42 => MemoryRead::Value(self.SCY),
            0xFF43 => MemoryRead::Value(self.SCX),
            0xFF44 => MemoryRead::Value(self.LY),
            0xFF45 => MemoryRead::Value(self.LYC),
            0xFF47 => MemoryRead::Value(self.BGP),
            0xFF48 => MemoryRead::Value(self.OBP0),
            0xFF49 => MemoryRead::Value(self.OBP1),
            0xFF4A => MemoryRead::Value(self.WY),
            0xFF4B => MemoryRead::Value(self.WX),
            _ => MemoryRead::PassThrough,
        }
    }
    fn write(&mut self, mmu: &MemoryBus, address: u16, value: u8) -> MemoryWrite {
        match address {
            0x8000..=0x97FF => unsafe {
                *(&mut self.tiles[0] as *mut Tile as *mut u8).offset((address & 0x1FFF) as isize) =
                    value
            },
            0x9800..=0x9FFF => self.tile_map[(address - 0x9800) as usize] = value,
            0xFE00..=0xFE9F => unsafe {
                *(&mut self.oam[0] as *mut Sprite as *mut u8).offset((address & 0x00FF) as isize) =
                    value
            },
            0xFF40 => self.LCDC = value,
            0xFF41 => self.STAT = value,
            0xFF42 => self.SCY = value,
            0xFF43 => self.SCX = value,
            0xFF44 => self.LY = value,
            0xFF45 => self.LYC = value,
            0xFF47 => self.BGP = value,
            0xFF48 => self.OBP0 = value,
            0xFF49 => self.OBP1 = value,
            0xFF4A => self.WY = value,
            0xFF4B => self.WX = value,
            _ => return MemoryWrite::PassThrough,
        }
        MemoryWrite::Value(value)
    }
}
