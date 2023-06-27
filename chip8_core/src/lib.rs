use rand::random;

pub const SCREEN_WIDTH: usize = 64; // Public because the front-end will need to read it
pub const SCREEN_HEIGHT: usize = 32;

const RAM_SIZE: usize = 4096; // kb
const NUM_REGS: usize = 16;
const STACK_SIZE: usize = 16;
const NUM_KEYS: usize = 16;
const START_ADDR: u16 = 0x200; // d512 Where the program will be loaded in RAM from

const FONTSET_SIZE: usize = 80;

const FONTSET: [u8; FONTSET_SIZE] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

pub struct Emu {
    pc: u16,             // Program Counter register (PC)
    ram: [u8; RAM_SIZE], // RAM: array of 1byte integers * RAM_SIZE
    screen: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
    v_regs: [u8; NUM_REGS],
    i_reg: u16,
    sp: u16,                  // Stack Pointer (SP)
    stack: [u16; STACK_SIZE], // LIFO, used when entering / exiting a subroutine
    keys: [bool; NUM_KEYS],
    dt: u8,              // Delay Timer (DT)
    st: u8,              // Sound Timer (ST)
    pub play_beep: bool, // Play beep sound
}

impl Emu {
    pub fn new() -> Self {
        let mut new_emu = Self {
            pc: START_ADDR,
            ram: [0; RAM_SIZE],
            screen: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            v_regs: [0; NUM_REGS],
            i_reg: 0,
            sp: 0,
            stack: [0; STACK_SIZE],
            keys: [false; NUM_KEYS],
            dt: 0,
            st: 0,
            play_beep: false,
        };

        new_emu.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
        new_emu
    }

    pub fn reset(&mut self) {
        self.pc = START_ADDR;
        self.ram = [0; RAM_SIZE];
        self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
        self.v_regs = [0; NUM_REGS];
        self.i_reg = 0;
        self.sp = 0;
        self.stack = [0; STACK_SIZE];
        self.keys = [false; NUM_KEYS];
        self.dt = 0;
        self.st = 0;
        self.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
        self.play_beep = false;
    }

    fn push(&mut self, val: u16) {
        self.stack[self.sp as usize] = val;
        self.sp += 1;
    }

    fn pop(&mut self) -> u16 {
        self.sp -= 1;
        self.stack[self.sp as usize]
    }

    pub fn tick(&mut self) {
        // Fetch
        let op = self.fetch();
        // Decode & Execute
        self.execute(op);
        self.tick_timers();
    }

