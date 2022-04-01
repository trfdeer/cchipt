use crate::emu::{CHARACTER_SPRITES, SCREEN_HEIGHT, SCREEN_WIDTH};

#[allow(non_snake_case)]
pub struct Chip8 {
    pub V: [u8; 16],            // Vx registers; 0 through F. VF is used as flag
    pub I: u16,                 // Index Register
    pub delay_timer: u8,        // Delay Timer
    pub sound_timer: u8,        // Sound Timer. Beeps when it reaches zero
    pub stack: [u16; 16],       // Stack for storing return addresses, when calling subroutines
    pub sp: u16,                // Stack Pointer
    pub pc: u16,                // Program Counter
    pub memory: [u8; 4096],     // 4KB RAM
    pub key_states: [bool; 16], // 16-key Keyboard
    pub gfx: [bool; 64 * 32],   // 64*32 Monochrome Display
    pub make_beep: bool,        // Flag to signal if a beep is needed
}

impl Chip8 {
    pub fn new() -> Self {
        let mut new_cpu = Self {
            V: [0u8; 16],
            I: 0,
            delay_timer: 0,
            sound_timer: 0,
            stack: [0u16; 16],
            sp: 0,
            pc: 0x200, // Execution starts at 0x200
            memory: [0u8; 4096],
            key_states: [false; 16],
            gfx: [false; 64 * 32],
            make_beep: false,
        };

        // Load charaters into memory for display
        new_cpu.memory[0x00..0x50].copy_from_slice(&CHARACTER_SPRITES);

        new_cpu
    }

    pub fn tick(&mut self) {
        self.execute_opcode();
        self.update_timers();
    }

