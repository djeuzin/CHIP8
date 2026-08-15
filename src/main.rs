use std::fs;
use std::process::Command;
use std::thread::sleep;
use std::time::{Duration, Instant};

const INTERVAL: Duration = Duration::from_millis(16);
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
            pc: 0x200,
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

fn decode_execute(cpu: &mut Cpu, ram: &mut [u8], display: &mut [[bool; 64]; 32], bytes: Bytes) -> bool {
    let Bytes(b1, b2) = bytes;

    let n1 = b1 >> 4;
    let n2 = get_second_nible(b1);
    let n3 = b2 >> 4;
    let n4 = get_second_nible(b2);

    match n1 {
        0x0 => {
            clear_screen();
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

            for i in 0..n4 {
                let sprite = ram[usize::from(cpu.register_i) + usize::from(n4)];
                x = usize::from(cpu.registers[usize::from(n2)] % 64);

                for j in (0..8).rev() {
                    let bit = (sprite << j) % 2;

                    if bit == 1 {
                        if display[x][y] {
                            cpu.registers[0xF] = 1;
                        }

                        display[x][y] = !display[x][y];
                    }

                    if x + 1 > 63 { break; }

                    x = x + 1;
                }

                if y + 1 > 31 { break; }

                y = y + 1;
            }

            draw(&display);
        },
        other => {
            println!("Invalid instruction {other}. Exiting");
            return false;
        }
    }

    return true;
}

fn load_rom(file_path: &str, ram: &mut [u8]) -> usize {
    let file_bytes = match fs::read(file_path) {
        Ok(bytes) => bytes,
        Err(_) => return 0
    };

    let mut index: usize = 0x200;

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
                print!("XXXXXX");
            }
            else {
                print!("O");
            }
        }
    }
}

fn clear_screen() {
    let output = Command::new("clear")
        .spawn()
        .expect("Failed to run command");

    let name = output.stdout;
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

    //for i in 0x200..MEMORY_SIZE {
    //    let a = ram[usize::from(i)];

    //    let n2 = get_second_nible(a);
    //    let n1 = a >> 4;

    //    println!("{} - N1: {n1:#x}, N2: {n2:#x})", (i-0x200));
    //}

    //return;

    let mut next_time = Instant::now() + INTERVAL;
    draw(&display);

    loop {
        let bytes = match fetch(&ram, &mut cpu.pc) {
            Some(bytes) => bytes,
            None => {
                println!("Coundn't fech bytes. Exiting.");
                break;
            }
        };

        let success = decode_execute(&mut cpu, &mut ram, &mut display, bytes);

        if !success { break; }

        //println!("PC: {:#x}", cpu.pc);
        //println!("I register: {:#x}", cpu.register_i);
        //let i: usize = 0;
        //while i < 16 {
        //    print!("V{:#x}: {:#x} ", i, cpu.registers[i]);
        //}
        //println!("");

        cpu.sound_timer = cpu.sound_timer.wrapping_sub(1);
        cpu.delay_timer = cpu.delay_timer.wrapping_sub(1);

        sleep(next_time - Instant::now());
        next_time += INTERVAL;
    }
}
