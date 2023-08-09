extern crate rustygb;
mod hardware;
use rustygb::Cartridge;

use std::env;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;

fn read_rom(rom_name: &str) -> Vec<u8> {
    let file = File::open(rom_name).expect("File Not Found");
    let buffer = BufReader::new(file);
    let mut rom: Vec<u8> = Vec::new();
    for byte_or_error in buffer.bytes() {
        let byte = byte_or_error.unwrap();
        rom.push(byte);
    }
    rom
}
fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        panic!("usage: rustygb [FileName]");
    }

    let hw = hardware::Hardware::new();

    let rom = read_rom(&args[1]);
    let cartridge = Cartridge::new(rom, vec![0; 0x8000]);

    rustygb::run(cartridge, hw);
}
