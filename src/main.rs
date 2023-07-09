use clap::{command, Parser};
use gameboy_emulator::start;
use std::path::PathBuf;

#[derive(Parser)]
#[command(author)]
struct Args {
    /// Dump the cpu state after each instruction
    #[arg(short = 'd', long = "debug-print")]
    debug_print: bool,

    /// Draw background and tile data
    #[arg(short = 'b', long = "draw-bg", default_value_t = true)]
    draw_background: bool,

    /// The path to the rom
    rom_path: PathBuf,
}

fn main() -> Result<(), String> {
    let args = Args::parse();
    start(args.debug_print, args.draw_background, args.rom_path)
}
