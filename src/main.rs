mod emulator;
extern crate sdl2;

use crate::emulator::Emulator;

use sdl2::event::Event;
use sdl2::pixels;
use sdl2::keyboard::Keycode;
use std::time::Duration;
use sdl2::gfx::primitives::DrawRenderer;
use sdl2::pixels::Color;

const SCALE: u16 = 20;
const SCREEN_WIDTH: u16 = Emulator::SCREEN_WIDTH * SCALE;
const SCREEN_HEIGHT: u16 = Emulator::SCREEN_HEIGHT * SCALE;

fn load_emu() -> Emulator {
    let mut emu = Emulator::new();
    let pc = emu.program_counter;
    emu.memory[pc + 0] = 0x60;
    emu.memory[pc + 1] = 0x20;

    emu.memory[pc + 2] = 0x61;
    emu.memory[pc + 3] = 0x10;

    emu.memory[pc + 4] = 0xA2;
    emu.memory[pc + 5] = 0x08;

    emu.memory[pc + 6] = 0xD0;
    emu.memory[pc + 7] = 0x13;

    emu.memory[pc + 8] = 0x3C;
    emu.memory[pc + 9] = 0xC3;
    emu.memory[pc + 10] = 0xFF;

    for _ in 0..4 {
        emu.emulate_cycle();
    }

    emu
}

fn emu_keypress(emu: &mut Emulator, keycode: Keycode, state: emulator::KeyState) {
    let key: Option<u8> = match keycode {
        Keycode::Num1 => Some(0x1),
        Keycode::Num2 => Some(0x2),
        Keycode::Num3 => Some(3),
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
            println!("{:?} = 0x{:x} key pressed {:?}", keycode, key, state);
            emu.set_key(key, state);
        },
        None => ()
    }
}

fn main() -> Result<(), String> {
    //let mut emu = load_emu();
    let mut emu = Emulator::load("./data/breakout.ch8");

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

        if emu.clear {
            canvas.clear();
        }

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

        if emu.draw {
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
        }

        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 120));
    }

    Ok(())
}
