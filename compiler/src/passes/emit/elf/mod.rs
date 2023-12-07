#![allow(non_camel_case_types)]
#![allow(clippy::upper_case_acronyms)]

mod header;
mod program;
mod section;

use crate::passes::emit::elf::header::ElfHeader;
use crate::passes::emit::elf::program::ProgramHeader;
use crate::time;
use std::io::Write;
use std::mem::size_of;
use zerocopy::AsBytes;

pub const PRG_OFFSET: usize = 0x0040_0000;

pub struct ElfFile {
    header: ElfHeader,
    p_headers: Vec<ProgramHeader>,
    program: Vec<u8>,
}

impl ElfFile {
    pub fn new(entry: usize, program: Vec<u8>) -> Self {
        let p_headers = vec![ProgramHeader::new(0x1000, program.len() as u64)];

        Self {
            header: ElfHeader::new((PRG_OFFSET + entry) as u64, p_headers.len() as u16),
            p_headers,
            program,
        }
    }

    pub fn write(self, w: &mut impl Write) {
        w.write_all(self.header.as_bytes()).unwrap();
        for pheader in self.p_headers {
            w.write_all(pheader.as_bytes()).unwrap();
        }

        for _ in 0..0x1000 - size_of::<ElfHeader>() - size_of::<ProgramHeader>() {
            w.write_all(&[0]).unwrap();
        }

        w.write_all(&self.program).unwrap();

        time!("write");
    }
}
