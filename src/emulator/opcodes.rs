use crate::emulator::*;

/// Dose nothing but skip the next instruction. Used for instructions not implemnted yet
pub fn ident(emu: &Emulator, _: u16) -> Emulator {
    Emulator {
        program_counter: emu.program_counter + 2,
        ..*emu
    }
}

/// Manages the 0x0FFF opcodes
pub fn system(emu: &Emulator, value: u16) -> Emulator {
    match value {
        // Return from subroutine
        0x0EE => return_from_subroutine(emu, value),
        _ => ident(emu, value) //TODO: Implement
    }
}

fn return_from_subroutine(emu: &Emulator, _: u16) -> Emulator {
    // The new stack pointer that contains the location of the code we are
    // going to jump back to
    let stack_pointer = emu.stack_pointer -1;
    Emulator {
        stack_pointer,
        program_counter: (emu.stack[stack_pointer] + 2) as usize,
        ..*emu
    }
}

pub fn goto(emu: &Emulator, value: u16) -> Emulator {
    Emulator {
        program_counter: value.into(),
        ..*emu
    }
}

pub fn call_subroutine(emu: &Emulator, value: u16) -> Emulator {
    let mut emu = Emulator {
        ..*emu
    };

    emu.stack[emu.stack_pointer] = emu.program_counter as u16;
    emu.stack_pointer += 1;
    emu.program_counter = value.into();
    emu
}

/// Skips the next instruction if VX equals NN
pub fn skip_true(emu: &Emulator, value: u16) -> Emulator {
    let reg_loc = (value >> 8) as usize;
    let expected_reg_value = (value & 0x0FF) as u8;

    let should_skip = emu.registers[reg_loc] == expected_reg_value;
    let pc_delta = if should_skip { 4 } else { 2 };

    Emulator {
        program_counter: emu.program_counter + pc_delta,
        ..*emu
    }
}

pub fn set_register(emu: &Emulator, value: u16) -> Emulator {
    let reg_loc = (value >> 8) as usize;
    let new_reg_value = (value & 0x0FF) as u8;

    let mut emu = Emulator {
        program_counter: emu.program_counter + 2,
        ..*emu
    };

    emu.registers[reg_loc] = new_reg_value;
    emu
}

pub fn set_index_register(emu: &Emulator, value: u16) -> Emulator {
    Emulator {
        index_register: value,
        program_counter: emu.program_counter + 2,
        ..*emu
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

    /// Run a subroutine and then return from it, testing both call_subroutine and
    /// return from subroutine
    #[test]
    fn subroutine() {
        let subroutine_loc = 0x623;

        let mut emu = Emulator::new();
        assert_eq!(emu.program_counter, PROGRAM_LOC);

        // Jump to subroutine 0x123
        emu.memory[emu.program_counter] = 0x26;
        emu.memory[emu.program_counter + 1] = 0x23;

        // Clear the screen 
        emu.memory[subroutine_loc] = 0x00;
        emu.memory[subroutine_loc + 1] = 0xE0;

        // and then return from subroutine
        emu.memory[subroutine_loc + 2] = 0x00;
        emu.memory[subroutine_loc + 3] = 0xEE;

        for _ in 0..3 {
            emu = emu.emulate_cycle();
        }

        // Make sure we are on the second instruction
        assert_eq!(PROGRAM_LOC + 2, emu.program_counter);
    }


    /// Test that we can insert a value to a register and then compare against
    /// value to skip the instruction
    #[test]
    fn skip_true() {
        let mut emu = Emulator::new();
        let pc = emu.program_counter; 

        // Set register A to 33
        emu.memory[pc] = 0x6A;
        emu.memory[pc + 1] = 0x33;

        // Skip the next instruction if register A is 33
        emu.memory[pc + 2] = 0x3A;
        emu.memory[pc + 3] = 0x33;
        
        // Set register A to 66
        emu.memory[pc + 4] = 0x6A;
        emu.memory[pc + 5] = 0x66;

        for _ in 0..3 {
            emu = emu.emulate_cycle();
        }

        // We have moved 3 instructions + one skipped instruction = 4 * 2 = 8
        assert_eq!(pc + 8, emu.program_counter);

        // Make sure that the last instruction didn't execute and that register A
        // is still at the inital value we set
        assert_eq!(0x33, emu.registers[0xA]);
    }
}
