use color_eyre::Result;
use winit::event::VirtualKeyCode;

use crate::chip8::Chip8;

pub const SCREEN_WIDTH: u32 = 64;
pub const SCREEN_HEIGHT: u32 = 32;

pub const SCALE: u32 = 16;
pub const REFRESH_RATE: u64 = 60;

pub const WINDOW_HEIGHT: u32 = SCREEN_HEIGHT * SCALE;
pub const WINDOW_WIDTH: u32 = SCREEN_WIDTH * SCALE;

pub const CHARACTER_SPRITES: [u8; 0x50] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x08, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x08, 0x80, // F
];

pub const KEYS: [VirtualKeyCode; 16] = [
    VirtualKeyCode::Key0,
    VirtualKeyCode::Key1,
    VirtualKeyCode::Key2,
    VirtualKeyCode::Key3,
    VirtualKeyCode::Key4,
    VirtualKeyCode::Key5,
    VirtualKeyCode::Key6,
    VirtualKeyCode::Key7,
    VirtualKeyCode::Key8,
    VirtualKeyCode::Key9,
    VirtualKeyCode::A,
    VirtualKeyCode::B,
    VirtualKeyCode::C,
    VirtualKeyCode::D,
    VirtualKeyCode::E,
    VirtualKeyCode::F,
];

pub struct Emu {
    pub cpu: Chip8,
    pub run_steps: bool,
    pub clock_rate: u64,
}

impl Default for Emu {
    fn default() -> Self {
        Self {
            cpu: Default::default(),
            run_steps: true,
            clock_rate: 600,
        }
    }
}

impl Emu {
    pub fn update_keystates(&mut self, new_keystates: [bool; 16]) {
        self.cpu.key_states = new_keystates;
    }

    pub fn progress(&mut self) {
        self.cpu.tick();
        if self.cpu.make_beep {
            self.beep();
        }
    }

    pub fn draw(&self, frame: &mut [u8]) {
        for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
            let x = (i % WINDOW_WIDTH as usize) / 16;
            let y = (i / WINDOW_WIDTH as usize) / 16;

            let on = self.cpu.gfx[y * 64 + x];

            let rgba = if on {
                [0xff, 0xff, 0xff, 0xff]
            } else {
                [0x11, 0x11, 0x11, 0xff]
            };

            pixel.copy_from_slice(&rgba);
        }
    }

    pub fn beep(&mut self) {
        self.cpu.make_beep = false;
        println!("BEEP"); // TODO
    }

    pub fn load_rom(&mut self, path: &str) -> Result<()> {
        let rom_bytes = std::fs::read(path)?;
        self.cpu.memory[0x200..(0x200 + rom_bytes.len())].copy_from_slice(&rom_bytes);
        Ok(())
    }
}
