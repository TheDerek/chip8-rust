mod emulator;
extern crate sdl2;

use crate::emulator::Emulator;

use sdl2::event::Event;
use sdl2::pixels;
use sdl2::keyboard::Keycode;
use std::time::Duration;
use sdl2::gfx::primitives::DrawRenderer;
use sdl2::pixels::Color;

const SCALE: u32 = 20;
const SCREEN_WIDTH: u32 = Emulator::SCREEN_WIDTH * SCALE;
const SCREEN_HEIGHT: u32 = Emulator::SCREEN_HEIGHT * SCALE;

fn main() -> Result<(), String> {
    let emu = Emulator::load("data/helloworld.bin");

    let white: Color = Color::RGB(255, 255, 255);
    let black: Color = Color::RGB(0, 0, 0);

    let sdl_context = sdl2::init()?;
    let video_subsys = sdl_context.video()?;
    let window = video_subsys.window("Derek's Chip8 Emulator", SCREEN_WIDTH, SCREEN_HEIGHT)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;

    canvas.set_draw_color(pixels::Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();

    let mut events = sdl_context.event_pump()?;

    'main: loop {
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        for event in events.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'main
                },
                _ => {}
            }
        }

        for y in 0..Emulator::SCREEN_HEIGHT {
            for x in 0..Emulator::SCREEN_WIDTH {
                let pixel = match emu.get_pixel(x, y) {
                    emulator::Pixel::ON => white,
                    emulator::Pixel::OFF => black
                };
                canvas.box_(
                    (x * SCALE) as i16,
                    (y * SCALE) as i16,
                    (x * SCALE + SCALE) as i16,
                    (y * SCALE + SCALE) as i16,
                    pixel
                ).unwrap();
            }
        }

        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }

    Ok(())
}
