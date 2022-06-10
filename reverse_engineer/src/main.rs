use std::env;
use std::fs::File;
use std::io::Read;

fn main() {
    println!("Hello, world!");

    let args: Vec<_> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: cargo run path/to/the/game");
        return;
    }

    // Read ROM

    println!("Requested game path: {}", &args[1]);
    let mut rom = File::open(&args[1]).expect("Unable to open file");
    let mut buffer = Vec::new();
    rom.read_to_end(&mut buffer).unwrap();

    for i in  (0..buffer.len()).step_by(2) {
        let opcode: u16 = buffer[i + 1] as u16 >> 8 | buffer[i] as u16;
        println!("0x{:04X}", opcode);
        println!("0x{:02X}{:02X}", buffer[i], buffer[i+1]);
    }
}

fn reverse_opcode(opcode: u8) {
    let digit1 = (opcode & 0xF000) >> 12;
    let digit2 = (opcode & 0x0F00) >> 8;
    let digit3 = (opcode & 0x00F0) >> 4;
    let digit4 = opcode & 0x000F;
    
    println!("0x{:04X}", &opcode);

    match (digit1, digit2, digit3, digit4) {
        (0, 0, 0, 0) => return, // NOP - No operation
        (0, 0, 0xE, 0) => {     // CLS - Clear screen
            
        },
        (0, 0, 0xE, 0xE) => {   // RET - Return from subroutine
            
        },
        (1, _, _, _) => {       // JMP NNN - Jump to 0xNNN
            
        },
        (2, _, _, _) => {       // CALL NNN - Enter subroutine at 0xNNN
            self.push(self.pc); // Push into the stack
            println!("CALL NNN: 0x{:03X}", opcode & 0xFFF);
            dbg!(self.pc);
            self.pc = op & 0xFFF;
            dbg!(self.pc);
        },
        (3, _, _, _)  => {      // SKP VX == 0xNN
            let vx_addr = digit2 as usize;
            let nn = (op & 0xFF) as u8;
            if self.v_regs[vx_addr] == nn {
                self.pc += 2;
            }
        },
        (4, _, _, _) => {       // SKP VX != 0xNN
            let vx_addr = digit2 as usize;
            let nn = (op & 0xFF) as u8;
            if self.v_regs[vx_addr] != nn {
                self.pc += 2;
            }
        },
        (5, _, _, _) => {       // SKP VX == VY
            let vx_addr = digit2 as usize;
            let vy_addr = digit3 as usize;
            if self.v_regs[vx_addr] == self.v_regs[vy_addr] {
                self.pc += 2;
            }
        },
        (6, _, _, _) => {       // SET VX = NN
            let nn = (op & 0xFF) as u8;
            let vx_addr = digit2 as usize;
            self.v_regs[vx_addr] = nn;
        },
        (7, _, _, _) => {       // SET VX += NN - Doesn't affect carry flag
            let nn = (op & 0xFF) as u8;
            let vx_addr = digit2 as usize;
            self.v_regs[vx_addr] = self.v_regs[vx_addr].wrapping_add(nn);
        },
        (8, _, _, 0) => {       // SET VX = VY
            let vx_addr = digit2 as usize;
            let vy_addr = digit3 as usize;
            self.v_regs[vx_addr] = self.v_regs[vy_addr];
        },
        (8, _, _, 1) => {       // SET VX |= VY
            let vx_addr = digit2 as usize;
            let vy_addr = digit3 as usize;
            self.v_regs[vx_addr] |= self.v_regs[vy_addr];
        },
        (8, _, _, 2) => {       // SET VX &= VY
            let vx_addr = digit2 as usize;
            let vy_addr = digit3 as usize;
            self.v_regs[vx_addr] &= self.v_regs[vy_addr];
        },
        (8, _, _, 3) => {       // SET VX ^= VY
            let vx_addr = digit2 as usize;
            let vy_addr = digit3 as usize;
            self.v_regs[vx_addr] ^= self.v_regs[vy_addr];
        },
        (8, _, _, 4) => {       // VX += VY
            let vx_addr = digit2 as usize;
            let vy_addr = digit3 as usize;
            let (new_vx, carry) = self.v_regs[vx_addr]
                .overflowing_add(self.v_regs[vy_addr]);
            
            let new_vf = if carry { 1 } else { 0 };
            self.v_regs[vx_addr] = new_vx;
            self.v_regs[0xF] = new_vf; // Setting VF
        },
        (8, _, _, 5) => {       // VX -= VY
            let vx_addr = digit2 as usize;
            let vy_addr = digit3 as usize;

            let (new_vx, borrow) = self.v_regs[vx_addr]
                .overflowing_sub(self.v_regs[vy_addr]);
            let new_vf = if borrow { 0 } else { 1 };
            
            self.v_regs[vx_addr] = new_vx;
            self.v_regs[0xF] = new_vf; // Setting VF
        },
        (8, _, _, 6) => {       // VX >>= 1
            let vx_addr = digit2 as usize;
            let lsb = self.v_regs[vx_addr] & 1;
            self.v_regs[vx_addr] >>= 1;
            self.v_regs[0xF] = lsb;
        },
        (8, _, _, 7) => {       // VX = VY - VX
            let vx_addr = digit2 as usize;
            let vy_addr = digit3 as usize;

            let (new_vx, borrow) = self.v_regs[vy_addr]
                .overflowing_sub(self.v_regs[vx_addr]);
            let new_vf = if borrow { 0 } else { 1 };
            
            self.v_regs[vx_addr] = new_vx;
            self.v_regs[0xF] = new_vf;
        },
        (8, _, _, 0xE) => {     // VX <<= 1
            let vx_addr = digit2 as usize;
            let msb = (self.v_regs[vx_addr] >> 7) & 1;
            self.v_regs[vx_addr] <<= 1;
            self.v_regs[0xF] = msb;
        },
        (9, _, _, 0) => {       // SKP VX != VY
            let vx_addr = digit2 as usize;
            let vy_addr = digit3 as usize;
            if self.v_regs[vx_addr] != self.v_regs[vy_addr] {
                self.pc += 2;
            }
        },
        (0xA, _, _, _) => {     // I = 0xNNN
            self.i_reg = op & 0xFFF;
        },
        (0xB, _, _, _) => {     // JMP V0 + 0xNNN
            let nnn = op & 0xFFF;
            self.pc = (self.v_regs[0] as u16) + nnn;
        },
        (0xC, _, _, _) => {     // VX = RND & 0xNN
            let vx_addr = digit2 as usize;
            let nn = (op & 0xFF) as u8;
            let rng: u8 = random();

            self.v_regs[vx_addr] = rng & nn;
        },
        (0xD, _, _, _) => {     // DRAW
            // Get the coordinates
            let x_cord = self.v_regs[digit2 as usize] as u16;
            let y_cord = self.v_regs[digit3 as usize] as u16;
            let num_rows = digit4; // How many rows high our sprite is
            
            let mut flipped = false;
            for y_line in 0..num_rows { // Iterate each line of the sprite
                let addr = (self.i_reg + y_line) as u16;
                let pixels = self.ram[addr as usize];

                for x_line in 0..8 {
                    if (pixels & (0b10000000 >> x_line)) != 0 {
                        // Sprites should wrap around screen, so apply module
                        let x = (x_cord + x_line) as usize % SCREEN_WIDTH;
                        let y = (y_cord + y_line) as usize % SCREEN_HEIGHT;

                        // Get our pixel's index for our 1D screen array
                        let idx = x + SCREEN_WIDTH * y;

                        // Check if we're about to flip the pixel and set
                        flipped |= self.screen[idx];
                        self.screen[idx] ^= true;
                    }
                }
                // Set VF
                // self.v_regs[0xF] = flipped as u8;
                if flipped {
                    self.v_regs[0xF] = 1;
                } else {
                    self.v_regs[0xF] = 0;
                }
            }
        },
        (0xE, _, 9, 0xE) => {       // SKP KEY VX PRESSED
            let vx = self.v_regs[digit2 as usize];
            let key = self.keys[vx as usize];
            if key {
                self.pc += 2;
            }
        },
        (0xE, _, 0xA, 1) => {       // SKP KEY VX NOT PRESSED
            let vx = self.v_regs[digit2 as usize];
            let key = self.keys[vx as usize];
            if !key {
                self.pc += 2;
            }
        },
        (0xF, _, 0, 7) => {         // VX = DT
            self.v_regs[digit2 as usize] = self.dt;
        },
        (0xF, _, 0, 0xA) => {       // WAIT FOR KEY - Blocking operation
            let vx_addr = digit2 as usize;
            let mut pressed = false;

            for i in 0..self.keys.len() {
                if self.keys[i] {
                    self.v_regs[vx_addr] = i as u8;
                    pressed = true;
                    break;
                }
            }
            
            // Redo opcode
            if !pressed {
                self.pc -= 2;
            }
        },
        (0xF, _, 1, 5) => {         // DT = VX
            self.dt = self.v_regs[digit2 as usize];
        },
        (0xF, _, 1, 8) => {         // ST = VX
            self.st = self.v_regs[digit2 as usize];
        },
        (0xF, _, 1, 0xE) => {       // I += VX
            let vx = self.v_regs[digit2 as usize] as u16;
            self.i_reg = self.i_reg.wrapping_add(vx);
        },
        (0xF, _, 2, 9) => {         // SET I TO FONT CHAR IN VX
            let vx_addr = digit2 as usize;
            let c = self.v_regs[vx_addr] as u16;

            self.i_reg = c * 5; // Each character is 5 bytes
        },
        (0xF, _, 3, 3) => {         // I = ADDR(BCD OF VX) - Binary-Coded Decimal
            // TODO: implement a more efficent BCD algorithm
            let vx_addr = digit2 as usize;
            let vx = self.v_regs[vx_addr] as f32;

            let hundreds = (vx / 100.0).floor() as u8;
            let tens = ((vx / 10.0) % 10.0).floor() as u8;
            let ones = (vx % 10.0) as u8;

            self.ram[self.i_reg as usize] = hundreds;
            self.ram[(self.i_reg + 1) as usize] = tens;
            self.ram[(self.i_reg + 2) as usize] = ones;

        },
        (0xF, _, 5, 5) => {         // CP V0-VX TO RAM FROM I
            let vx_addr = digit2 as usize;
            let i = self.i_reg as usize;

            for idx in 0..=vx_addr {
                self.ram[i + idx] = self.v_regs[idx];
            }
        },
        (0xF, _, 6, 5) => {         // CP RAM TO V0-VX FROM I
            let vx_addr = digit2 as usize;
            let i = self.i_reg as usize;

            for idx in 0..=vx_addr {
                self.v_regs[idx] = self.ram[i + idx];
            }
        }
        (_, _, _, _) => unimplemented!("Unimplemented opcode: {}", op),
    }
}