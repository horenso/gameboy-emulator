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
                } => {
                    break 'main_loop;
                }
                _ => (),
            }
        }

        // 16 * 24

        // Executing cpu instructions
        cpu.fetch_and_execute();
        cpu.debug_print(&mut io::stdout());

        draw_tiles(&cpu, &mut canvas);
        canvas.present();

        sleep(Duration::from_millis(500));
    }

    Ok(())
}

fn draw_tiles(cpu: &Cpu, canvas: &mut WindowCanvas) {
    let data = vec![
        0x3C, 0x7E, 0x42, 0x42, 0x42, 0x42, 0x42, 0x42, 0x7E, 0x5E, 0x7E, 0x0A, 0x7C, 0x56, 0x38,
        0x7C,
    ];

    let scale = 32;

    for tile_y in 0..8 {
        let byte1 = data[tile_y * 2];
        let byte2 = data[tile_y * 2 + 1];
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
                    (7 - shift) * scale,
                    (tile_y as i32) * scale,
                    scale as u32,
                    scale as u32,
                ))
                .unwrap();
        }
    }
}
