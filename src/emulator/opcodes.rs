use crate::emulator::*;

/// Manages the 0x0FFF opcodes
pub fn system(emu: &mut Emulator, value: u16) {
    match value {
        // Return from subroutine
        0x0EE => return_from_subroutine(emu, value),
        0x0E0 => {
            emu.program_counter += 2;
            emu.graphics = [Pixel::OFF; Emulator::SCREEN_SIZE];
            emu.clear = true;
            println!("Clearing the screen!");
        },
        _ => panic!("No instruction for 0x0{:X}", value)
    };
}

fn return_from_subroutine(emu: &mut Emulator, _: u16) {
    // The new stack pointer that contains the location of the code we are
    // going to jump back to
    emu.stack_pointer -= 1;
    emu.program_counter = (emu.stack[emu.stack_pointer] + 2) as usize;
}

pub fn goto(emu: &mut Emulator, value: u16) {
    emu.program_counter = value.into();
}

pub fn call_subroutine(emu: &mut Emulator, value: u16) {
    emu.stack[emu.stack_pointer] = emu.program_counter as u16;
    emu.stack_pointer += 1;
    emu.program_counter = value.into();
}

/// Skips the next instruction if VX equals NN
pub fn skip_true(emu: &mut Emulator, value: u16) {
    let reg_loc = (value >> 8) as usize;
    let expected_reg_value = (value & 0x0FF) as u8;

    let should_skip = emu.registers[reg_loc] == expected_reg_value;
    let pc_delta = if should_skip { 4 } else { 2 };

    emu.program_counter = emu.program_counter + pc_delta;
}

/// Skips the next instruction if VX does not equal NN
pub fn skip_false(emu: &mut Emulator, value: u16) {
    let reg_loc = (value >> 8) as usize;
    let expected_reg_value = (value & 0x0FF) as u8;

    let should_skip = emu.registers[reg_loc] != expected_reg_value;
    let pc_delta = if should_skip { 4 } else { 2 };

    emu.program_counter = emu.program_counter + pc_delta;
}

/// Skips the next instruction if VX equals VY
pub fn skip_equals(emu: &mut Emulator, value: u16) {
    skip_condition(|x, y| { x == y })(emu, value);
}

/// Skips the next instruction if VX does not equal VY
pub fn skip_not_equals(emu: &mut Emulator, value: u16) {
    skip_condition(|x, y| { x != y })(emu, value);
}

pub fn skip_condition(condition: fn(u8, u8) -> bool) -> Box<dyn Fn(&mut Emulator, u16)> {
    Box::new(move |emu: &mut Emulator, value: u16| {
        // 5XY0
        let x = (value >> 8) as usize;
        let y = ((value & 0x0F0) >> 4) as usize;

        let mut pc_inc = 2;

        if condition(emu.registers[x], emu.registers[y]) {
            pc_inc += 2;
        }

        emu.program_counter = emu.program_counter + pc_inc;
    })
}

/// Instruction 6XNN, store the number NN in register VX
pub fn set_register(emu: &mut Emulator, value: u16) {
    let reg_loc = (value >> 8) as usize;
    let new_reg_value = (value & 0x0FF) as u8;


    emu.registers[reg_loc] = new_reg_value;
    emu.program_counter += 2;
}

pub fn add_to_register(emu: &mut Emulator, value: u16) {
    let reg_loc = (value >> 8) as usize;
    let reg_inc = (value & 0x0FF) as u8;

    emu.registers[reg_loc] = addition_carry(emu.registers[reg_loc], reg_inc).0;
    emu.program_counter += 2;
}

pub fn set_index_register(emu: &mut Emulator, value: u16) {
    emu.index_register = value;
    emu.program_counter += 2;
}

/// Various bitwise and mathmatical operations for 0x8***
pub fn maths_ops(emu: &mut Emulator, value: u16) {
    let secondary_instruction = (value & 0x00F) as u8;
    let ix = (value >> 8) as usize;
    let iy = ((value & 0x0F0) >> 4) as usize;

    let x = emu.registers[ix];
    let f = emu.registers[0xF];
    let y = emu.registers[iy];

    // These operations both set Vx and Vf
    let (x, f) = match secondary_instruction {
        0x0 => (y, f),
        0x1 => (x | y, f),
        0x2 => (x & y, f),
        0x3 => (x ^ y, f),
        0x4 => addition_carry(x, y),
        0x5 => minus_carry(x, y),
        0x6 => (x >> 1, x & 0b00000001),
        0x7 => minus_carry(y, x),
        0xE => (x << 1, x >> 7),
        _ => (x, f)
    };

    emu.registers[ix] = x;
    emu.registers[0xF] = f;
    emu.program_counter += 2;
}

