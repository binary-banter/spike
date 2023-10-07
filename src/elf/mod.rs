mod elf;
mod program;
mod section;

use elf::ElfHeader;
use program::ProgramHeader;
use std::io::Write;
use std::mem::size_of;
use zerocopy::AsBytes;

#[repr(C, packed)]
pub struct ElfFile<'a> {
    header: ElfHeader,
    p_headers: Vec<ProgramHeader>,
    program: &'a [u8],
}

impl<'a> ElfFile<'a> {
    pub fn new(program: &'a [u8]) -> Self {
        let p_headers = vec![ProgramHeader::new(0x1000, program.len() as u64)];

        Self {
            header: ElfHeader::new(0x400000, p_headers.len() as u16),
            p_headers,
            program,
        }
    }

    pub fn emit(self, w: &mut impl Write) {

        w.write_all(self.header.as_bytes()).unwrap();
        for pheader in self.p_headers {
            w.write_all(pheader.as_bytes()).unwrap();
        }

        for _ in 0..0x1000 - size_of::<ElfHeader>() - size_of::<ProgramHeader>() {
            w.write_all(&[0]).unwrap();
        }

        w.write_all(self.program).unwrap();
    }
}
