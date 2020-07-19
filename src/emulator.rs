mod opcodes;

use std::fs::File;
use std::io::Read;

// Where the program starts in memory
const PROGRAM_LOC: usize = 0x200;

#[derive(Copy, Clone)]
pub enum Pixel {
    ON,
    OFF
}

pub struct Emulator {
    opcode: u16,
    memory: [u8; 4096],
    registers: [u8; 16],
    index_register: u16,
    program_counter: usize,
    graphics: [Pixel; Emulator::SCREEN_SIZE],
    delay_timer: u8,
    sound_timer: u8,
    stack: [u16; 16],
    stack_pointer: usize,
}

impl Emulator {
    pub const SCREEN_WIDTH: u32 = 64;
    pub const SCREEN_HEIGHT: u32 = 32;
    const SCREEN_SIZE: usize = (Emulator::SCREEN_WIDTH * Emulator::SCREEN_HEIGHT) as usize;

    fn new() -> Emulator {
        Emulator {
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
        }
    }

    pub fn load(path: &str) -> Emulator {
        let mut file = File::open(path).unwrap();
        let mut buf = [0u8; 4096];

        file.read(&mut buf[512..]).unwrap();

        let mut emu = Emulator::new();
        emu.memory = buf;

        for i in 0..Emulator::SCREEN_SIZE {
            if i % 3 != 0 {
                continue;
            }

            emu.graphics[i] = Pixel::ON;
        }

        emu
    }

    pub fn get_pixel(&self, x: u32, y: u32) -> Pixel {
       self.graphics[((y * Emulator::SCREEN_WIDTH) + x) as usize]
    }

    fn get_opcode(&self) -> u16 {
        (self.memory[self.program_counter] as u16) << 8
            | self.memory[self.program_counter + 1] as u16
    }

    pub fn emulate_cycle(&self) -> Emulator {
        let opcode = self.get_opcode();
        let (instruction, value) = Emulator::deconstruct_opcode(opcode);

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
            _   => opcodes::ident
        };

        return run(&self, value)
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
