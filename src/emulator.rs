use std::fs::File;
use std::io::Read;

// Where the program starts in memory
const PROGRAM_LOC: usize = 0x200;

pub struct Emulator {
    opcode: u16,
    memory: [u8; 4096],
    registers: [u8; 16],
    index_register: u16,
    program_counter: usize,
    graphics: [u8; 64 * 32],
    delay_timer: u8,
    sound_timer: u8,
    stack: [u16; 16],
    stack_pointer: usize,
}

impl Emulator {
    fn new() -> Emulator {
        Emulator {
            opcode: 0,
            memory: [0; 4096],
            registers: [0; 16],
            index_register: 0,
            program_counter: PROGRAM_LOC,
            graphics: [0; 64 * 32],
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
        emu
    }

    fn get_opcode(&self) -> u16 {
        (self.memory[self.program_counter] as u16) << 8
            | self.memory[self.program_counter + 1] as u16
    }

    pub fn emulate_cycle(&self) -> Emulator {
        let mut emu = Emulator {
            ..*self
        };

        let opcode = self.get_opcode();
        let (instruction, value) = Emulator::deconstruct_opcode(opcode);

        match instruction {
            0x0 => {
                match value {
                    0xEE => {
                        // Return from a subroutine
                        emu.stack_pointer -= 1;
                        emu.program_counter = (emu.stack[emu.stack_pointer] + 2) as usize
                    }
                    _ => println!("Missed!")
                }
            }
            0x1 => {
                // Jumps to address at `value`
                emu.program_counter = value.into();
            }
            0x2 => {
                // Calls Subroutine at `value`
                emu.stack[emu.stack_pointer] = emu.program_counter as u16;
                emu.stack_pointer += 1;
                emu.program_counter = value.into();
            }
            0xA => {
                emu.index_register = value;
                emu.program_counter += 2;
            }
            _ => println!("Missed!")
        }

        return emu;
    }

    fn deconstruct_opcode(opcode: u16) -> (u8, u16) {
        ((opcode >> 12) as u8, opcode & 0x0FFF)
    }
}

#[cfg(test)]
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
