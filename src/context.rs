use crate::utils::{Bytes, load_rom};
use crate::cpu::Cpu;
use crate::constants::MEMORY_SIZE;

pub struct CH8Context {
    pub cpu: Cpu,
    pub ram: [u8; MEMORY_SIZE],
    pub display: [[bool; 64]; 32],
    pub bytes: Bytes
}

impl CH8Context {
    pub fn init(file_path: &str) -> Self {
        Self {
            cpu: Cpu::init(),
            ram: load_rom(file_path).unwrap(),
            display: [[false; 64]; 32],
            bytes: Bytes(0, 0)
        }
    }
}
