use std::fs::File;
use std::io::Read;

// Where the program starts in memory
const PROGRAM_LOC: usize = 0x200;

struct Emulator {
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
    fn load(path: &str) -> Emulator {
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

    fn emulate_cycle(&self) -> Emulator {
        let mut emu = Emulator {
            ..*self
        };

        let opcode = self.get_opcode();
        let (instruction, value) = deconstruct_opcode(opcode);

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
}

fn deconstruct_opcode(opcode: u16) -> (u8, u16) {
    ((opcode >> 12) as u8, opcode & 0x0FFF)
}

fn main() {
    let emu = Emulator::load("data/helloworld.bin");
    println!("{:x?}", &emu.memory[PROGRAM_LOC..550]);
    println!("{:x?}", emu.get_opcode());
    println!("{:x?}", emu.get_opcode());

    // Run a single cycle
    println!("Current opcode: {:x?}", emu.get_opcode());
    println!("Current index register: {:x?}", emu.index_register);

    let emu = emu.emulate_cycle();

    println!("Current opcode: {:x?}", emu.get_opcode());
    println!("Current index register: {:x?}", emu.index_register);
}

#[cfg(test)]
mod tests {
    #[test]
    fn exploration() {
        assert_eq!(2 + 2, 4);
    }
}
