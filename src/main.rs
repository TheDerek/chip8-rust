mod emulator;

use crate::emulator::Emulator;

fn main() {
    let emu = Emulator::load("data/helloworld.bin");
    //println!("{:x?}", &emu.memory[PROGRAM_LOC..550]);
    //println!("{:x?}", emu.get_opcode());
    //println!("{:x?}", emu.get_opcode());

    // Run a single cycle
    //println!("Current opcode: {:x?}", emu.get_opcode());
    //println!("Current index register: {:x?}", emu.index_register);

    let emu = emu.emulate_cycle();

    //println!("Current opcode: {:x?}", emu.get_opcode());
    //println!("Current index register: {:x?}", emu.index_register);
}
