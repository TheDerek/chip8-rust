mod opcodes;

use std::time::{Duration, Instant};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::ops::Not;

// Where the program starts in memory
const TIME_STEP_SECONDS: f32 = 1f32/60f32;
const PROGRAM_LOC: usize = 0x200;
const FONTSET_LOC: u16 = 0x050;
const FONTSET: [u8; 5 * 16] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80  // F
];

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Pixel {
    ON,
    OFF
}

#[derive(Debug)]
pub enum KeyState {
    DOWN,
    UP
}

impl Not for Pixel {
    type Output = Pixel;

    fn not(self) -> Self::Output {
        match self {
            Pixel::ON => Pixel::OFF,
            Pixel::OFF => Pixel::ON
        }
    }
}

pub struct Emulator {
    opcode: u16,
    pub memory: [u8; 4096],
    registers: [u8; 16],
    index_register: u16,
    pub program_counter: usize,
    graphics: [Pixel; Emulator::SCREEN_SIZE],
    delay_timer: u8,
    sound_timer: u8,
    stack: [u16; 16],
    stack_pointer: usize,
    keys: HashMap<u8, KeyState>,
    number_of_keys_pressed: i32,
    last_key_pressed: u8,
    pub draw: bool,
    pub clear: bool,
    last_cycle_time: Option<Instant>,
    hz_counter: Duration
}

impl Emulator {
    pub const SCREEN_WIDTH: u16 = 64;
    pub const SCREEN_HEIGHT: u16 = 32;
    const SCREEN_SIZE: usize = (Emulator::SCREEN_WIDTH * Emulator::SCREEN_HEIGHT) as usize;

    pub fn new() -> Emulator {
        let mut emu = Emulator {
            opcode: 0,
            memory: [0; 4096],
            registers: [0; 16],
            index_register: 0,
            program_counter: PROGRAM_LOC,
            graphics: [Pixel::OFF; Emulator::SCREEN_SIZE],
            delay_timer: 0,
            sound_timer: 0,
            stack: [0; 16],
            stack_pointer: 0,
            keys: HashMap::new(),
            number_of_keys_pressed: 0,
            last_key_pressed: 0,
            draw: false,
            clear: false,
            last_cycle_time: None,
            hz_counter: Duration::new(0, 0)
        };

        // Insert all the keys as currently unpressed
        for i in 0x0..0x10 {
            emu.keys.insert(i, KeyState::UP);
        }

        // Install the fontset in memory
        for i in 0..FONTSET.len() {
            emu.memory[FONTSET_LOC as usize + i] = FONTSET[i];
        }

        emu
    }

    pub fn load(path: &str) -> Emulator {
        let mut file = File::open(path).unwrap();
        let mut buf = [0u8; 4096];

        file.read(&mut buf[512..]).unwrap();

        let mut emu = Emulator::new();
        emu.memory = buf;

        emu
    }

    pub fn get_pixel(&self, x: u16, y: u16) -> Pixel {
       self.graphics[((y * Emulator::SCREEN_WIDTH) + x) as usize]
    }

    fn handle_timers(&mut self, delta: Option<Duration>) {

        // Handle timers
        match self.last_cycle_time {
            // This is the first emulated cycle, nothing to do here
            None => (),
            Some(time) => {
                let delta = match delta {
                    Some(delta) => delta,
                    None => time.elapsed()
                };

                self.hz_counter += delta;

                if self.hz_counter > Duration::from_secs_f32(TIME_STEP_SECONDS) {
                    self.hz_counter = Duration::new(0, 0);

                    if self.delay_timer > 0 {
                        self.delay_timer -= 1;
                    }

                    if self.sound_timer > 0 {
                        self.sound_timer -= 1;

                        if self.sound_timer == 1 {
                            println!("Ping!");
                        }
                    }
                }
            }
        };

        self.last_cycle_time = Some(Instant::now());
    }

    /// Emulates a cycle of the emulator
    ///
    /// # Arguments
    ///
    /// * `delta` - The time since this emulator was last called in milliseconds,
    /// if not provided an internal timer will be used
    pub fn emulate_cycle(&mut self) {
        self.handle_timers(None);

        // Reset the drawing an clearing flags
        self.clear = false;
        self.draw = false;

        let opcode = self.get_opcode();
        let (instruction, value) = Emulator::deconstruct_opcode(opcode);

        //println!("{} {:X}", self.program_counter, opcode);

        let run = match instruction {
            0x0 => opcodes::system,
            0x1 => opcodes::goto,
            0x2 => opcodes::call_subroutine,
            0x3 => opcodes::skip_true,
            0x4 => opcodes::skip_false,
            0x5 => opcodes::skip_equals,
            0x6 => opcodes::set_register,
            0x7 => opcodes::add_to_register,
            0x8 => opcodes::maths_ops,
            0x9 => opcodes::skip_not_equals,
            0xA => opcodes::set_index_register,
            0xB => opcodes::goto_plus_register,
            0xC => opcodes::rand,
            0xD => opcodes::draw,
            0xE => opcodes::skip_pressed,
            0xF => opcodes::misc_opcodes,
            _   => panic!("Missed opcoded {:x}", opcode)
        };

        run(self, value);
    }

    pub fn set_key(&mut self, key: u8, state: KeyState) {
        match state {
            KeyState::UP => self.number_of_keys_pressed -= 1,
            KeyState::DOWN => {
                self.number_of_keys_pressed += 1;
                self.last_key_pressed = key;
            }
        };

        // Should prevent negative keys being pressed if a key was pressed
        // before the program was started
        if self.number_of_keys_pressed < 0 {
            self.number_of_keys_pressed = 0;
        }

        self.keys.insert(key, state);
    }

    pub fn get_key(&self, key: u8) -> KeyState {
        match self.keys.get(&key) {
            Some(KeyState::DOWN) => KeyState::DOWN,
            _ => KeyState::UP
        }
    }

    fn get_opcode(&self) -> u16 {
        (self.memory[self.program_counter] as u16) << 8
            | self.memory[self.program_counter + 1] as u16
    }

    fn deconstruct_opcode(opcode: u16) -> (u8, u16) {
        ((opcode >> 12) as u8, opcode & 0x0FFF)
    }
}

#[cfg(red)]
mod tests {
    use super::*;

    #[test]
    fn set_index_register() {
        let new_index_reg = 0x210;
        let mut emu = Emulator::new();
        emu.memory[emu.program_counter] = 0xA2;
        emu.memory[emu.program_counter + 1] = 0x10;

        let emu = emu.emulate_cycle();
        assert_eq!(emu.index_register, new_index_reg);
    }
}