/// Adds VY to VX. VF is set to 1 when there's a carry, and to 0 when there isn't.
fn addition_carry(x: u8, y: u8) -> (u8, u8) {
    let result: i16 = (x as i16) + (y as i16);

    if result > 255 {
        return ((result - 256) as u8, 1);
    }

    (result as u8, 0)
}

/// VY is subtracted from VX. VF is set to 0 when there's a borrow, and 1 when there isn't
fn minus_carry(x: u8, y: u8) -> (u8, u8) {
    let result: i16 = (x as i16) - (y as i16);

    if result < 0 {
        return ((256 + result) as u8, 0);
    }

    (result as u8, 1)
}

/// BNNN
pub fn goto_plus_register(emu: &mut Emulator, value: u16) {
    emu.program_counter = value as usize + emu.registers[0] as usize;
}

/// CXNN
pub fn rand(emu: &mut Emulator, value: u16) {
    let ix: usize = (value >> 8).into();
    let nn = (value & 0x0FF) as u8;

    emu.registers[ix] = rand::random::<u8>() & nn;
    emu.program_counter += 2;
}

/// DXYN
pub fn draw(emu: &mut Emulator, value: u16) {
    emu.program_counter += 2;
    emu.draw = true;

    // value = 0xXYN
    let x = value >> 8;
    let y = (value >> 4) & 0x0F;
    let x = emu.registers[x as usize] as u16;
    let y = emu.registers[y as usize] as u16;
    let w = 8;
    let h = value & 0x00F;
    let mut flipped: bool = false;

    println!("Drawing {:X} at x={}, y={}", emu.index_register, x, y);

    for yline in 0..h {
        // Each byte is a line
        let line = emu.memory[(emu.index_register + yline) as usize];

        // For every bit in the line
        for xline in 0..w {
            // Get the most significant bit and check if it is 1
            let pixel = match ((line >> xline) & 0x01) == 1 {
                true => Pixel::ON,
                false => Pixel::OFF
            };

            let did_flip = emu.set_pixel(x + (w - xline) - 2, y + yline, pixel);
            flipped = flipped || did_flip;
        }
    }

    emu.registers[0xF] = if flipped { 1 } else { 0 };
}

/// EX9E & EXA1
pub fn skip_pressed(emu: &mut Emulator, value: u16) {
    (match value & 0x0FF {
        0x9E => skip_if_pressed,
        0xA1 => skip_if_not_pressed,
        _ => panic!("No instruction for 0xE{:X}", value)
    })(emu, value);
}

fn skip_if_pressed(emu: &mut Emulator, value: u16) {
    let x = value >> 8;

    match emu.keys.get(&(x as u8)) {
        Some(KeyState::DOWN) => emu.program_counter += 4,
        _ => emu.program_counter += 2
    }
}

fn skip_if_not_pressed(emu: &mut Emulator, value: u16) {
    let x = value >> 8;

    match emu.keys.get(&(x as u8)) {
        Some(KeyState::UP) => emu.program_counter += 4,
        _ => emu.program_counter += 2
    }
}

/// Misc opcodes starting with F
pub fn misc_opcodes(emu: &mut Emulator, value: u16) {
    let x = value >> 8;
    let xi = x as usize;
    let instruction = (value & 0x0FF) as u8;

    match instruction {
        // Set VX to the value of the delay timer
        0x07 => emu.registers[xi] = emu.delay_timer,
        // Wait for any keypress and then store it in vx
        0x0A => {
            let mut c = emu.number_of_keys_pressed;
            loop {
                if emu.number_of_keys_pressed < c {
                    c = emu.number_of_keys_pressed;
                }

                else if emu.number_of_keys_pressed > c {
                    break;
                }
            }

            emu.registers[x as usize] = emu.last_key_pressed;
        },
        0x15 => emu.delay_timer = emu.registers[xi],
        0x18 => emu.sound_timer = emu.registers[xi],
        0x1E => emu.index_register += emu.registers[xi] as u16,
        0x29 => emu.index_register = FONTSET_LOC + 5 * x,
        0x33 => {
            let bcd = get_binary_coded_decimal(emu.registers[xi]);
            emu.memory[emu.index_register as usize + 0] = bcd.0;
            emu.memory[emu.index_register as usize + 1] = bcd.1;
            emu.memory[emu.index_register as usize + 2] = bcd.2;
        },
        0x55 => {
            for i in 0..(x+1) {
                emu.memory[(emu.index_register + i) as usize]
                    = emu.registers[i as usize];
            }
        },
        0x65 => {
            for i in 0..(x+1) {
                emu.registers[i as usize]
                    = emu.memory[(emu.index_register + i) as usize];
            }
        }
        _ => ()
    };


    emu.program_counter += 2;
}

