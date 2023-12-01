use crate::passes::emit::elf::program::ProgramHeader;
use crate::passes::emit::elf::section::SectionHeader;
use std::clone::Clone;
use std::mem::size_of;
use zerocopy::AsBytes;

#[repr(C, packed)]
#[derive(AsBytes, Copy, Clone)]
pub struct ElfHeader {
    e_ident: ElfIdentifier,
    e_type: ObjectType,
    e_machine: Machine,
    e_version: u32,
    e_entry: u64,
    e_phoff: u64,
    e_shoff: u64,
    e_flags: u32,
    e_ehsize: u16,
    e_phentsize: u16,
    e_phnum: u16,
    e_shentsize: u16,
    e_shnum: u16,
    e_shstrndx: u16,
}

#[repr(C, packed)]
#[derive(AsBytes, Copy, Clone)]
struct ElfIdentifier {
    ei_magic: [u8; 4],
    ei_class: Bitness,
    ei_data: Endianness,
    ei_version: u8,
    ei_osabi: OSABI,
    ei_abiversion: u8,
    ei_pad: [u8; 7],
}

#[allow(unused)]
#[repr(u8)]
#[derive(AsBytes, Copy, Clone)]
pub enum Bitness {
    Bitness32 = 1,
    Bitness64 = 2,
}

#[allow(unused)]
#[repr(u8)]
#[derive(AsBytes, Copy, Clone)]
pub enum Endianness {
    LittleEndian = 1,
    BigEndian = 2,
}

#[allow(unused)]
#[repr(u8)]
#[derive(AsBytes, Copy, Clone)]
enum OSABI {
    NoneSystemV = 0,
    HPUX = 1,
    NetBSD = 2,
    Linux = 3,
}

#[allow(unused)]
#[repr(u16)]
#[derive(AsBytes, Copy, Clone)]
enum ObjectType {
    Unknown = 0x00,
    Relocatable = 0x01,
    Executable = 0x02,
    Shared = 0x03,
    Core = 0x04,
}

#[allow(unused)]
#[repr(u16)]
#[derive(AsBytes, Copy, Clone)]
enum Machine {
    X86 = 0x03,
    MIPS = 0x08,
    ARM = 0x28,
    AMD64 = 0x3E,
    ARMV8 = 0xB7,
    RISCV = 0xF3,
}

const ELF_MAGIC: [u8; 4] = *b"\x7FELF";

impl ElfIdentifier {
    fn new() -> Self {
        Self {
            ei_magic: ELF_MAGIC,
            ei_class: Bitness::Bitness64,
            ei_data: Endianness::LittleEndian,
            ei_version: 1,
            ei_osabi: OSABI::NoneSystemV,
            ei_abiversion: 0,
            ei_pad: [0x00; 7],
        }
    }
}

impl ElfHeader {
    pub fn new(entry: u64, ph_num: u16) -> Self {
        assert_eq!(size_of::<Self>(), 64);

        Self {
            e_ident: ElfIdentifier::new(),
            e_type: ObjectType::Executable,
            e_machine: Machine::AMD64,
            e_version: 0x01,
            e_entry: entry,
            e_phoff: 0x40, // The program headers immediately follow the elf-header.
            e_shoff: 0,
            e_flags: 0x00,
            e_ehsize: size_of::<ElfHeader>() as u16,
            e_phentsize: size_of::<ProgramHeader>() as u16,
            e_phnum: ph_num,
            e_shentsize: size_of::<SectionHeader>() as u16,
            e_shnum: 0,
            e_shstrndx: 0,
        }
    }
}
