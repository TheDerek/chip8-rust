mod emulator;
extern crate sdl2;

use crate::emulator::Emulator;

use sdl2::event::Event;
use sdl2::pixels;
use sdl2::keyboard::Keycode;
use std::time::Duration;
use sdl2::gfx::primitives::DrawRenderer;
use sdl2::pixels::Color;
use std::collections::HashMap;

const SCALE: u16 = 20;
const SCREEN_WIDTH: u16 = Emulator::SCREEN_WIDTH * SCALE;
const SCREEN_HEIGHT: u16 = Emulator::SCREEN_HEIGHT * SCALE;
static KEY_MAP: &'static [Keycode] = &[
    Keycode::X,
    Keycode::Num1,
    Keycode::Num2,
    Keycode::Num3,
    Keycode::Q,
    Keycode::W,
    Keycode::E,
    Keycode::A,
    Keycode::S,
    Keycode::D,
    Keycode::Z,
    Keycode::C,
    Keycode::Num4,
    Keycode::R,
    Keycode::F,
    Keycode::V,
];

fn emu_keypress(emu: &mut Emulator, keycode: Keycode, state: emulator::KeyState) {
    let key: Option<u8> = match keycode {
        Keycode::Num1 => Some(0x1),
        Keycode::Num2 => Some(0x2),
        Keycode::Num3 => Some(0x3),
        Keycode::Num4 => Some(0xC),
        Keycode::Q => Some(0x4),
        Keycode::W => Some(0x5),
        Keycode::E => Some(0x6),
        Keycode::R => Some(0xD),
        Keycode::A => Some(0x7),
        Keycode::S => Some(0x8),
        Keycode::D => Some(0x9),
        Keycode::F => Some(0xE),
        Keycode::Z => Some(0xA),
        Keycode::X => Some(0x0),
        Keycode::C => Some(0xB),
        Keycode::V => Some(0xF),
        _ => None
    };

    match key {
        Some(key) => {
            emu.set_key(key, state);
        },
        None => ()
    }
}

/// Write emulator info to the terminal
fn write_emu_info(emu: &mut Emulator) {
    // Clear any existing stuff
    print!("{esc}[2J{esc}[1;1H", esc = 27 as char);

    // Write the state of each input
    println!("Input\n-----\n");
    for (value, key) in KEY_MAP.iter().enumerate() {
        println!("Key: [{}]\tValue: {:X}\tState: {:?}", key, value, emu.get_key(value as u8));
    }
}

fn main() -> Result<(), String> {
    //let mut emu = load_emu();
    let mut emu = Emulator::load("./data/pong.ch8");

    let white: Color = Color::RGB(255, 255, 255);
    let black: Color = Color::RGB(0, 0, 0);

    let sdl_context = sdl2::init()?;
    let video_subsys = sdl_context.video()?;
    let window = video_subsys.window("Derek's Chip8 Emulator", SCREEN_WIDTH.into(), SCREEN_HEIGHT.into())
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
        emu.emulate_cycle();

        canvas.clear();

        for event in events.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'main
                },
                // Send the rest of the keypresses to the emulator
                Event::KeyDown { keycode: Some(keycode), .. } => emu_keypress(
                    &mut emu,
                    keycode,
                    emulator::KeyState::DOWN
                ),
                Event::KeyUp { keycode: Some(keycode), .. } => emu_keypress(
                    &mut emu,
                    keycode,
                    emulator::KeyState::UP
                ),
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

        // Write debugging info to the terminal
        write_emu_info(&mut emu);

        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 120));
    }

    Ok(())
}