fn get_binary_coded_decimal(value: u8) -> (u8, u8, u8) {
    let x = value / 100;
    let y = (value - (x * 100)) / 10;
    let z = value - x * 100 - y * 10;

    return (x, y, z);
}

impl Emulator {
    fn set_pixel(&mut self, x: u16, y: u16, pixel: Pixel) -> bool {
        let i = (y * Emulator::SCREEN_WIDTH + x) as usize;

        if i > (Emulator::SCREEN_WIDTH * Emulator::SCREEN_HEIGHT - 1) as usize {
            return false;
        }

        let previous_state: Pixel = self.graphics[i];

        if previous_state != pixel {
            // If the current pixel is different to the new pixel the value
            // will be set to ON
            self.graphics[i] = Pixel::ON;
        } else {
            // Otherwise it will be OFF
            self.graphics[i] = Pixel::OFF;
        }

        // If the pixel changed from ON -> OFF indicate that it was
        return previous_state == Pixel::ON && self.graphics[i] == Pixel::OFF;
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

        emu.emulate_cycle();
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
            emu.emulate_cycle();
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

        // Set register B to 66
        emu.memory[pc + 6] = 0x6B;
        emu.memory[pc + 7] = 0x66;

        for _ in 0..3 {
            emu.emulate_cycle();
        }

        // We have moved 3 instructions + one skipped instruction = 4 * 2 = 8
        assert_eq!(pc + 8, emu.program_counter);

        // Make sure that the last instruction didn't execute and that register A
        // is still at the inital value we set
        assert_eq!(0x33, emu.registers[0xA]);
    }

    /// Test that we can insert a value to a register and then compare against
    /// value to skip the instruction
    #[test]
    fn skip_false() {
        let mut emu = Emulator::new();
        let pc = emu.program_counter;

        // Set register A to A33
        emu.memory[pc] = 0x6A;
        emu.memory[pc + 1] = 0x33;

        // Skip the next instruction if register A is not A34
        emu.memory[pc + 2] = 0x4A;
        emu.memory[pc + 3] = 0x34;

        // Set register A to 66
        emu.memory[pc + 4] = 0x6A;
        emu.memory[pc + 5] = 0x66;

        // Set register B to 66
        emu.memory[pc + 6] = 0x6B;
        emu.memory[pc + 7] = 0x66;

        for _ in 0..3 {
            emu.emulate_cycle();
        }

        // We have moved 3 instructions + one skipped instruction = 4 * 2 = 8
        assert_eq!(pc + 8, emu.program_counter);

        // Make sure that the last instruction didn't execute and that register A
        // is still at the inital value we set
        assert_eq!(0x33, emu.registers[0xA]);
    }

    #[test]
    fn skip_equals() {
        let mut emu = Emulator::new();
        let pc = emu.program_counter;        emu.memory[pc] = 0x6A;
        emu.memory[pc + 1] = 0x33;

        // Skip the next instruction if register A is 33
        emu.memory[pc + 2] = 0x3A;
        emu.memory[pc + 3] = 0x33;

        // Set register A to 66
        emu.memory[pc + 4] = 0x6A;
        emu.memory[pc + 5] = 0x66;

        // Set register B to 66
        emu.memory[pc + 6] = 0x6B;
        emu.memory[pc + 7] = 0x66;

        for _ in 0..3 {
            emu.emulate_cycle();
        }

        // We have moved 3 instructions + one skipped instruction = 4 * 2 = 8
        assert_eq!(pc + 8, emu.program_counter);

        // Make sure that the last instruction didn't execute and that register A
        // is still at the inital value we set
        assert_eq!(0x33, emu.registers[0xA]);
    }

    /// Test that we can insert a value to a register and then compare against
    /// value to skip the instruction
    #[test]
    fn skip_false2() {
        let mut emu = Emulator::new();
        let pc = emu.program_counter;

        // Set register A to A33
        emu.memory[pc] = 0x6A;
        emu.memory[pc + 1] = 0x33;

        // Skip the next instruction if register A is not A34
        emu.memory[pc + 2] = 0x4A;
        emu.memory[pc + 3] = 0x34;


        emu.registers[2] = 5;
        emu.registers[3] = 5;

        // Skip the next instruction if VX equals VY
        emu.memory[pc] = 0x52;
        emu.memory[pc + 1] = 0x30;

        // Process that instruction
        emu.emulate_cycle();

        // Make sure we have skipped ahead two instructions
       assert_eq!(emu.program_counter, pc + 4);
    }