    fn execute(&mut self, op: u16) {
        let digit1 = (op & 0xF000) >> 12;
        let digit2 = (op & 0x0F00) >> 8;
        let digit3 = (op & 0x00F0) >> 4;
        let digit4 = op & 0x000F;

        match (digit1, digit2, digit3, digit4) {
            (0, 0, 0, 0) => return, // NOP - No operation
            (0, 0, 0xE, 0) => {
                // CLS - Clear screen
                self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
            }
            (0, 0, 0xE, 0xE) => {
                // RET - Return from subroutine
                self.pc = self.pop();
            }
            (1, _, _, _) => {
                // JMP NNN - Jump to 0xNNN
                self.pc = op & 0xFFF; // 0x1NNN & 0xFFF = 0xNNN
            }
            (2, _, _, _) => {
                // CALL NNN - Enter subroutine at 0xNNN
                self.push(self.pc);
                dbg!(self.pc);
                self.pc = op & 0xFFF;
                dbg!(self.pc);
            }
            (3, _, _, _) => {
                // SKP VX == 0xNN
                let vx_addr = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                if self.v_regs[vx_addr] == nn {
                    self.pc += 2;
                }
            }
            (4, _, _, _) => {
                // SKP VX != 0xNN
                let vx_addr = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                if self.v_regs[vx_addr] != nn {
                    self.pc += 2;
                }
            }
            (5, _, _, _) => {
                // SKP VX == VY
                let vx_addr = digit2 as usize;
                let vy_addr = digit3 as usize;
                if self.v_regs[vx_addr] == self.v_regs[vy_addr] {
                    self.pc += 2;
                }
            }
            (6, _, _, _) => {
                // SET VX = NN
                let nn = (op & 0xFF) as u8;
                let vx_addr = digit2 as usize;
                self.v_regs[vx_addr] = nn;
            }
            (7, _, _, _) => {
                // SET VX += NN - Doesn't affect carry flag
                let nn = (op & 0xFF) as u8;
                let vx_addr = digit2 as usize;
                self.v_regs[vx_addr] = self.v_regs[vx_addr].wrapping_add(nn);
            }
            (8, _, _, 0) => {
                // SET VX = VY
                let vx_addr = digit2 as usize;
                let vy_addr = digit3 as usize;
                self.v_regs[vx_addr] = self.v_regs[vy_addr];
            }
            (8, _, _, 1) => {
                // SET VX |= VY
                let vx_addr = digit2 as usize;
                let vy_addr = digit3 as usize;
                self.v_regs[vx_addr] |= self.v_regs[vy_addr];
            }
            (8, _, _, 2) => {
                // SET VX &= VY
                let vx_addr = digit2 as usize;
                let vy_addr = digit3 as usize;
                self.v_regs[vx_addr] &= self.v_regs[vy_addr];
            }
            (8, _, _, 3) => {
                // SET VX ^= VY
                let vx_addr = digit2 as usize;
                let vy_addr = digit3 as usize;
                self.v_regs[vx_addr] ^= self.v_regs[vy_addr];
            }
            (8, _, _, 4) => {
                // VX += VY
                let vx_addr = digit2 as usize;
                let vy_addr = digit3 as usize;
                let (new_vx, carry) = self.v_regs[vx_addr].overflowing_add(self.v_regs[vy_addr]);

                let new_vf = if carry { 1 } else { 0 };
                self.v_regs[vx_addr] = new_vx;
                self.v_regs[0xF] = new_vf; // Setting VF
            }
            (8, _, _, 5) => {
                // VX -= VY
                let vx_addr = digit2 as usize;
                let vy_addr = digit3 as usize;

                let (new_vx, borrow) = self.v_regs[vx_addr].overflowing_sub(self.v_regs[vy_addr]);
                let new_vf = if borrow { 0 } else { 1 };

                self.v_regs[vx_addr] = new_vx;
                self.v_regs[0xF] = new_vf; // Setting VF
            }
            (8, _, _, 6) => {
                // VX >>= 1
                let vx_addr = digit2 as usize;
                let lsb = self.v_regs[vx_addr] & 1;
                self.v_regs[vx_addr] >>= 1;
                self.v_regs[0xF] = lsb;
            }
            (8, _, _, 7) => {
                // VX = VY - VX
                let vx_addr = digit2 as usize;
                let vy_addr = digit3 as usize;

                let (new_vx, borrow) = self.v_regs[vy_addr].overflowing_sub(self.v_regs[vx_addr]);
                let new_vf = if borrow { 0 } else { 1 };

                self.v_regs[vx_addr] = new_vx;
                self.v_regs[0xF] = new_vf;
            }
            (8, _, _, 0xE) => {
                // VX <<= 1
                let vx_addr = digit2 as usize;
                let msb = (self.v_regs[vx_addr] >> 7) & 1;
                self.v_regs[vx_addr] <<= 1;
                self.v_regs[0xF] = msb;
            }
            (9, _, _, 0) => {
                // SKP VX != VY
                let vx_addr = digit2 as usize;
                let vy_addr = digit3 as usize;
                if self.v_regs[vx_addr] != self.v_regs[vy_addr] {
                    self.pc += 2;
                }
            }
            (0xA, _, _, _) => {
                // I = 0xNNN
                self.i_reg = op & 0xFFF;
            }
            (0xB, _, _, _) => {
                // JMP V0 + 0xNNN
                let nnn = op & 0xFFF;
                self.pc = (self.v_regs[0] as u16) + nnn;
            }
            (0xC, _, _, _) => {
                // VX = RND & 0xNN
                let vx_addr = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                let rng: u8 = random();

                self.v_regs[vx_addr] = rng & nn;
            }
            (0xD, _, _, _) => {
                // DRAW
                // Get the coordinates
                let x_cord = self.v_regs[digit2 as usize] as u16;
                let y_cord = self.v_regs[digit3 as usize] as u16;
                let num_rows = digit4; // How many rows high our sprite is

                let mut flipped = false;
                for y_line in 0..num_rows {
                    // Iterate each line of the sprite
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
            }
            (0xE, _, 9, 0xE) => {
                // SKP KEY VX PRESSED
                let vx = self.v_regs[digit2 as usize];
                let key = self.keys[vx as usize];
                if key {
                    self.pc += 2;
                }
            }
            (0xE, _, 0xA, 1) => {
                // SKP KEY VX NOT PRESSED
                let vx = self.v_regs[digit2 as usize];
                let key = self.keys[vx as usize];
                if !key {
                    self.pc += 2;
                }
            }
            (0xF, _, 0, 7) => {
                // VX = DT
                self.v_regs[digit2 as usize] = self.dt;
            }
            (0xF, _, 0, 0xA) => {
                // WAIT FOR KEY - Blocking operation
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
            }
            (0xF, _, 1, 5) => {
                // DT = VX
                self.dt = self.v_regs[digit2 as usize];
            }
            (0xF, _, 1, 8) => {
                // ST = VX
                self.st = self.v_regs[digit2 as usize];
            }
            (0xF, _, 1, 0xE) => {
                // I += VX
                let vx = self.v_regs[digit2 as usize] as u16;
                self.i_reg = self.i_reg.wrapping_add(vx);
            }
            (0xF, _, 2, 9) => {
                // SET I TO FONT CHAR IN VX
                let vx_addr = digit2 as usize;
                let c = self.v_regs[vx_addr] as u16;

                self.i_reg = c * 5; // Each character is 5 bytes
            }
            (0xF, _, 3, 3) => {
                // I = ADDR(BCD OF VX) - Binary-Coded Decimal
                // TODO: implement a more efficent BCD algorithm
                let vx_addr = digit2 as usize;
                let vx = self.v_regs[vx_addr] as f32;

                let hundreds = (vx / 100.0).floor() as u8;
                let tens = ((vx / 10.0) % 10.0).floor() as u8;
                let ones = (vx % 10.0) as u8;

                self.ram[self.i_reg as usize] = hundreds;
                self.ram[(self.i_reg + 1) as usize] = tens;
                self.ram[(self.i_reg + 2) as usize] = ones;
            }
            (0xF, _, 5, 5) => {
                // CP V0-VX TO RAM FROM I
                let vx_addr = digit2 as usize;
                let i = self.i_reg as usize;

                for idx in 0..=vx_addr {
                    self.ram[i + idx] = self.v_regs[idx];
                }
            }
            (0xF, _, 6, 5) => {
                // CP RAM TO V0-VX FROM I
                let vx_addr = digit2 as usize;
                let i = self.i_reg as usize;

                for idx in 0..=vx_addr {
                    self.v_regs[idx] = self.ram[i + idx];
                }
            }
            (_, _, _, _) => unimplemented!("Unimplemented opcode: {}", op),
        }
    }

    fn fetch(&mut self) -> u16 {
        let higher_byte = self.ram[self.pc as usize] as u16;
        let lower_byte = self.ram[(self.pc + 1) as usize] as u16;
        let op = (higher_byte << 8) | lower_byte; // Big Endian decode
        self.pc += 2;
        op
    }

    pub fn tick_timers(&mut self) {
        if self.dt > 0 {
            self.dt -= 1;
        }

        if self.st > 0 {
            if self.st == 1 {
                self.play_beep = true;
            }
            self.st -= 1;
        }
    }

    pub fn get_display(&self) -> &[bool] {
        &self.screen
    }

    pub fn keypress(&mut self, idx: usize, pressed: bool) {
        self.keys[idx] = pressed;
    }

    pub fn load(&mut self, program: &[u8]) {
        let start = START_ADDR as usize;
        let end = (START_ADDR as usize) + program.len();

        self.ram[start..end].copy_from_slice(program);
    }
}
