mod cpu;
mod memory;
use std::io::{self, Write};

fn wait_for_enter() {
    let mut input = String::new();
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut input).unwrap();
}

fn main() {
    let mut cpu = crate::cpu::Cpu::default();
    let mut mem = crate::memory::MemoryBus::new();

    loop {
        wait_for_enter();
        cpu.step(&mut mem);
    }
}
