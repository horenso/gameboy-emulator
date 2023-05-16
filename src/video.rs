use sdl2::{pixels::Color, rect::Rect, render::WindowCanvas, Sdl};

use crate::bus::Bus;

pub struct Video<'a> {
    sdl_context: &'a Sdl,
    canvas: WindowCanvas,
    tile_data_canvas: WindowCanvas,
}

impl Video<'_> {
    pub fn new<'a>(sdl_context: &'a Sdl) -> Video<'a> {
        let video_subsystem = sdl_context.video().unwrap();

        let window = video_subsystem
            .window("game", 512, 512)
            // .position_centered()
            .build()
            .expect("could not initialize video subsystem");

        let canvas = window
            .into_canvas()
            .build()
            .expect("could not make a canvas");

        let tile_data_window = video_subsystem
            .window("tile_data", 512, 512)
            .build()
            .expect("could not initialize video subsystem");

        let tile_data_canvas = tile_data_window
            .into_canvas()
            .build()
            .expect("could not make a canvas");

        return Video {
            sdl_context,
            canvas,
            tile_data_canvas,
        };
    }

    pub fn draw(&mut self, bus: &Bus) {
        self.canvas.clear();
        let lcdc_control = bus.read(0xFF40);
        let start_addr = if lcdc_control & 8 == 0 {
            0x9800
        } else {
            0x9C00
        };
        let scale = 2;
        for tile_number in 0..1024 {
            let start_x = (tile_number % 32) * 8;
            let start_y = (tile_number / 32) * 8;
            let tile = (bus.read((start_addr + tile_number) as u16) as u16) * 16;
            draw_tile(
                bus,
                &mut self.canvas,
                0x8000 + (tile as u16),
                scale,
                start_x,
                start_y,
            );
        }
        self.canvas.present();
    }

    pub fn draw_tile_data(&mut self, bus: &Bus) {
        self.tile_data_canvas.clear();
        let mut addr = 0x8000;
        let scale = 2;
        for tile in 0..384 {
            let start_x = (tile % 20) * 8;
            let start_y = (tile / 20) * 8;
            draw_tile(
                bus,
                &mut self.tile_data_canvas,
                addr,
                scale,
                start_x,
                start_y,
            );
            addr += 16;
        }
        println!("end drawing at {:x}", addr);
        self.tile_data_canvas.present();
    }
}

fn draw_tile(
    bus: &Bus,
    canvas: &mut WindowCanvas,
    addr: u16,
    scale: i32,
    start_x: i32,
    start_y: i32,
) {
    let mut addr = addr;
    for pixel_y in 0..8 {
        let byte1 = bus.read(addr);
        addr += 1;
        let byte2 = bus.read(addr);
        addr += 1;
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
                    (7 - shift + (start_x as i32)) * scale as i32,
                    (pixel_y + start_y) as i32 * scale as i32,
                    scale as u32,
                    scale as u32,
                ))
                .unwrap();
        }
        // Draw tile outline:
        // canvas.set_draw_color(Color::RED);
        // canvas
        //     .draw_rect(Rect::new(
        //         start_x as i32 * scale,
        //         start_y as i32 * scale,
        //         8 * scale as u32,
        //         8 * scale as u32,
        //     ))
        //     .unwrap();
    }
}
