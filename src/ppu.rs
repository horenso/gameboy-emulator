use sdl2::{
    pixels::PixelFormatEnum,
    rect::Rect,
    render::{TextureCreator, WindowCanvas},
    video::WindowContext,
    Sdl,
};

use crate::proc::cpu::Cpu;
use crate::{bus::Bus, util::helper::is_bit_set};

// grid of 20x20 8x8 tiles with 3 color channels
const TILE_DATA_SIZE: usize = 20 * 20 * 8 * 8 * 3;

const COLORS: [(u8, u8, u8); 4] = [
    (0xFF, 0xFF, 0xFF),
    (0x80, 0x80, 0x80),
    (0x00, 0xFF, 0xFF),
    (0x00, 0x00, 0x00),
];

pub struct Ppu<'a> {
    sdl_context: &'a Sdl,
    canvas: WindowCanvas,
    texture_creator: TextureCreator<WindowContext>,
    tile_data: [u8; TILE_DATA_SIZE],
    bg_buffer: [u8; 160 * 144 * 3],
    // tile_data_canvas: WindowCanvas,
}

impl Ppu<'_> {
    pub fn new(sdl_context: &Sdl) -> Ppu {
        let video_subsystem = sdl_context.video().unwrap();

        let window = video_subsystem
            .window("Game Boy", 256 + 160, 256 + 160)
            // .position_centered()
            .build()
            .expect("could not initialize video subsystem");

        let canvas = window
            .into_canvas()
            .build()
            .expect("could not make a canvas");

        let texture_creator = canvas.texture_creator();

        // let tile_data_window = video_subsystem
        //     .window("tile_data", 160 * 4, 160 * 4)
        //     .build()
        //     .expect("could not initialize video subsystem");

        // let tile_data_canvas = tile_data_window
        //     .into_canvas()
        //     .build()
        //     .expect("could not make a canvas");

        // let tile_data_texture_creator = tile_data_canvas.texture_creator();

        Ppu {
            sdl_context,
            canvas,
            texture_creator,
            tile_data: [0x40; TILE_DATA_SIZE],
            bg_buffer: [0; 160 * 144 * 3],
            // tile_data_canvas,
        }
    }

    fn update_tile_data(&mut self, bus: &Bus, cpu: &Cpu, start_address: u16) {
        let mut addr = 0x8000;
        eprintln!("start_address {:x}", addr);
        for tile in 0..384 {
            let start_x = (tile % 20) * 8;
            let start_y = (tile / 20) * 8;
            draw_tile_into_texture(bus, cpu, &mut self.tile_data, addr, start_x, start_y);
            addr += 16;
        }
    }

    pub fn draw(&mut self, bus: &Bus, cpu: &Cpu) {
        let lcdc_control = bus.read(Option::Some(cpu), 0xFF40);

        let lcd_enabled = is_bit_set(lcdc_control, 7);
        let tile_map_area = is_bit_set(lcdc_control, 6);
        let window_enabled = is_bit_set(lcdc_control, 5);
        let (tile_data_start_addr, relative_addr_mode) = if is_bit_set(lcdc_control, 4) {
            (0x8800, true)
        } else {
            (0x8000, false)
        };
        let start_addr = if is_bit_set(lcdc_control, 3) {
            0x9C00
        } else {
            0x9800
        };

        self.update_tile_data(bus, cpu, tile_data_start_addr);

        eprintln!("start addr {:x}", start_addr);

        let mut texture = self
            .texture_creator
            .create_texture_streaming(PixelFormatEnum::RGB24, 160, 160)
            .expect("Couldn't create texture!");
        texture.update(None, &self.tile_data, 20 * 8 * 3).expect("");

        self.canvas.clear();

        for tile_number in 0..1024 {
            let addr = start_addr + tile_number;
            let tile_id = bus.read(Option::Some(cpu), addr) as i32;
            // let tile_id = if relative_addr_mode {
            //     tile_id_read + (-128)
            // } else {
            //     tile_id_read
            // } as i32;
            // bus.read(cpu, addr) as i32;
            let tile_x = (tile_id % 20) * 8;
            let tile_y = (tile_id / 20) * 8;

            let target_x = ((tile_number % 32) * 8) as i32;
            let target_y = ((tile_number / 32) * 8) as i32;

            self.canvas
                .copy(
                    &texture,
                    Some(Rect::new(tile_x, tile_y, 8, 8)),
                    Some(Rect::new(target_x, target_y, 8, 8)),
                )
                .expect("");
        }

        self.canvas
            .copy(&texture, None, Some(Rect::new(256, 0, 20 * 8, 20 * 8)))
            .expect("");
        self.canvas.present();
    }

    fn draw_background(&self, bus: &Bus, cpu: &Cpu) {
        let lcdc_control = bus.read(Option::Some(cpu), 0xFF40);

        let lcd_enabled = is_bit_set(lcdc_control, 7);
        let tile_map_area = is_bit_set(lcdc_control, 6);
        let window_enabled = is_bit_set(lcdc_control, 5);
        let (tile_data_start_addr, relative_addr_mode) = if is_bit_set(lcdc_control, 4) {
            (0x8000, false)
        } else {
            (0x8800, true)
        };
        let start_addr = if is_bit_set(lcdc_control, 3) {
            0x9C00
        } else {
            0x9800
        };

        let scroll_y = bus.read(Option::Some(cpu), 0xff42);
        let scroll_x = bus.read(Option::Some(cpu), 0xff43);
    }
}

fn draw_tile_into_texture(
    bus: &Bus,
    cpu: &Cpu,
    buffer: &mut [u8],
    addr: u16,
    start_x: i32,
    start_y: i32,
) {
    let mut addr = addr;
    for pixel_y in 0..8 {
        let byte1 = bus.read(Option::Some(cpu), addr);
        addr += 1;
        let byte2 = bus.read(Option::Some(cpu), addr);
        addr += 1;
        for shift in (0..8).rev() {
            let higher = ((byte1 >> shift) & 1) << 1;
            let lower = (byte2 >> shift) & 1;
            let color_id = higher | lower;
            let pos_x = (7 - shift + start_x) as usize;
            let pos_y = (pixel_y + start_y) as usize;
            let pos_buf = (pos_y * 160 + pos_x) * 3;

            (buffer[pos_buf], buffer[pos_buf + 1], buffer[pos_buf + 2]) = COLORS[color_id as usize];
        }
    }
}
