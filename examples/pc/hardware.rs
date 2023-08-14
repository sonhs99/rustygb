use minifb::{Key, Scale, Window, WindowOptions};
use rustygb::{FrameBuffer, Pixel};

pub struct Hardware {
    window: Window,
    cross_button: u8,
    ab_button: u8,
    count: i32,
}

impl Hardware {
    pub fn new() -> Hardware {
        let window = Window::new(
            "test",
            rustygb::FRAME_WIDTH,
            rustygb::FRAME_HEIGHT,
            WindowOptions {
                resize: false,
                scale: Scale::X4,
                ..WindowOptions::default()
            },
        )
        .unwrap_or_else(|e| {
            panic!("{}", e);
        });
        Hardware {
            window: window,
            cross_button: 0,
            ab_button: 0,
            count: 0,
        }
    }

    fn update_keys(&mut self) {
        self.cross_button = 0;
        self.ab_button = 0;
        for key in self.window.get_keys() {
            match key {
                Key::Up => self.cross_button |= 0x04,
                Key::Down => self.cross_button |= 0x08,
                Key::Left => self.cross_button |= 0x02,
                Key::Right => self.cross_button |= 0x01,
                Key::Z => self.ab_button |= 0x01,
                Key::X => self.ab_button |= 0x02,
                Key::A => self.ab_button |= 0x08,
                Key::B => self.ab_button |= 0x04,
                _ => {}
            }
        }
    }
}

impl rustygb::Hardware for Hardware {
    fn is_active(&mut self) -> bool {
        self.window.is_open()
    }

    fn draw_framebuffer(&mut self, frame_buffer: &FrameBuffer) {
        let mut frame = [0u32; rustygb::FRAME_WIDTH * rustygb::FRAME_HEIGHT];
        for idx in 0..rustygb::FRAME_WIDTH * rustygb::FRAME_HEIGHT {
            frame[idx] = match frame_buffer.pixels[idx] {
                Pixel::Black => 0xFF000000,
                Pixel::Dark => 0xFF555555,
                Pixel::Bright => 0xFFAAAAAA,
                Pixel::White => 0xFFFFFFFF,
            }
        }
        self.window
            .update_with_buffer(&frame, rustygb::FRAME_WIDTH, rustygb::FRAME_HEIGHT)
            .unwrap();
    }

    fn get_keys(&mut self) -> (u8, u8) {
        (self.cross_button, self.ab_button)
    }

    fn update(&mut self) {
        if self.count == 0xFF {
            self.count = 0;
            self.update_keys();
        } else {
            self.count += 1;
        }
    }
}
