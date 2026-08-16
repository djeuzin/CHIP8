use std::thread::sleep;
use std::time::{Duration, Instant};

extern crate sdl2;

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;

use rand::Rng;

use crate::constants::*;
use crate::context::CH8Context;
use crate::utils::{Bytes, get_second_nible, set_rect_coords};

pub struct Cpu {
    pc: u16,
    register_i: u16,
    stack: [u16; 16],
    stack_index: usize,
    delay_timer: u8,
    sound_timer: u8,
    registers: [u8; 16]
}

impl Cpu {
    pub fn init() -> Self {
        Self {
            pc: 0x200,
            register_i: 0,
            stack: [0; 16],
            stack_index: 0,
            delay_timer: 255,
            sound_timer: 255,
            registers: [0; 16]
        }
    }
}

fn logical_and_arithmetic_instructions(ctx: &mut CH8Context, n2: u8, n3: u8, n4: u8, super_chip: bool) {
    let x = usize::from(n2);
    let y = usize::from(n3);
    
    match n4 {
        0x0 => {
            ctx.cpu.registers[x] = ctx.cpu.registers[y];
        },
        0x1 => {
            ctx.cpu.registers[x] |= ctx.cpu.registers[y];
        },
        0x2 => {
            ctx.cpu.registers[x] &= ctx.cpu.registers[y];
        },
        0x3 => {
            ctx.cpu.registers[x] ^= ctx.cpu.registers[y];
        },
        0x4 => {
            let (count, overflow) = ctx.cpu.registers[x].overflowing_add(ctx.cpu.registers[y]);
            ctx.cpu.registers[x] = count;

            if overflow {
                ctx.cpu.registers[0xF] = 1;
            }
        },
        0x5 => {
            ctx.cpu.registers[x] -= ctx.cpu.registers[y];

            if ctx.cpu.registers[x] >= ctx.cpu.registers[y] {
                ctx.cpu.registers[0xF] = 1;
            } 
            else {
                ctx.cpu.registers[0xF] = 0;
            }
        },
        0x6 => {
            if !super_chip {
                ctx.cpu.registers[x] = ctx.cpu.registers[y];
            }

            ctx.cpu.registers[0xF] = ctx.cpu.registers[x] % 2;

            ctx.cpu.registers[x] = ctx.cpu.registers[x] >> 1;
        },
        0x7 => {
            ctx.cpu.registers[y] -= ctx.cpu.registers[x];

            if ctx.cpu.registers[y] >= ctx.cpu.registers[x] {
                ctx.cpu.registers[0xF] = 1;
            } 
            else {
                ctx.cpu.registers[0xF] = 0;
            }
        },
        0xE => {
            if !super_chip {
                ctx.cpu.registers[x] = ctx.cpu.registers[y];
            }

            ctx.cpu.registers[0xF] = (ctx.cpu.registers[x] << 7) % 2;

            ctx.cpu.registers[x] = ctx.cpu.registers[x] << 1;
        },
        _ => {
            println!("Invalid instruction.");
        }
    }
}

pub fn fetch(ram: &[u8], pc: &mut u16) -> Option<Bytes> {
    let i = usize::from(*pc);
    if i + 1 >= MEMORY_SIZE {
        return None
    }

    let b1 = ram[i];
    let b2 = ram[i + 1];

    *pc = pc.wrapping_add(2);
    Some(Bytes(b1, b2))
}

