mod elf;
mod program;
mod section;

use std::hash::Hasher;
use elf::ElfHeader;
use program::ProgramHeader;
use std::io::Write;
use std::mem::size_of;
use zerocopy::AsBytes;

#[repr(C, packed)]
pub struct ElfFile {
    header: ElfHeader,
    p_headers: Vec<ProgramHeader>,
    // s_headers: Vec<SectionHeader>,
    bytes: &'static [u8],
}

// { 0x48, 0xC7, 0xC0, 0x3C, 0x00, 0x00, 0x00, 0x48, 0x31, 0xFF, 0x0F, 0x05 }
// const TEST_PROGRAM: [u8; 12] = [ 0x48, 0xC7, 0xC0, 0x3C, 0x00, 0x00, 0x00, 0x48, 0x31, 0xFF, 0x0F, 0x05 ];
const TEST_PROGRAM: [u8; 10] = [  0xb8, 0x3c,0x00, 0x00, 0x00,0x48,  0x31, 0xff, 0x0f, 0x05, ];

impl ElfFile {
    pub fn new() -> Self {
        let p_headers = vec![ProgramHeader::new(0x1000, TEST_PROGRAM.len() as u64)];

        Self {
            header: ElfHeader::new(0x400000, p_headers.len() as u16),
            p_headers,
            bytes: &TEST_PROGRAM,
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

        w.write_all(self.bytes).unwrap();



        // write!(w, "{}", self.header.serialize().unwrap()).unwrap();
    }
}
