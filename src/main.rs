#![allow(dead_code)] // TODO: Obv delete this at some point

// TODO: Figure out how to properly structure Rust project.
mod bus;
mod cartridge;
mod cpu;
mod decode;
mod helper;
mod instruction;
mod registers;

use bus::Bus;
use cartridge::Cartridge;
use cpu::Cpu;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use std::env::args;
use std::fs::{create_dir, File};
use std::path::Path;
use std::rc::Rc;
use std::thread::sleep;
use std::time::Duration;

fn main() -> Result<(), String> {
    let cartridge_path_arg = args()
        .nth(1)
        .unwrap_or("Expected one argument with the path to the cartridge.".to_string());
    let cartridge = Cartridge::load_from_file(cartridge_path_arg.clone())?;

    let bus = Bus::new(cartridge);
    let mut cpu = Cpu::new(Rc::new(bus));

    let base = Path::new(&cartridge_path_arg)
        .file_name()
        .unwrap()
        .to_str()
        .unwrap(); // TODO: Be better at Rust.
    std::fs::create_dir_all("logs").unwrap();
    let mut debug_file = File::create(format!("logs/{}.log", base)).unwrap();

    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window("gameboy-emulator", 800, 600)
        .position_centered()
        .build()
        .expect("could not initialize video subsystem");

    let mut canvas = window
        .into_canvas()
        .build()
        .expect("could not make a canvas");

    canvas.set_draw_color(Color::RGB(0, 255, 255));
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump()?;
    'main_loop: loop {
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    break 'main_loop;
                }
                _ => (),
            }
        }
        canvas.present();

        // Executing cpu instructions
        cpu.fetch_and_execute();
        cpu.debug_print(&mut debug_file);

        // sleep(Duration::from_millis(400));
    }

    Ok(())
}
