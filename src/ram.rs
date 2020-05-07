const MEMORY_SIZE: usize = 1 * 1024;

pub struct Ram {
    mem: [u8; MEMORY_SIZE],
}

impl Ram {
    pub fn new() -> Self {
        Self {
            mem: [0; MEMORY_SIZE],
        }
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        self.mem[address as usize]
    }

    pub fn load_byte(&mut self, address: u16, value: u8) {
        self.mem[address as usize] = value;
    }
}
