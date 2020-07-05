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
    stack_pointer: u16,
}

impl Emulator {
    fn load(path: &str) -> Emulator {
        let mut file = File::open(path).unwrap();
        let mut buf = [0u8; 4096];

        file.read(&mut buf[512..]).unwrap();

        Emulator {
            opcode: 0,
            memory: buf,
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

    fn get_opcode(&self) -> u16 {
        (self.memory[self.program_counter] as u16) << 8
            | self.memory[self.program_counter + 1] as u16
    }

    fn emulate_cycle(&self) -> Emulator {
        let opcode = self.get_opcode();
        Emulator {
            ..*self
        }
    }
}

fn main() {
    let emu = Emulator::load("data/helloworld.bin");
    println!("{:x?}", &emu.memory[PROGRAM_LOC..550]);
    println!("{:x?}", emu.get_opcode());
}
