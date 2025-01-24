use std::{collections::HashMap, fs, time::{Duration, Instant}};
use minifb::{Key, Window, WindowOptions};
use rand::Rng;
use rodio::{source::SineWave, Sink, Source};

const WIDTH: usize = 64;
const HEIGHT: usize = 32;

struct Chip8 {
    mem: [u8; 4096],
    pc: u16,
    v: [u8; 16],
    i: u16,
    stack: [u16; 16],
    sp: u8,
    delay_timer: u8,
    sound_timer: u8,
    display: [[bool; WIDTH]; HEIGHT],
    key_map: HashMap<u8, Key>
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
        self.display.fill([false; WIDTH]);
    }

    fn jsr(&mut self, target: u16) {
        self.stack[self.sp as usize] = self.pc;
        self.sp = (self.sp + 1) % 16;
        self.pc = target;
    }

    fn rts(&mut self) {
        let (result, overflow) = self.sp.overflowing_sub(1);
        
        if overflow {
            self.sp = 15;
        } else {
            self.sp = result;
        }
        
        self.pc = self.stack[self.sp as usize];
    }

    fn skc(&mut self, condition: bool) {
        if condition {
            self.pc += 2;
        }
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
        display: [[false; WIDTH]; HEIGHT],
        key_map: HashMap::from([
            (0x1, Key::Key1),
            (0x2, Key::Key2),
            (0x3, Key::Key3),
            (0xC, Key::Key4),
            (0x4, Key::Q),
            (0x5, Key::W),
            (0x6, Key::E),
            (0xD, Key::R),
            (0x7, Key::A),
            (0x8, Key::S),
            (0x9, Key::D),
            (0xE, Key::F),
            (0xA, Key::Z),
            (0x0, Key::X),
            (0xB, Key::C),
            (0xF, Key::V),
        ])
    };

    let mut rng = rand::thread_rng();

    let mut window = Window::new(
        "Chip8 Emulator",
        WIDTH,
        HEIGHT,
        WindowOptions{
            resize: false,
            scale: minifb::Scale::X16,
            ..WindowOptions::default()
        },
    ).unwrap_or_else(|e| {
        panic!("{}", e);
    });

    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];

    chip8.load_fonts();
    chip8.load_rom("roms/test_opcode.ch8");

    let mut last_timer_update: Instant = Instant::now();

    while window.is_open() && !window.is_key_down(Key::Escape) {
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
                    _ => panic!("Illegal Opcode in ROM")
                }
            },
            0x1 => chip8.jmp(nnn),
            0x2 => chip8.jsr(nnn),
            0x3 => chip8.skc(chip8.v[x as usize] == instr_low),
            0x4 => chip8.skc(chip8.v[x as usize] != instr_low),
            0x5 => chip8.skc(chip8.v[x as usize] == chip8.v[y as usize]),
            0x6 => chip8.v[x as usize] = instr_low,
            0x7 => chip8.v[x as usize] += instr_low,
            0x8 => {
                match z {
                    0x0 => chip8.v[x as usize ] = chip8.v[y as usize],
                    0x1 => chip8.v[x as usize] = chip8.v[x as usize] | chip8.v[y as usize],
                    0x2 => chip8.v[x as usize] = chip8.v[x as usize] & chip8.v[y as usize],
                    0x3 => chip8.v[x as usize] = chip8.v[x as usize] ^ chip8.v[y as usize],
                    0x4 => {
                        let vx = chip8.v[x as usize];
                        let vy = chip8.v[y as usize];

                        let (result, overflow) = vx.overflowing_add(vy);

                        chip8.v[x as usize] = result;
                        chip8.v[0xF] = if overflow {1} else {0};
                    },
                    0x5 => {
                        let vx = chip8.v[x as usize];
                        let vy = chip8.v[y as usize];

                        let (result, overflow) = vx.overflowing_sub(vy);

                        chip8.v[x as usize] = result;
                        chip8.v[0xF] = if overflow {1} else {0};
                    },
                    0x6 => {
                        chip8.v[0xF] = chip8.v[x as usize] & 0b00000001;
                        chip8.v[x as usize] = chip8.v[x as usize] >> 1;
                    },
                    0x7 => {
                        let vx = chip8.v[x as usize];
                        let vy = chip8.v[y as usize];

                        let (result, overflow) = vy.overflowing_sub(vx);

                        chip8.v[x as usize] = result;
                        chip8.v[0xF] = if overflow {1} else {0};
                    },
                    0xE => {
                        chip8.v[0xF] = (chip8.v[x as usize] & 0b10000000) >> 7;
                        chip8.v[x as usize] = chip8.v[x as usize] << 1;
                    },
                    _ => panic!("Illegal Opcode in ROM")
                }
            },
            0x9 => chip8.skc(chip8.v[x as usize] != chip8.v[y as usize]),
            0xA => chip8.i = nnn,
            0xB => chip8.jmp((chip8.v[0] as u16) + nnn),
            0xC => chip8.v[x as usize] = rng.gen::<u8>() & instr_low,
            0xD => {
                let mut y_disp: u8 = chip8.v[y as usize] % 32;

                chip8.v[0xF] = 0;

                for row in 0..z {
                    let sprite_byte: u8 = chip8.mem[(chip8.i as usize) + (row as usize)];
                    let mut bit: u8 = 0b10000000;
                    let mut x_disp: u8 = chip8.v[x as usize] % 64;

                    while bit > 0 {
                        if sprite_byte & bit != 0 {
                            if chip8.display[y_disp as usize][x_disp as usize] {
                                chip8.display[y_disp as usize][x_disp as usize] = false;
                                chip8.v[0xF] = 1;
                            } else {
                                chip8.display[y_disp as usize][x_disp as usize] = true;
                            }
                        }
                        bit = bit >> 1;
                        x_disp += 1;
                        if x_disp == 64 {
                            break;
                        }
                    }
                    y_disp += 1;
                    if y_disp == 32 {
                        break;
                    }
                }
            },
            0xE => {
                match instr_low {
                    0x9E => chip8.skc(window.is_key_down(chip8.key_map[&chip8.v[x as usize]])),
                    0xA1 => chip8.skc(window.is_key_released(chip8.key_map[&chip8.v[x as usize]])),
                    _ => panic!("Illegal Opcode in ROM")
                }
            },
            0xF => {
                match instr_low {
                    0x07 => chip8.v[x as usize] = chip8.delay_timer,
                    0x0A => {
                        let mut keycode_pressed: u8 = 0xFF;
                        let mut is_key_pressed: bool = false;

                        for (keycode, key) in chip8.key_map.iter() {
                            if window.is_key_down(*key) {
                                keycode_pressed = *keycode;
                                is_key_pressed = true;
                                break;
                            }
                        }

                        if is_key_pressed {
                            chip8.v[x as usize] = keycode_pressed;
                        } else {
                            chip8.pc -= 2;
                        }
                    },
                    0x15 => chip8.delay_timer = chip8.v[x as usize],
                    0x18 => chip8.sound_timer = chip8.v[x as usize],
                    0x1E => chip8.i += chip8.v[x as usize] as u16,
                    0x29 => chip8.i = 0x50 + (5 * chip8.v[x as usize] as u16),
                    0x33 => {
                        let mut hex_num: u8 = chip8.v[x as usize];
                        chip8.mem[(chip8.i + 2) as usize] = hex_num % 10;
                        hex_num /= 10;
                        chip8.mem[(chip8.i + 1) as usize] = hex_num % 10;
                        hex_num /= 10;
                        chip8.mem[(chip8.i) as usize] = hex_num % 10;
                    },
                    0x55 => {
                        for i in 0..x {
                            chip8.mem[(chip8.i as usize) + (i as usize)] = chip8.v[i as usize];
                        }
                    },
                    0x65 => {
                        for i in 0..x {
                            chip8.v[i as usize] = chip8.mem[(chip8.i as usize) + (i as usize)];
                        }
                    },
                    _ => panic!("Illegal Opcode in ROM")
                }
            }
            _ => panic!("Illegal Opcode in ROM: {:#01x}", (instr_high & 0xF0) >> 4)
        }

        let now = Instant::now();
        if now.duration_since(last_timer_update) >= Duration::from_secs_f64(1./64.) {

            if chip8.delay_timer > 0 {
                chip8.delay_timer -= 1;
            }

            if chip8.sound_timer > 0 {
                let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
                let sink = Sink::try_new(&stream_handle).unwrap();
                let beep = SineWave::new(440.0);
                sink.append(beep.take_duration(std::time::Duration::from_millis(100)));
                sink.detach();
                chip8.sound_timer -= 1;
            }

            last_timer_update = now;
        }

        for (i, &pixel) in chip8.display.as_flattened().iter().enumerate() {
            buffer[i] = if pixel {0x00FFFFFF} else {0x00000000};
            println!("{}", buffer[i]);
        }

        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }

}

