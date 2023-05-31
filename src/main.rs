#![allow(dead_code)] // TODO: Obv delete this at some point

// TODO: Figure out how to properly structure Rust project.
mod bus;
mod cartridge;
mod cpu;
mod util;
mod video;

use bus::Bus;
use cartridge::Cartridge;
use clap::Parser;
use cpu::core::Cpu;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::io;
use std::thread::sleep;
use std::time::{Duration, Instant};
use video::Video;

#[derive(Parser)]
#[command(author)]
struct Args {
    /// Dump the cpu state after each instruction
    #[arg(short = 'd', long = "debug-print")]
    debug_print: bool,

    /// Draw background and tile data
    #[arg(short = 'b', long = "draw-bg", default_value_t = false)]
    draw_background: bool,

    /// The path to the rom
    rom_path: std::path::PathBuf,
}

fn main() -> Result<(), String> {
    let args = Args::parse();

    let cartridge = Cartridge::load_from_file(args.rom_path.to_str().unwrap())?;
    let print_cpu_debug = args.debug_print;
    let mut show_background = args.draw_background;

    let mut cpu = Cpu::new();
    let mut bus = Bus::new(cartridge);

    let sdl_context = sdl2::init()?;
    let mut video = Video::new(&sdl_context);

    let mut event_pump = sdl_context.event_pump()?;
    let mut is_paused = false;

    if print_cpu_debug {
        cpu.debug_print(&bus, &mut io::stdout());
    }
    'main_loop: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'main_loop,
                Event::KeyDown {
                    keycode: Some(key), ..
                } => match key {
                    Keycode::Escape => break 'main_loop,
                    Keycode::P => {
                        is_paused = !is_paused;
                        if is_paused {
                            println!("Paused!")
                        }
                    }
                    Keycode::D => show_background = !show_background,
                    _ => (),
                },
                _ => (),
            }
        }
        if is_paused {
            sleep(Duration::from_millis(100));
            continue;
        }

        cpu.fetch_and_execute(&mut bus);
        if print_cpu_debug {
            cpu.debug_print(&bus, &mut io::stdout());
        }

        if show_background && bus.v_ram_dirty {
            let now = Instant::now();

            video.draw(&bus, &cpu);
            bus.v_ram_dirty = false;

            let elapsed = now.elapsed();
            eprintln!("Elapsed: {:.2?}", elapsed);
        }

        // sleep(Duration::from_millis(5));
        // println!("Ticks: {}", cpu.counter);
    }
    Ok(())
}
