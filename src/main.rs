use std::fs;

struct Chip8 {
    mem: [u8; 4096],
    pc: u16,
    v: [i8; 16],
    i: i16,
    stack: [i16; 16],
    sp: i8,
    delay_timer: i8,
    sound_timer: i8,
    display: [bool; 64 * 32],
    keypad: [bool; 16],
}

const CHIP8_FONTSET: [u8; 80] = [
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
    0xF0, 0x80, 0xF0, 0x80, 0x80  // F
];

impl Chip8 {
    fn load_fonts(&mut self) {
        let font_start: usize = 0x50;
        self.mem[font_start..font_start + CHIP8_FONTSET.len()].clone_from_slice(&CHIP8_FONTSET);
    }  
    
    fn load_rom(&mut self, filename: &str) {
        let rom_start: usize = 0x200;
        let rom: Vec<u8> = fs::read(filename).expect("Unable to Read ROM");
        self.mem[rom_start..rom_start + rom.len()].clone_from_slice(&rom);
    }

    fn jmp(&mut self, target: u16) {
        self.pc = target;
    }

    fn cls(&mut self) {
        self.display.fill(false);
    }

    fn rts(&mut self) {

    }
}


fn main() {
    let mut chip8 = Chip8 {
        mem: [0; 4096],
        pc: 0x200,
        v: [0; 16],
        i: 0,
        stack: [0; 16],
        sp: 0,
        delay_timer: 0,
        sound_timer: 0,
        display: [false; 64 * 32],
        keypad: [false; 16]
    };

    chip8.load_fonts();
    chip8.load_rom("roms/SCTEST");

    while true {
        let instr_high: u8 = chip8.mem[chip8.pc as usize];
        let instr_low: u8 = chip8.mem[(chip8.pc + 1) as usize];
        chip8.pc += 2;
        
        println!("{:#04x}", ((instr_high as u16) << 8 | (instr_low as u16)));

        let x: u8 = instr_high & 0x0F;
        let y: u8 = instr_low & 0xF0 >> 4;
        let z: u8 = instr_low & 0x0F;
        let nnn: u16 = (x as u16) << 8 | (instr_low as u16);

        match (instr_high & 0xF0) >> 4 {
            0x0 => {
                match instr_low {
                    0xE0 => chip8.cls(),
                    0xEE => chip8.rts(),
                    _ => println!("Illegal Opcode in ROM")
                }
            },
            0x1 => {
                chip8.jmp(nnn);
            },
            _ => {
                println!("Illegal Opcode in ROM: {:#01x}", (instr_high & 0xF0) >> 4);
                break;
            }
        }
    }

}

