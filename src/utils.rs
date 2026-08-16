use crate::constants::{NIBBLE_MASK, MEMORY_SIZE, FONTS};

use std::fs;
use sdl2::rect::Rect;

pub struct Bytes(pub u8, pub u8);

pub fn get_second_nible(byte: u8) -> u8 {
    byte & NIBBLE_MASK
}

pub fn load_rom(file_path: &str) -> Option<[u8; MEMORY_SIZE]> {
    let file_bytes = match fs::read(file_path) {
        Ok(bytes) => bytes,
        Err(_) => return None,
    };

    let mut index: usize = 0x200;
    let mut ram = [0; MEMORY_SIZE];

    for byte in &file_bytes {
        ram[index] = *byte;
        index = index+1;
    }

    let mut index: usize = 0x50;

    for font in FONTS {
        ram[index] = font;
        index = index+1;
    }

    Some(ram)
}

pub fn set_rect_coords(r: &mut Rect, x: i32, y: i32) {
    let x_offset = 0;
    let y_offset = 0;

    r.x = x_offset + x * 10; 
    r.y = y_offset + y * 10;
}
