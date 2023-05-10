#![allow(dead_code)] // TODO: Obv delete this at some point

// TODO: Figure out how to properly structure Rust project.
mod bus;
mod cartridge;
mod cpu;
mod decode;
mod helper;
mod instruction;
mod registers;
mod video;

use bus::Bus;
use cartridge::Cartridge;
use cpu::Cpu;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::env::args;
use std::io;
use std::thread::sleep;
use std::time::Duration;
use video::Video;

fn main() -> Result<(), String> {
    let cartridge_path_arg = args()
        .nth(1)
        .unwrap_or("Expected one argument with the path to the cartridge.".to_string());
    let cartridge = Cartridge::load_from_file(cartridge_path_arg.clone())?;

    let mut bus = Bus::new(cartridge);
    let mut cpu = Cpu::new();

    let sdl_context = sdl2::init()?;
    let mut video = Video::new(&sdl_context);

    let mut event_pump = sdl_context.event_pump()?;
    let mut is_paused = false;

    cpu.debug_print(&bus, &mut io::stdout());
    'main_loop: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'main_loop,
                Event::KeyDown {
                    keycode: Some(Keycode::P),
                    ..
                } => is_paused = !is_paused,
                _ => (),
            }
        }
        if is_paused {
            sleep(Duration::from_millis(100));
            continue;
        }

        cpu.fetch_and_execute(&mut bus);
        cpu.debug_print(&bus, &mut io::stdout());
        video.draw(&mut bus);

        // sleep(Duration::from_millis(2000));
        println!("Ticks: {}", cpu.counter);
    }
    Ok(())
}
