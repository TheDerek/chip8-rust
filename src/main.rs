mod emulator;
extern crate sdl2;

use crate::emulator::Emulator;

use sdl2::event::Event;
use sdl2::pixels;
use sdl2::keyboard::Keycode;
use std::time::Duration;
use sdl2::gfx::primitives::DrawRenderer;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use std::any::type_name;

const SCALE: u16 = 20;
const SCREEN_WIDTH: u16 = Emulator::SCREEN_WIDTH * SCALE;
const SCREEN_HEIGHT: u16 = Emulator::SCREEN_HEIGHT * SCALE;
const INPUT_PANEL_HEIGHT: u16 = 100;
const INPUT_KEY_WIDTH: i16 = 50;
const INPUT_KEY_MARGIN: i16 = 10;
const INPUT_KEY_HEIGHT: i16 = 40;

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

struct TextRenderer<'a> {
    font: sdl2::ttf::Font<'a, 'a>,
    texture_creator: sdl2::render::TextureCreator<sdl2::video::WindowContext>,
    canvas: &'a mut sdl2::render::Canvas<sdl2::video::Window>,
//    ttf_context: sdl2::ttf::Sdl2TtfContext,
}

impl<'a> TextRenderer<'a> {
    fn new(path: &str, canvas: &'a mut sdl2::render::Canvas<sdl2::video::Window>) -> TextRenderer<'a> {
        let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string()).unwrap();
        let font = ttf_context.load_font(path, 128).unwrap();
        let texture_creator = canvas.texture_creator();

        TextRenderer {
            canvas: canvas,
            font: font,
            texture_creator: texture_creator
//            ttf_context: ttf_context,
        }
    }

    fn render(&mut self, text: &str, rect: Rect) {
        let surface = self.font.render(text)
            .blended(Color::RGBA(255, 255, 255, 255)).map_err(|e| e.to_string()).unwrap();

        let texture = self.texture_creator.create_texture_from_surface(&surface)
            .map_err(|e| e.to_string()).unwrap();

        self.canvas.copy(&texture, None, rect).unwrap();
    }
}

fn main() -> Result<(), String> {
    //let mut emu = load_emu();
    let mut emu = Emulator::load("./data/space-invaders.ch8");

    let white: Color = Color::RGB(255, 255, 255);
    let black: Color = Color::RGB(0, 0, 0);
    let red: Color = Color::RGB(255, 0, 0);

    let sdl_context = sdl2::init()?;
    let video_subsys = sdl_context.video()?;
    let window = video_subsys.window("Derek's Chip8 Emulator", SCREEN_WIDTH.into(), (SCREEN_HEIGHT + INPUT_PANEL_HEIGHT).into())
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    let text_renderer = TextRenderer::new("./data/hack.ttf", &mut canvas);

    canvas.set_draw_color(black);
    canvas.clear();
    canvas.present();

    let mut events = sdl_context.event_pump()?;

    'main: loop {
        emu.emulate_cycle();

        canvas.clear();
        canvas.set_draw_color(black);

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
        
        // Draw the pixel lines
        for y in 0..Emulator::SCREEN_HEIGHT {
            canvas.line(
                0,
                (y * SCALE) as i16,
                SCREEN_WIDTH as i16,
                (y * SCALE) as i16,
                red
            ).unwrap();
        }



        for x in 0..Emulator::SCREEN_WIDTH {
            canvas.line(
                (x * SCALE) as i16,
                0,
                (x * SCALE) as i16,
                SCREEN_HEIGHT as i16,
                red
            ).unwrap();
        }

        // Draw the input information
        for i in 0..16 {
            let x = INPUT_KEY_WIDTH * i + i * INPUT_KEY_MARGIN + INPUT_KEY_MARGIN;
            let y = SCREEN_HEIGHT as i16 + INPUT_KEY_MARGIN;
            canvas.box_(
                x,
                y,
                x + INPUT_KEY_WIDTH,
                y + INPUT_KEY_HEIGHT,
                white
            ).unwrap();
        }

        text_renderer.render("Hello world", Rect::new(0, SCREEN_HEIGHT.into(), 100, 50));

//        let surface = font.render("Hello Rust!")
//            .blended(Color::RGBA(255, 255, 255, 255)).map_err(|e| e.to_string())?;
//        let texture = texture_creator.create_texture_from_surface(&surface)
//            .map_err(|e| e.to_string())?;
//        canvas.copy(&texture, None, Rect::new(0, SCREEN_HEIGHT.into(), 100, 50))?;


        canvas.set_draw_color(black);
        canvas.present();
        //::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 120));
    }

    Ok(())
}
