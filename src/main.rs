mod constants;
mod context;
mod cpu;
mod utils;

use crate::context::CH8Context;
use crate::cpu::run;

use clap::Parser;
use std::path::Path;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "roms/ibm.ch8")]
    path: String,
}

pub fn main() {
    let args = Args::parse();

    if !Path::new(&args.path).exists() { 
        println!("Invalid path to file. {}", args.path);
        return;
    }

    let mut ctx = CH8Context::init(&args.path);

    run(&mut ctx);
}