    #[test]
    fn not_equals() {
        let mut emu = Emulator::new();
        let pc = emu.program_counter;
        emu.registers[2] = 5;
        emu.registers[3] = 6;

        // Skip the next instruction if VX doesn't equal VY
        emu.memory[pc] = 0x92;
        emu.memory[pc + 1] = 0x30;

        // Process that instruction
        emu.emulate_cycle();

        // Make sure we have skipped ahead two instructions
        assert_eq!(emu.program_counter, pc + 4);
    }

    #[test]
    fn add_to_register() {
        let mut emu = Emulator::new();
        let pc = emu.program_counter;
        emu.registers[3] = 5;

        // Add 6 to register 3
        emu.memory[pc] = 0x73;
        emu.memory[pc + 1] = 0x06;

        emu.emulate_cycle();

        assert_eq!(11, emu.registers[3]);
    }

    #[test]
    fn test_assign() {
        let mut emu = Emulator::new();
        let pc = emu.program_counter;
        let y: usize = 3;
        let x: usize = 0xA;

        emu.registers[y] = 5;

        // Assign X to Y
        emu.memory[pc] = 0x8A;
        emu.memory[pc + 1] = 0x30;

        // Process that instruction
        emu.emulate_cycle();

        // Make sure we have assigned to the register
        assert_eq!(5, emu.registers[x]);
    }

    #[test]
    fn test_carry() {
        let (x, f) = addition_carry(255, 0);
        assert_eq!(255, x);
        assert_eq!(0, f);

        let (x, f) = addition_carry(255, 2);
        assert_eq!(1, x);
        assert_eq!(1, f);

        let (x, f) = minus_carry(50, 3);
        assert_eq!(47, x);
        assert_eq!(1, f);

        let (x, f) = minus_carry(0, 1);
        assert_eq!(255, x);
        assert_eq!(0, f);
    }

    #[test]
    fn right_bit_shift() {
        let mut emu = Emulator::new();
        let pc = emu.program_counter;
        let x: usize = 0xA;

        emu.registers[x] = 0b00000001;

        // Shift x to the right
        emu.memory[pc] = 0x8A;
        emu.memory[pc + 1] = 0x06;

        // Process that instruction
        emu.emulate_cycle();

        // Make sure we have assigned to the register
        assert_eq!(0, emu.registers[x]);
        assert_eq!(1, emu.registers[0xF]);
    }

    #[test]
    fn left_bit_shift() {
        let mut emu = Emulator::new();
        let pc = emu.program_counter;
        let x: usize = 0xA;

        emu.registers[x] = 0b10000000;

        // Shift x to the left
        emu.memory[pc] = 0x8A;
        emu.memory[pc + 1] = 0x0E;

        // Process that instruction
        emu.emulate_cycle();

        // Make sure we have assigned to the register
        assert_eq!(0, emu.registers[x]);
        assert_eq!(1, emu.registers[0xF]);
    }

    #[test]
    fn jump_to_address() {
        let mut emu = Emulator::new();
        let pc = emu.program_counter;

        emu.registers[0] = 5;

        // Jump to the address ABC
        emu.memory[pc] = 0xBA;
        emu.memory[pc + 1] = 0xBC;

        // Process that instruction
        emu.emulate_cycle();

        // Make sure we have jumped to the address 0xABC + 5
        assert_eq!(0xABC + 5, emu.program_counter);
    }

    #[test]
    fn rand() {
        let mut emu = Emulator::new();
        let pc = emu.program_counter;

        emu.registers[6] = 0x1F;

        // Put a random number into 6
        emu.memory[pc] = 0xC6;
        emu.memory[pc + 1] = 0x0F;

        // Process that instruction
        emu.emulate_cycle();

        // Make sure that register 6 is random number <= 0x0F
        assert!(emu.registers[6] <= 0x0F);
    }

    #[test]
    fn clear_screen() {
        let mut emu = Emulator::new();
        let pc = emu.program_counter;

        // Put some random stuff on the screen
        emu.graphics[14] = Pixel::ON;
        emu.graphics[02] = Pixel::ON;
        emu.graphics[4] = Pixel::ON;

        // Clear the screen
        emu.memory[pc] = 0x00;
        emu.memory[pc + 1] = 0xE0;

        // Process that instruction
        emu.emulate_cycle();

        // Make sure that the screen is blank
        for pixel in emu.graphics.iter() {
            assert_eq!(Pixel::OFF, *pixel);
        }
    }

    #[test]
    fn binary_coded_decimal() {
        let bcd = get_binary_coded_decimal(254);
        assert_eq!(2, bcd.0);
        assert_eq!(5, bcd.1);
        assert_eq!(4, bcd.2);
    }
}
