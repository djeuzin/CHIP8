mod constants;
mod context;
mod cpu;
mod utils;


use crate::context::CH8Context;
use crate::constants::FILE_PATH;
use crate::cpu::run;

pub fn main() {
    let mut ctx = CH8Context::init(FILE_PATH);

    run(&mut ctx);
}