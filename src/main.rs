use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use std::env::args;
use std::path::Path;
use std::thread::sleep;
use std::time::Duration;

fn main() -> Result<(), String> {
    let cartridge_path_arg = args()
        .nth(1)
        .expect("Expected one argument with the path to the cartridge.");
    let cartridge_path = Path::new(&cartridge_path_arg);

    if !cartridge_path.exists() {
        return Err(format!("Path {} does not exist!", cartridge_path.display()));
    }

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
                _ => {}
            }
        }
        canvas.present();
        sleep(Duration::from_millis(100));
    }

    Ok(())
}