    fn update_timers(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            if self.sound_timer == 1 {
                self.make_beep = true;
            }
            self.sound_timer -= 1;
        }
    }

    pub fn get_opcode(&self) -> u16 {
        u16::from_be_bytes([
            self.memory[self.pc as usize],
            self.memory[(self.pc + 1) as usize],
        ])
    }

    pub fn decode_instruction(opcode: &u16) -> String {
        match opcode & 0xF000 {
            0x0000 => match opcode & 0x000F {
                0x0000 => String::from("CLS"),
                0x000E => String::from("RET"),
                _ => String::default(),
            },
            0x1000 => {
                let nnn = opcode & 0x0FFF;
                format!("{:4} {nnn:03x}", "JP")
            }
            0x2000 => {
                let nnn = opcode & 0x0FFF;
                format!("{:4} {nnn:03x}", "CALL")
            }
            0x3000 => {
                let x = ((opcode & 0x0F00) >> 8) as u8;
                let kk = (opcode & 0x00FF) as u8;
                format!("{:4} V{x:X}, {kk:02x}", "SE")
            }
            0x4000 => {
                let x = ((opcode & 0x0F00) >> 8) as u8;
                let kk = (opcode & 0x00FF) as u8;
                format!("{:4} V{x:X}, {kk:02x}", "SNE")
            }
            0x5000 => {
                let x = ((opcode & 0x0F00) >> 8) as u8;
                let y = ((opcode & 0x00F0) >> 4) as u8;
                format!("{:4} V{x:X}, V{y:X}", "SE")
            }
            0x6000 => {
                let x = ((opcode & 0x0F00) >> 8) as u8;
                let kk = (opcode & 0x00FF) as u8;
                format!("{:4} V{x:X}, {kk:02x}", "LD")
            }
            0x7000 => {
                let x = ((opcode & 0x0F00) >> 8) as u8;
                let kk = (opcode & 0x00FF) as u8;
                format!("{:4} V{x:X}, {kk:02x}", "ADD")
            }
            0x8000 => {
                let x = ((opcode & 0x0F00) >> 8) as u8;
                let y = ((opcode & 0x00F0) >> 4) as u8;

                match opcode & 0x000F {
                    0x0000 => format!("{:4} V{x:X}, V{y:X}", "LD"),
                    0x0001 => format!("{:4} V{x:X}, V{y:X}", "OR"),
                    0x0002 => format!("{:4} V{x:X}, V{y:X}", "AND"),
                    0x0003 => format!("{:4} V{x:X}, V{y:X}", "XOR"),
                    0x0004 => format!("{:4} V{x:X}, V{y:X}", "ADD"),
                    0x0005 => format!("{:4} V{x:X}, V{y:X}", "SUB"),
                    0x0006 => format!("{:4} V{x:X}, V{y:X}", "SHR"),
                    0x0007 => format!("{:4} V{x:X}, V{y:X}", "SUBN"),
                    0x000E => format!("{:4} V{x:X}, V{y:X}", "SHL"),
                    _ => unreachable!(),
                }
            }
            0x9000 => {
                let x = ((opcode & 0x0F00) >> 8) as u8;
                let y = ((opcode & 0x00F0) >> 4) as u8;
                format!("{:4} V{x:X}, V{y:X}", "SNE")
            }
            0xA000 => {
                let nnn = opcode & 0x0FFF;
                format!("{:4} I, {nnn:03x}", "LD")
            }
            0xB000 => {
                let nnn = opcode & 0x0FFF;
                format!("{:4} V0, {nnn:03x}", "JP")
            }
            0xC000 => {
                let x = ((opcode & 0x0F00) >> 8) as u8;
                let kk = (opcode & 0x00FF) as u8;
                format!("{:4} V{x:X}, {kk:02x}", "RND")
            }
            0xD000 => {
                let x = ((opcode & 0x0F00) >> 8) as u8;
                let y = ((opcode & 0x00F0) >> 4) as u8;
                let n = (opcode & 0x000F) as u8;
                format!("{:4} V{x:X}, V{y:X}, {n:x}", "DRW")
            }
            0xE000 => {
                let x = ((opcode & 0x0F00) >> 8) as u8;
                match opcode & 0x000F {
                    0x000E => format!("{:4} V{x:X}", "SKP"),
                    0x0001 => format!("{:4} V{x:X}", "SKNP"),
                    _ => unreachable!(),
                }
            }
            0xF000 => {
                let x = ((opcode & 0x0F00) >> 8) as u8;
                match opcode & 0x00FF {
                    0x0007 => format!("{:4} V{x:X}, DT", "LD"),
                    0x000A => format!("{:4} V{x:X}, K", "LD"),
                    0x0015 => format!("{:4} DT, V{x:X}", "LD"),
                    0x0018 => format!("{:4} ST, V{x:X}", "LD"),
                    0x001E => format!("{:4} I, V{x:X}", "ADD"),
                    0x0029 => format!("{:4} F, V{x:X}", "LD"),
                    0x0033 => format!("{:4} B, V{x:X}", "LD"),
                    0x0055 => format!("{:4} [I], V{x:X}", "LD"),
                    0x0065 => format!("{:4} V{x:X}, [I]", "LD"),
                    _ => unreachable!(),
                }
            }
            _ => unreachable!(),
        }
    }

    fn execute_opcode(&mut self) {
        let opcode = self.get_opcode();
        match opcode & 0xF000 {
            0x0000 => match opcode & 0x000F {
                // 00E0 - CLS
                // Clear the display.
                0x0000 => {
                    self.gfx = [false; 64 * 32];
                    self.pc += 2;
                }
                // 1nnn - JP addr
                // Jump to location nnn.
                0x000E => {
                    self.sp -= 1;
                    self.pc = self.stack[self.sp as usize];
                    self.pc += 2;
                }
                // 0nnn - SYS addr (Not Implemented)
                // Jump to a machine code routine at nnn.
                _ => {}
            },
            // 1nnn - JP addr
            // Jump to location nnn.
            0x1000 => {
                let nnn = opcode & 0x0FFF;
                self.pc = nnn;
            }
            // 2nnn - CALL addr
            // Call subroutine at nnn.
            0x2000 => {
                let nnn = opcode & 0x0FFF;
                self.stack[self.sp as usize] = self.pc;
                self.sp += 1;
                self.pc = nnn;
            }
            // 3xkk - SE Vx, byte
            // Skip next instruction if Vx = kk.
            0x3000 => {
                let x = ((opcode & 0x0F00) >> 8) as u8;
                let kk = (opcode & 0x00FF) as u8;
                if self.V[x as usize] == kk {
                    self.pc += 2;
                }

                self.pc += 2;
            }
            // 4xkk - SNE Vx, byte
            // Skip next instruction if Vx != kk.
            0x4000 => {
                let x = ((opcode & 0x0F00) >> 8) as u8;
                let kk = (opcode & 0x00FF) as u8;
                if self.V[x as usize] != kk {
                    self.pc += 2;
                }

                self.pc += 2;
            }
            // 5xy0 - SE Vx, Vy
            // Skip next instruction if Vx = Vy.
            0x5000 => {
                let x = ((opcode & 0x0F00) >> 8) as u8;
                let y = ((opcode & 0x00F0) >> 4) as u8;
                if self.V[x as usize] == self.V[y as usize] {
                    self.pc += 2;
                }
                self.pc += 2;
            }
            // 6xkk - LD Vx, byte
            // Set Vx = kk.
            0x6000 => {
                let x = ((opcode & 0x0F00) >> 8) as u8;
                let kk = (opcode & 0x00FF) as u8;
                self.V[x as usize] = kk;
                self.pc += 2;
            }
            // 7xkk - ADD Vx, byte
            // Set Vx = Vx + kk.
            0x7000 => {
                let x = ((opcode & 0x0F00) >> 8) as u8;
                let kk = (opcode & 0x00FF) as u8;
                self.V[x as usize] = self.V[x as usize].wrapping_add(kk);
                self.pc += 2;
            }
            0x8000 => {
                let x = ((opcode & 0x0F00) >> 8) as u8;
                let y = ((opcode & 0x00F0) >> 4) as u8;

                match opcode & 0x000F {
                    // 8xy0 - LD Vx, Vy
                    // Set Vx = Vy.
                    0x0000 => {
                        self.V[x as usize] = self.V[y as usize];
                        self.pc += 2;
                    }
                    // 8xy1 - OR Vx, Vy
                    // Set Vx = Vx OR Vy.
                    0x0001 => {
                        self.V[x as usize] |= self.V[y as usize];
                        self.pc += 2;
                    }
                    // 8xy2 - AND Vx, Vy
                    // Set Vx = Vx AND Vy.
                    0x0002 => {
                        self.V[x as usize] &= self.V[y as usize];
                        self.pc += 2;
                    }
                    // 8xy3 - XOR Vx, Vy
                    // Set Vx = Vx XOR Vy.
                    0x0003 => {
                        self.V[x as usize] ^= self.V[y as usize];
                        self.pc += 2;
                    }
                    // 8xy4 - ADD Vx, Vy
                    // Set Vx = Vx + Vy, set VF = carry.
                    0x0004 => {
                        let vx = self.V[x as usize];
                        let vy = self.V[y as usize];
                        let (sum, carry) = vx.overflowing_add(vy);

                        self.V[x as usize] = sum;
                        self.V[0xF_usize] = if carry { 1 } else { 0 };
                        self.pc += 2;
                    }
                    // 8xy5 - SUB Vx, Vy
                    // Set Vx = Vx - Vy, set VF = NOT borrow.
                    0x0005 => {
                        let vx = self.V[x as usize];
                        let vy = self.V[y as usize];
                        let (diff, borrow) = vx.overflowing_sub(vy);

                        self.V[x as usize] = diff;
                        self.V[0xF_usize] = if borrow { 0 } else { 1 };
                        self.pc += 2;
                    }
                    // 8xy6 - SHR Vx {, Vy}
                    // Set Vx = Vx SHR 1.
                    0x0006 => {
                        let vx = self.V[x as usize];
                        // let vy = self.V[y as usize];

                        self.V[x as usize] >>= 1;
                        self.V[0xF_usize] = vx & 1;
                        self.pc += 2;
                    }
                    // 8xy7 - SUBN Vx, Vy
                    // Set Vx = Vy - Vx, set VF = NOT borrow.
                    0x0007 => {
                        let vx = self.V[x as usize];
                        let vy = self.V[y as usize];
                        let (diff, borrow) = vy.overflowing_sub(vx);

                        self.V[x as usize] = diff;
                        self.V[0xF_usize] = if borrow { 0 } else { 1 };
                        self.pc += 2;
                    }
                    // 8xyE - SHL Vx {, Vy}
                    // Set Vx = Vx SHL 1.
                    0x000E => {
                        let vx = self.V[x as usize];
                        // let vy = self.V[y as usize];

                        self.V[x as usize] <<= 1;
                        self.V[0xF_usize] = (vx >> 7) & 1;
                        self.pc += 2;
                    }
                    _ => unreachable!(),
                }
            }
            // 9xy0 - SNE Vx, Vy
            // Skip next instruction if Vx != Vy.
            0x9000 => {
                let x = ((opcode & 0x0F00) >> 8) as u8;
                let y = ((opcode & 0x00F0) >> 4) as u8;
                if self.V[x as usize] != self.V[y as usize] {
                    self.pc += 4;
                } else {
                    self.pc += 2;
                }
            }
            // Annn - LD I, addr
            // Set I = nnn.
            0xA000 => {
                let nnn = opcode & 0x0FFF;
                self.I = nnn;
                self.pc += 2;
            }
            // Bnnn - JP V0, addr
            // Jump to location nnn + V0.
            0xB000 => {
                let nnn = opcode & 0x0FFF;
                self.pc = self.V[0] as u16 + nnn;
            }
            // Cxkk - RND Vx, byte
            // Set Vx = random byte AND kk.
            0xC000 => {
                let x = ((opcode & 0x0F00) >> 8) as u8;
                let kk = (opcode & 0x00FF) as u8;

                let rand_byte = rand::random::<u8>();
                self.V[x as usize] = rand_byte & kk;
                self.pc += 2;
            }
            // Dxyn - DRW Vx, Vy, nibble
            // Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision.
            0xD000 => {
                let x = ((opcode & 0x0F00) >> 8) as usize;
                let vx = self.V[x as usize] as usize;

                let y = ((opcode & 0x00F0) >> 4) as usize;
                let vy = self.V[y as usize] as usize;

                let n = (opcode & 0x000F) as usize;
                let bytes = &self.memory[(self.I as usize)..(self.I as usize + n)];
                let mut collision = false;

                for (row, _) in bytes.iter().enumerate().take(n) {
                    let byte = bytes[row];
                    for col in 0..8 {
                        let index = ((row + vy) % SCREEN_HEIGHT as usize) * 64
                            + ((col + vx) % SCREEN_WIDTH as usize);
                        let cur_val = if self.gfx[index] { 1 } else { 0 };
                        let new_val = cur_val ^ ((byte & (0x80 >> col)) >> (7 - col));
                        if new_val == 0 && cur_val == 1 {
                            collision = true;
                        }
                        self.gfx[index] = new_val == 1;
                    }
                }
                self.V[0xF_usize] = if collision { 1 } else { 0 };

                self.pc += 2;
            }
            0xE000 => {
                let x = ((opcode & 0x0F00) >> 8) as u8;
                match opcode & 0x000F {
                    // Ex9E - SKP Vx
                    // Skip next instruction if key with the value of Vx is pressed.
                    0x000E => {
                        let vx = self.V[x as usize];
                        if self.key_states[vx as usize] {
                            self.pc += 2;
                        }
                        self.pc += 2;
                    }
                    // ExA1 - SKNP Vx
                    // Skip next instruction if key with the value of Vx is not pressed.
                    0x0001 => {
                        let vx = self.V[x as usize];
                        if !self.key_states[vx as usize] {
                            self.pc += 2;
                        }
                        self.pc += 2;
                    }
                    _ => unreachable!(),
                }
            }
            0xF000 => {
                let x = ((opcode & 0x0F00) >> 8) as u8;
                match opcode & 0x00FF {
                    // Fx07 - LD Vx, DT
                    // Set Vx = delay timer value.
                    0x0007 => {
                        self.V[x as usize] = self.delay_timer;
                        self.pc += 2;
                    }
                    // Fx0A - LD Vx, K
                    // Wait for a key press, store the value of the key in Vx.
                    0x000A => {
                        for (i, key) in self.key_states.iter().enumerate() {
                            if *key {
                                self.V[x as usize] = i as u8;
                                self.pc += 2;
                            }
                        }
                    }
                    // Fx15 - LD DT, Vx
                    // Set delay timer = Vx.
                    0x0015 => {
                        self.delay_timer = self.V[x as usize];
                        self.pc += 2;
                    }
                    // Fx18 - LD ST, Vx
                    // Set sound timer = Vx.
                    0x0018 => {
                        self.sound_timer = self.V[x as usize];
                        self.pc += 2;
                    }
                    // Fx1E - ADD I, Vx
                    // Set I = I + Vx.
                    0x001E => {
                        self.I += self.V[x as usize] as u16;
                        self.pc += 2;
                    }
                    // Fx29 - LD F, Vx
                    // Set I = location of sprite for digit Vx.
                    0x0029 => {
                        let vx = self.V[x as usize];
                        self.I = (vx * 0x5) as u16;
                        self.pc += 2;
                    }
                    // Fx33 - LD B, Vx
                    // Store BCD representation of Vx in memory locations I, I+1, and I+2.
                    0x0033 => {
                        let vx = self.V[x as usize];

                        self.memory[self.I as usize] = vx / 100;
                        self.memory[(self.I + 1) as usize] = (vx / 10) % 10;
                        self.memory[(self.I + 2) as usize] = vx % 10;
                        self.pc += 2;
                    }
                    // Fx55 - LD [I], Vx
                    // Store registers V0 through Vx in memory starting at location I.
                    0x0055 => {
                        for i in 0..=x as u16 {
                            self.memory[(self.I + i) as usize] = self.V[x as usize];
                        }
                        self.pc += 2;
                    }
                    // Fx65 - LD Vx, [I]
                    // Read registers V0 through Vx from memory starting at location I.
                    0x0065 => {
                        for i in 0..=x as u16 {
                            self.V[x as usize] = self.memory[(self.I + i) as usize];
                        }
                        self.pc += 2;
                    }
                    _ => unreachable!(),
                }
            }
            _ => unreachable!(),
        }
    }
}

impl Default for Chip8 {
    fn default() -> Self {
        Self::new()
    }
}
