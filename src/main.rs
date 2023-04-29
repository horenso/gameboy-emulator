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
use sdl2::rect::{Point, Rect};
use sdl2::render::{Canvas, WindowCanvas};
use sdl2::sys::Window;
use std::env::args;
use std::fs::File;
use std::io;
use std::path::Path;
use std::thread::sleep;
use std::time::Duration;

fn main() -> Result<(), String> {
    let cartridge_path_arg = args()
        .nth(1)
        .unwrap_or("Expected one argument with the path to the cartridge.".to_string());
    let cartridge = Cartridge::load_from_file(cartridge_path_arg.clone())?;

    let bus = Bus::new(cartridge);
    let mut cpu = Cpu::new(bus);

    let base = Path::new(&cartridge_path_arg)
        .file_name()
        .unwrap()
        .to_str()
        .unwrap(); // TODO: Be better at Rust.

    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window("gameboy-emulator", 800, 800)
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

    let mut is_paused = false;
    let mut addr: u16 = 0x0000;

    cpu.debug_print(&mut io::stdout());
    'main_loop: loop {
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
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

        // 16 * 24

        // Executing cpu instructions
        cpu.fetch_and_execute();
        cpu.debug_print(&mut io::stdout());

        draw_tile(&cpu.bus, &mut canvas, &mut addr);
        canvas.present();

        sleep(Duration::from_millis(2000));
        println!("Ticks: {}", cpu.counter);
    }

    Ok(())
}

fn draw_tile(bus: &Bus, canvas: &mut WindowCanvas, addr: &mut u16) {
    let scale = 4;

    for tile in 0..384 {
        println!("next tile");
        let cols = (tile / 20) * 8;
        let rows = (tile % 20) * 8;
        for pixel_y in 0..8 {
            let byte1 = bus.read(*addr);
            println!("Byte1: {:04x} from {:04x}", byte1, addr);
            *addr += 1;
            let byte2 = bus.read(*addr);
            println!("Byte2: {:04x} from {:04x}", byte2, addr);
            *addr += 1;
            for shift in (0..8).rev() {
                let higher = ((byte1 >> shift) & 1) << 1;
                let lower = (byte2 >> shift) & 1;
                let color_id = higher | lower;
                let color = match color_id {
                    0 => Color::WHITE,
                    2 => Color::GRAY,
                    3 => Color::BLUE,
                    1 => Color::BLACK,
                    _ => unreachable!(),
                };

                canvas.set_draw_color(color);
                canvas
                    .fill_rect(Rect::new(
                        (7 - shift + (rows as i32)) * scale as i32,
                        (pixel_y + cols) as i32 * scale as i32,
                        scale as u32,
                        scale as u32,
                    ))
                    .unwrap();
            }
            canvas.set_draw_color(Color::RED);
            canvas
                .draw_rect(Rect::new(
                    rows as i32 * scale,
                    cols as i32 * scale,
                    8 * scale as u32,
                    8 * scale as u32,
                ))
                .unwrap();
        }
    }
}
