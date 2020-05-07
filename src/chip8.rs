use crate::bus::Bus;
use crate::cpu::Cpu;
use crate::cpu::PROGRAM_START;
use anyhow::Result;
use std::fs;
use std::io::Read;
use std::path::Path;

pub struct Chip8 {
    bus: Bus,
    cpu: Cpu,
}

impl Chip8 {
    pub fn new() -> Self {
        Self {
            bus: Bus::new(),
            cpu: Cpu::new(),
        }
    }

    pub fn load_file<P: AsRef<Path>>(&mut self, filename: P) -> Result<()> {
        let metadata = fs::metadata(filename.as_ref())?;
        let mut file = fs::File::open(filename.as_ref())?;
        let mut buffer = Vec::with_capacity(metadata.len() as usize);
        file.read_to_end(&mut buffer)?;

        self.load(buffer);
        Ok(())
    }

    pub fn load(&mut self, buffer: Vec<u8>) {
        for (i, byte) in buffer.iter().enumerate() {
            self.bus.load_byte(PROGRAM_START + (i as u16), byte.clone());
        }
    }

    pub fn clock(&mut self) {
        self.cpu.exec_instruction(&mut self.bus);
    }
}
