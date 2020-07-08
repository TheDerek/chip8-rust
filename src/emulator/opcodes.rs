use crate::emulator::*;

/// Dose nothing but skip the next instruction. Used for instructions not implemnted yet
pub fn ident(emu: &Emulator, value: u16) -> Emulator {
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

pub fn return_from_subroutine(emu: &Emulator, value: u16) -> Emulator {
    // The new stack pointer that contains the location of the code we are
    // going to jump back to
    let stack_pointer = emu.stack_pointer -1;
    Emulator {
        stack_pointer,
        program_counter: (emu.stack[stack_pointer] + 2) as usize,
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

    #[test]
    /// Run a subroutine and then return from it, testing both call_subroutine and
    /// return from subroutine
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
}


