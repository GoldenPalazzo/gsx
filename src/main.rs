mod cpu;
mod memory;
use std::io::{self, Write};

fn wait_for_enter() {
    let mut input = String::new();
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut input).unwrap();
}

fn main() {
    let buf = vec![0u8; 0x2000];
    let cpu = crate::cpu::Cpu::new();

    loop {
        wait_for_enter();
        cpu.step(&buf);
    }
}
