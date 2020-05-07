use crate::ram::Ram;

pub struct Bus {
    ram: Ram,
    delay_timer: u8,
}

impl Bus {
    pub fn new() -> Self {
        Self {
            ram: Ram::new(),
            delay_timer: 0,
        }
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        self.ram.read_byte(address)
    }

    pub fn load_byte(&mut self, address: u16, value: u8) {
        self.ram.load_byte(address, value);
    }
}
