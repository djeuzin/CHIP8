use std::fs;

const MEMORY_SIZE: usize = 4 * 1024;
const NIBBLE_MASK: u8 = 0b0000_1111;

const FONTS: [u8; 80] = [
            0xF0, 0x90, 0x90, 0x90, 0xF0, 
            0x20, 0x60, 0x20, 0x20, 0x70, 
            0xF0, 0x10, 0xF0, 0x80, 0xF0, 
            0xF0, 0x10, 0xF0, 0x10, 0xF0, 
            0x90, 0x90, 0xF0, 0x10, 0x10, 
            0xF0, 0x80, 0xF0, 0x10, 0xF0, 
            0xF0, 0x80, 0xF0, 0x90, 0xF0, 
            0xF0, 0x10, 0x20, 0x40, 0x40, 
            0xF0, 0x90, 0xF0, 0x90, 0xF0, 
            0xF0, 0x90, 0xF0, 0x10, 0xF0, 
            0xF0, 0x90, 0xF0, 0x90, 0x90, 
            0xE0, 0x90, 0xE0, 0x90, 0xE0, 
            0xF0, 0x80, 0x80, 0x80, 0xF0, 
            0xE0, 0x90, 0x90, 0x90, 0xE0, 
            0xF0, 0x80, 0xF0, 0x80, 0xF0, 
            0xF0, 0x80, 0xF0, 0x80, 0x80  
];

struct Bytes(u8, u8);

struct Cpu {
    pc: u16,
    register_i: u16,
    stack: [u16; 16],
    delay_timer: u8,
    sound_timer: u8,
    registers: [u8; 16],
}

impl Cpu {
    fn init() -> Self {
        Self {
            pc: 200,
            register_i: 0,
            stack: [0; 16],
            delay_timer: 255,
            sound_timer: 255,
            registers: [0; 16]
        }
    }
}

fn get_second_nible(b: u8) -> u8 {
    b & NIBBLE_MASK
}

fn fetch(ram: &[u8], pc: &mut u16) -> Option<Bytes> {
    let i = usize::from(*pc);
    if i + 1 >= MEMORY_SIZE {
        return None
    }

    let b1 = ram[i];
    let b2 = ram[i + 1];

    *pc = pc.wrapping_add(2);
    Some(Bytes(b1, b2))
}

fn decode_execute(cpu: &mut Cpu, ram: &mut [u8], display: &mut [[bool; 64]; 32], bytes: Bytes) {
    let Bytes(b1, b2) = bytes;

    let n1 = b1 >> 4;
    let n2 = get_second_nible(b1);
    let n3 = b2 >> 4;
    let n4 = get_second_nible(b2);

    match n1 {
        0x0 => {
            //Clear screen
        },
        0x1 => {
            cpu.pc = (u16::from(n1) << 8) + u16::from(b2);
        },
        0x6 => {
            let x = usize::from(n2);
            cpu.registers[x] = b2;
        },
        0x7 => {
            let x = usize::from(n2);
            cpu.registers[x] += b2;
        },
        0xA => {
            cpu.register_i = (u16::from(n1) << 8) + u16::from(b2);
        },
        0xD => {
            // DXYN draw
            let mut x = usize::from(cpu.registers[usize::from(n2)] % 64);
            let mut y = usize::from(cpu.registers[usize::from(n3)] % 32);
            
            cpu.registers[0xF] = 0;

            for _i in 0..n4 {
                let sprite = cpu.registers[usize::from(cpu.register_i) + usize::from(n4)];

                for j in (1..8).rev() {
                    let bit = (sprite << j) % 2;

                    if bit == 1 && display[x][y] {
                        display[x][y] = false;
                        cpu.registers[0xF] = 0;
                    }
                    else if bit == 1 && !display[x][y] {
                        display[x][y] = true;
                    }

                    if x + 1 > 63 { break; }

                    x = x + 1;
                }

                if y + 1 > 31 { break; }

                y = y + 1;
            }

        },
        _ => {
            println!("Invalid instruction. Exiting");
            return;
        }
    }
}

fn load_rom(file_path: &str, ram: &mut [u8]) -> usize {
    let file_bytes = match fs::read(file_path) {
        Ok(bytes) => bytes,
        Err(_) => return 0
    };

    let mut index: usize = 200;

    for byte in &file_bytes {
        ram[index] = *byte;
        index = index+1;
    }

    let mut index: usize = 0x50;

    for font in FONTS {
        ram[index] = font;
        index = index+1;
    }

    file_bytes.len()
}

fn draw(display: &[[bool; 64]; 32]) {
    for line in display {
        for item in line {
            if *item {
                print!("[x]");
            }
            else {
                print!("[ ]");
            }
        }
    }
}

fn clear_screen() {
    println!("\r\x1b[2J\r\x1b[H")
}

fn main() {
    let mut ram: [u8; MEMORY_SIZE] = [0; MEMORY_SIZE];
    let mut cpu = Cpu::init();
    let mut display: [[bool; 64]; 32] = [[false; 64]; 32];

    let bytes_read = load_rom("../../../Downloads/ibm.ch8", &mut ram);

    if bytes_read == 0 {
        println!("Failed to read file. Exiting.");
        return;
    }

    let mut i = 0;

    while i < 10 {
        clear_screen();
        draw(&display);
        
        i += 1;
    }

    return;

    loop {
        let bytes = match fetch(&ram, &mut cpu.pc) {
            Some(bytes) => bytes,
            None => {
                println!("Coundn't fech bytes. Exiting.");
                break;
            }
        };

        decode_execute(&mut cpu, &mut ram, &mut display, bytes);

        clear_screen();

        draw(&display);
    }
}