pub fn decode_execute(mut ctx: &mut CH8Context) -> bool {
    let Bytes(b1, b2) = ctx.bytes;

    let n1 = b1 >> 4;
    let n2 = get_second_nible(b1);
    let n3 = b2 >> 4;
    let n4 = get_second_nible(b2);

    match n1 {
        0x0 => {
            if n2 == 0x0 && n3 == 0xE {
                if n4 == 0x0 { 
                    ctx.display = [[false; 64]; 32];
                }
                else {
                    ctx.cpu.stack_index -= 1;
                    ctx.cpu.pc = ctx.cpu.stack[ctx.cpu.stack_index];
                }
            }
            else if n3 == 0x0 {
                ctx.cpu.pc -= 2;
            }
        },
        0x1 => {
            ctx.cpu.pc = (u16::from(n1) << 8) | u16::from(b2);
        },
        0x2 => {
            ctx.cpu.stack[ctx.cpu.stack_index] = ctx.cpu.pc;
            ctx.cpu.stack_index += 1;
            ctx.cpu.pc = (u16::from(n1) << 8) | u16::from(b2);
        },
        0x3 => {
            if ctx.cpu.registers[usize::from(n2)] == b2 {
                ctx.cpu.pc += 2;
            }
        },
        0x4 => {
            if ctx.cpu.registers[usize::from(n2)] != b2 {
                ctx.cpu.pc += 2;
            }            
        },
        0x5 => {
            if ctx.cpu.registers[usize::from(n2)] == ctx.cpu.registers[usize::from(n3)] {
                ctx.cpu.pc += 2;
            }
        },
        0x6 => {
            let x = usize::from(n2);
            ctx.cpu.registers[x] = b2;
        },
        0x7 => {
            let x = usize::from(n2);
            ctx.cpu.registers[x] += b2;
        },
        0x8 => {
            logical_and_arithmetic_instructions(&mut ctx, n2, n3, n4, false);
        },
        0x9 => {
            if ctx.cpu.registers[usize::from(n2)] != ctx.cpu.registers[usize::from(n3)] {
                ctx.cpu.pc += 2;
            }
        },
        0xA => {
            ctx.cpu.register_i = (u16::from(n2) << 8) | u16::from(b2);
        },
        0xB => {
            ctx.cpu.pc = (u16::from(n2) << 8) | u16::from(b2) + u16::from(ctx.cpu.registers[0x0]);
        },
        0xC => {
            let mut rng = rand::thread_rng();
            let n: u8 = rng.gen_range(0..b2);

            ctx.cpu.registers[usize::from(n2)] = n & b2;
        },
        0xD => {
            // DXYN draw
            let mut y = usize::from(ctx.cpu.registers[usize::from(n3)] % 32);
            
            ctx.cpu.registers[0xF] = 0;

            for i in 0..n4 {
                let sprite = ctx.ram[usize::from(ctx.cpu.register_i) + usize::from(i)];
                let mut x = usize::from(ctx.cpu.registers[usize::from(n2)] % 64);

                for j in (0..8).rev() {
                    let bit = (sprite >> j) % 2;

                    if bit == 1 {
                        if ctx.display[y][x] {
                            ctx.cpu.registers[0xF] = 1;
                            ctx.display[y][x] = false;
                        }

                        ctx.display[y][x] = true;
                    }

                    if x + 1 > 63 { break; }

                    x = x + 1;
                }

                if y + 1 > 31 { break; }

                y = y + 1;
            }
        },
        other => {
            println!("Invalid instruction {other}. Exiting");
            return false;
        }
    }

    return true;
}

pub fn run(mut ctx: &mut CH8Context, ips: u64) {
    let interval: Duration = Duration::from_millis(1000 / ips);
    let mut next_time = Instant::now() + interval;

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("rust-sdl2 demo", SCREEN_WIDTH, SCREEN_HEIGHT)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    let mut rect = Rect::new(0, 0, RECT_WIDTH, RECT_HEIGHT);

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump().unwrap();

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                _ => {}
            }
        }

        ctx.bytes = match fetch(&ctx.ram, &mut ctx.cpu.pc) {
            Some(bytes) => bytes,
            None => {
                println!("Coundn't fech bytes. Exiting.");
                break;
            }
        };

        let success = decode_execute(&mut ctx);

        if !success { break; }

        ctx.cpu.sound_timer = ctx.cpu.sound_timer.wrapping_sub(1);
        ctx.cpu.delay_timer = ctx.cpu.delay_timer.wrapping_sub(1);

        canvas.set_draw_color(Color::RGB(0,0,0));
        canvas.clear();
        canvas.set_draw_color(Color::RGB(255,255,255));
        for i in 0..32 {
            for j in 0..64 {
                let x: i32 = j as i32;
                let y: i32 = i as i32;

                if ctx.display[i][j] {
                    set_rect_coords(&mut rect, x, y);
                    let _ = canvas.fill_rect(rect);
                }
            }
        }

        sleep(next_time - Instant::now());
        next_time += interval;

        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
