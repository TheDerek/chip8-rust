use crate::emulator::Emulator;

pub fn ident(emu: &Emulator, value: u16) -> Emulator {
    Emulator {
        ..*emu
    }
}

pub fn system(emu: &Emulator, value: u16) -> Emulator {
    match value {
        0xEE => Emulator {
            stack_pointer: emu.stack_pointer - 1,
            program_counter: (emu.stack[emu.stack_pointer] + 2) as usize,
            ..*emu
        },
        _ => ident(emu, value) //TODO: Implement
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
}


