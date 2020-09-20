use std::{fs::File, io::Read};
use nfd::Response;
use std::convert::TryInto;
use olc_pixel_game_engine as olc;

pub mod cpu;
pub mod memory;
pub mod consts;

struct Hackcat {
    pub cpu: cpu::CPU,
    pub screen: olc::Sprite,
    pub disassemble: Option<Vec<String>>
}

pub const SCREEN_ADDR : i32 = 16384;
pub const SCREEN_W : i32 = 512;
pub const SCREEN_H : i32 = 256;
pub const KBR_ADDR : i32 = 24576;

impl olc::Application for Hackcat {
  fn on_user_create(&mut self) -> Result<(), olc::Error> {
    olc::clear(olc::WHITE);
    Ok(())
  }

  fn on_user_update(&mut self, _elapsed_time: f32) -> Result<(), olc::Error> {
    for _ in 0..1000000 {
        self.cpu.execute();
    }

    if self.cpu.cc % 400000 == 0 {
        self.cpu.ram.write_u16(24576_u16, handle_input());
        let vram = &self.cpu.ram.mem_32[(16384_u16 as usize) .. (24576_u16 as usize)];
        for (idx, pix) in vram.iter().enumerate() {
            let mut sect = *pix;
            for i in 0..16 {
                self.screen.set_pixel(
                    ((idx as i32 % (SCREEN_W/16)) * 16) + i,
                    idx as i32 / (SCREEN_W/16) ,
                    if sect & 0b1 == 0b1 {olc::BLACK} else {olc::WHITE}
                );
                sect = sect >> 1;
            }
        }
        olc::clear(olc::WHITE);
        olc::draw_sprite(0, 0, &self.screen);
    
        //dbg
        let _ = olc::draw_string(SCREEN_W + 10, 3, (format!("CC: {} PC: {}", self.cpu.cc/100000, self.cpu.pc/2)).as_str(), olc::BLACK);
        let _ = olc::draw_string(SCREEN_W + 10, 13, (format!("AR: {}", self.cpu.register_a)).as_str(), olc::BLACK);
        let _ = olc::draw_string(SCREEN_W + 10, 23, (format!("DR: {}", self.cpu.register_d)).as_str(), olc::BLACK);
        let _ = olc::draw_string(SCREEN_W + 10, 33, (format!("{}", self.cpu.ram.read_u16(24576_u16))).as_str(), olc::BLACK);
    
        self.disassemble.as_mut().unwrap().iter().skip((self.cpu.pc/2) as usize).enumerate().take(6).for_each(|(i, opc)| {
            if i == 0 {
                let _ = olc::draw_string(SCREEN_W + 10, 43 + 10*i as i32, ("> ".to_string() + opc).as_str(), olc::DARK_RED);
            } else {
                let _ = olc::draw_string(SCREEN_W + 10, 43 + 10*i as i32, opc.as_str(), olc::BLACK);
            }
        });
    }
    Ok(())
  }

  fn on_user_destroy(&mut self) -> Result<(), olc::Error> {
    Ok(())
  }
}

fn main() {
    let mut cpu = cpu::CPU::new();
    let program_path = load_file();
    let mut program_file = File::open(program_path).expect("No File");
    let mut program = vec![0; 0xFFFF];
    program_file.read(&mut program).expect("Buffer Overflow");
    cpu.rom.load_program(treat_file(program));
    cpu.ram.write_u16(0, 10);
    cpu.ram.write_u16(1, 2);
    let disassemble = (&cpu).disassemble_loaded_rom();
    cpu.rom.dump();
    let mut example = Hackcat { cpu, screen: olc::Sprite::with_dims(SCREEN_W, SCREEN_H), disassemble: Some(disassemble) };
    olc::start("Hackcat~", &mut example, SCREEN_W + 200, SCREEN_H, 1, 1).unwrap();
}

fn load_file() -> String {
    let result = nfd::open_file_dialog(None, None).unwrap_or_else(|e| {
        panic!(e);
    });
  
    match result {
        Response::Okay(file_path) => return file_path,
        Response::OkayMultiple(_) => panic!("nahh"),
        Response::Cancel => panic!("gamer"),
    }
}

pub fn treat_file(file: Vec<u8>) -> Vec<u8> {
    let mut n_buffer = vec![0; 0xFFFF];
    let mut i_f = 0;
    for (i, e) in file.iter().enumerate().step_by(17) {
        if *e == 0 {break;}
        let slice1 = &file[i..i+8];
        let slice2 = &file[i+8..i+16];
        let string = String::from_utf8(slice1.try_into().expect("bruh")).unwrap();
        let intval = isize::from_str_radix(string.as_str(), 2).unwrap();
        n_buffer[i_f] = intval as u8;
        let string = String::from_utf8(slice2.try_into().expect("bruh")).unwrap();
        let intval = isize::from_str_radix(string.as_str(), 2).unwrap();
        n_buffer[i_f+1] = intval as u8;
        i_f += 2;
    }
    println!("PSIZE: {}", i_f/2);
    return n_buffer;
}

fn handle_input() -> u16 {
    for key in consts::INPUTS.iter() {
        if olc::get_key(key.0).held {
            return key.2 as u16;
        }
    }

    return 0;
}