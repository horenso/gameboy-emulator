mod cpu;
mod memory;
mod util;

use clap::Parser;
use cpu::cpu_impl::Cpu;
use memory::bus::Bus;
use memory::cartridge::Cartridge;
use memory::ppu::Ppu;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::io;
use std::thread::sleep;
use std::time::{Duration, Instant};

const FREQUENCY_HZ: u64 = 4194304;
const CYCLES_IN_ONE_SIXTIETH_S: u64 = 70224;
const ONE_SIXTIETH_S: Duration = Duration::from_nanos(16_700_000);

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
    rom_path: std::path::PathBuf,
}

fn main() -> Result<(), String> {
    let args = Args::parse();

    let cartridge = Cartridge::load_from_file(args.rom_path.to_str().unwrap())?;
    cartridge.print_info();
    let print_cpu_debug = args.debug_print;
    let mut show_background = args.draw_background;

    let mut bus = Bus::new(cartridge);
    let mut cpu = Cpu::new(bus);

    let sdl_context = sdl2::init()?;
    let mut video = Ppu::new(&sdl_context);

    let mut event_pump = sdl_context.event_pump()?;
    let mut is_paused = false;

    'main_loop: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'main_loop,
                Event::KeyDown {
                    keycode: Some(key), ..
                } => match key {
                    Keycode::Escape => break 'main_loop,
                    Keycode::P => {
                        is_paused = false;
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

        let before_run = Instant::now();
        while cpu.cycles < CYCLES_IN_ONE_SIXTIETH_S {
            if print_cpu_debug {
                cpu.debug_print(&mut io::stdout());
            }
            cpu.fetch_and_execute();
        }
        is_paused = true;

        if show_background && cpu.bus.v_ram_dirty {
            let now = Instant::now();

            video.draw(&cpu);
            cpu.bus.v_ram_dirty = false;

            eprintln!("Drawing took: {:.2?}", now.elapsed());
        }

        let delta_time = before_run.elapsed();
        if delta_time < ONE_SIXTIETH_S {
            let time_to_sleep = ONE_SIXTIETH_S - delta_time;
            sleep(time_to_sleep);
            eprintln!("Done frame, slept: {} ms", time_to_sleep.as_millis());
        }
        cpu.cycles = 0;
    }
    Ok(())
}
