use crate::bus::Bus;
use rand;
use rand::Rng;

pub const PROGRAM_START: u16 = 0x200;
const NUM_REGISTERS: usize = 16;
const STACK_CAPACITY: usize = 16;

pub struct Cpu {
    vx: [u8; NUM_REGISTERS],
    pc: u16,
    idx: u16,
    stack: Vec<u16>,
    rng: rand::rngs::ThreadRng,
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            vx: [0; NUM_REGISTERS],
            pc: PROGRAM_START,
            idx: 0,
            stack: Vec::with_capacity(STACK_CAPACITY),
            rng: rand::thread_rng(),
        }
    }

    pub fn exec_instruction(&mut self, bus: &mut Bus) {
        let hi = bus.read_byte(self.pc) as u16;
        let lo = bus.read_byte(self.pc + 1) as u16;
        let opcode = (hi << 8) | lo;

        // NNN : address, 12-bit value, the lowest 12 bits of the instruction
        // NN  : 8-bit constant
        // N   : 4-bit constant the lowest 4 bits of the instruction
        // X   : A 4-bit value, the lower 4 bits of the high byte of the instruction
        // Y   : A 4-bit value, the upper 4 bits of the low byte of the instruction
        // PC  : Program Counter
        //
        // https://en.wikipedia.org/wiki/CHIP-8#Opcode_table
        // http://devernay.free.fr/hacks/chip8/C8TECH10.HTM#3.0
        let nnn = opcode & 0x0FFF;
        let nn = (opcode & 0x0FF) as u8;
        let n = (opcode & 0x00F) as u8;
        let x = ((opcode & 0x0F00) >> 8) as u8;
        let y = ((opcode & 0x00F0) >> 4) as u8;

        let vx = self.read_reg(x);
        let vy = self.read_reg(y);

        match (opcode & 0xF000) >> 12 {
            0x0 => {
                match nn {
                    0xE0 => {
                        // 00E0 Clears display
                        // bus.clear_screen()
                        self.pc += 2;
                    }
                    0xEE => {
                        // 00EE return from subroutine
                        self.pc = self.stack.pop().unwrap();
                    }
                    _ => panic!("Unrecongnized 0x00** opcode {:X}:{:X}", self.pc, opcode),
                }
            }
            0x1 => {
                // 1NNN Jump to address NNN
                self.pc = nnn;
            }
            0x2 => {
                // 2NNN Call subroutine at NNN
                self.stack.push(self.pc + 2);
                self.pc = nnn;
            }
            0x3 => {
                // 3XNN Skip next instruction if vx eq nn
                if nn == vx {
                    self.pc += 4;
                } else {
                    self.pc += 2;
                }
            }
            0x4 => {
                // 4XNN Skip next instruction if vx neq nn
                if nn != vx {
                    self.pc += 4;
                } else {
                    self.pc += 2;
                }
            }
            0x5 => {
                // 5XY0 Skip next instruction if vx eq vy
                if vx != vy {
                    self.pc += 4;
                } else {
                    self.pc += 2;
                }
            }
            0x6 => {
                // 6XNN Set vx to nn
                self.load_reg(x, nn);
                self.pc += 2;
            }
            0x7 => {
                // 7XNN Adds nn to vx (carry flag is not changed)
                self.load_reg(x, vx.wrapping_add(nn));
                self.pc += 2;
            }
            0x8 => {
                match n {
                    0x0 => {
                        // 8XY0 Set vx to value of vy
                        self.load_reg(x, vy);
                    }
                    0x1 => {
                        // 8XY1 Set vx to vx OR vy
                        self.load_reg(x, vx | vy);
                    }
                    0x2 => {
                        // 8XY2 Set vx to vx AND vy
                        self.load_reg(x, vx & vy);
                    }
                    0x3 => {
                        // 8XY3 Set vx to vx XOR vy
                        self.load_reg(x, vx ^ vy);
                    }
                    0x4 => {
                        // 8XY4 Add vy to value of vx. VF carry bit set
                        let sum = vx as u16 + vy as u16;
                        self.load_reg(x, sum as u8);
                        if sum > 0xFF {
                            self.load_reg(0xF, 1);
                        } else {
                            self.load_reg(0xF, 0);
                        }
                    }
                    0x5 => {
                        // 8XY5 Sub vy from value of vx. VF 0 if borrow and 1 if not
                        self.load_reg(x, vx - vy);
                        if vy > vx {
                            self.load_reg(0xF, 1);
                        } else {
                            self.load_reg(0xF, 0);
                        }
                    }
                    0x6 => {
                        // 8XY6 Stores least sig bit of vx in vf and then shift vx right 1
                        self.load_reg(0xF, vx & 0x1);
                        self.load_reg(x, vx >> 1);
                    }
                    0x7 => {
                        // 8XY7 Set vx to vy - vx. vf 0 when borrow else 1
                        self.load_reg(x, vy - vx);
                        if vx > vy {
                            self.load_reg(0xF, 1);
                        } else {
                            self.load_reg(0xF, 0);
                        }
                    }
                    0xE => {
                        // 8XYE Stores most sig bit of vx in vf and then shift vx left 1
                        self.load_reg(0xF, (vx >> 8) >> 7);
                        self.load_reg(x, vx << 1);
                    }
                    _ => panic!("Unrecongnized 0x8XY* opcode {:X}:{:X}", self.pc, opcode),
                }

                self.pc += 2;
            }
            0x9 => {
                // 9XY0 Skip if vx = vy
                if vx != vy {
                    self.pc += 4;
                } else {
                    self.pc += 2;
                }
            }
            0xA => {
                // ANNN Set I to address NNN
                self.idx = nnn;
                self.pc += 2;
            }
            0xB => {
                // BNNN Jump to address NNN plus v0
                self.pc = self.read_reg(0) as u16 + nnn;
            }
            0xC => {
                // CXNN set vx to the result of bitwise operation of a random number
                let r: u8 = self.rng.gen_range(0, 255);
                self.load_reg(x, r);
                self.pc += 2;
            }
            0xD => todo!(),
            0xE => {
                match nn {
                    0x95 => {
                        // EX9E Skip next instruction if key in vx not pressed
                        todo!();
                    }
                    0xA1 => {
                        // EXA1 Skip next instruction if key in vx is pressed
                        todo!();
                    }
                    _ => panic!("Unrecongnized 0xEX** opcode {:X}:{:X}", self.pc, opcode),
                }
            }
            0xF => match nn {
                0x07 => {
                    todo!();
                }
                0x15 => {
                    todo!();
                }
                0x18 => {
                    todo!();
                }
                0x1E => {
                    todo!();
                }
                0x29 => {
                    todo!();
                }
                0x33 => {
                    todo!();
                }
                0x55 => {
                    todo!();
                }
                0x65 => {
                    todo!();
                }
                _ => panic!("Unrecongnized 0xFX** opcode {:X}:{:X}", self.pc, opcode),
            },
            _ => panic!("Unrecongnized opcode {:X}:{:X}", self.pc, opcode),
        }
    }

    pub fn read_reg(&self, index: u8) -> u8 {
        self.vx[index as usize]
    }

    pub fn load_reg(&mut self, index: u8, value: u8) {
        self.vx[index as usize] = value;
    }
}
