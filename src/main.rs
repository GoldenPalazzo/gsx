mod cpu;
mod memory;
use std::env;
use std::fs;
use std::io::{self, Write};

fn wait_for_enter() {
    let mut input = String::new();
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut input).unwrap();
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let args: Vec<String> = env::args().collect();
    let bios = std::fs::read("SCPH1001.BIN").expect("BIOS SCPH1001.BIN not found!");

    let mut cpu;
    let mut mem = crate::memory::MemoryBus::with_bios(&bios);
    if args.len() > 1 {
        let fpath = &args[1];
        let data = fs::read(fpath).expect("Can't read file");
        if &data[0..8] == b"PS-X EXE" {
            let (pc, r28, r29_30) = mem.load_psexe(&data);
            cpu = crate::cpu::Cpu::new(pc, r28, r29_30, r29_30);
        } else {
            cpu = crate::cpu::Cpu::default();
        }
    } else {
        cpu = crate::cpu::Cpu::default();
    }

    loop {
        // wait_for_enter();
        cpu.step(&mut mem);
    }
}
