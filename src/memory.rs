pub struct ROM32K {
    pub raw_program: Vec<u8>,
}

pub struct RAM16K {
    pub mem_32: Vec<u16>,
}

impl RAM16K {
    pub fn new() -> Self {
        Self {
            mem_32: vec![0; 0xFFFF],
        }
    }

    pub fn read_u16(&self, addr: u16) -> &u16 {
        return &self.mem_32[addr as usize];
    }

    pub fn write_u16(&mut self, addr: u16, data: u16) {
        if let Some(elem) = self.mem_32.get_mut(addr as usize) {
            *elem = data;
        }
    }
}

impl ROM32K {
    pub fn new() -> Self {
        Self {
            raw_program: vec![0; 0xFFFFF],
        }
    }

    pub fn load_program(&mut self, program: Vec<u8>) {
        self.raw_program = program;
    }

    pub fn read_byte(&self, addr: u16) -> &u8 {
        return &self.raw_program[addr as usize];
    }

    pub fn dump(&self) {
        for (i, byte) in self.raw_program.iter().enumerate().take(4*10) {
            if i % 4 == 0 && i != 0 {
                println!("|");
            }
            print!("{:0>8b} ", *byte);
        }
    }
}