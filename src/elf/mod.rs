#![allow(unused)]

use std::io::Write;
use std::mem;
use crate::elf::Bitness::Bitness64;
use crate::elf::Endianness::{BigEndian, LittleEndian};
use bitflags::bitflags;
use std::mem::size_of;
use zerocopy::AsBytes;

#[derive(AsBytes, Copy, Clone)]
#[repr(C, packed)]
struct ElfHeader {
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

#[derive(AsBytes, Copy, Clone)]
#[repr(C, packed)]
struct ElfIdentifier {
    ei_magic: [u8; 4],
    ei_class: Bitness,
    ei_data: Endianness,
    ei_version: u8,
    ei_osabi: OSABI,
    ei_abiversion: u8,
    ei_pad: [u8; 7],
}

#[derive(AsBytes, Copy, Clone)]
#[repr(u8)]
enum Bitness {
    Bitness32 = 1,
    Bitness64 = 2,
}

#[derive(AsBytes, Copy, Clone)]
#[repr(u8)]
enum Endianness {
    LittleEndian = 1,
    BigEndian = 2,
}

#[derive(AsBytes, Copy, Clone)]
#[repr(u8)]
enum OSABI {
    NoneSystemV = 0,
    HPUX = 1,
    NetBSD = 2,
    Linux = 3,
}

#[derive(AsBytes, Copy, Clone)]
#[repr(u16)]
enum ObjectType {
    Unknown = 0x00,
    Relocatable = 0x01,
    Executable = 0x02,
    Shared = 0x03,
    Core = 0x04,
}

#[derive( AsBytes, Copy, Clone)]
#[repr(u16)]
enum Machine {
    X86 = 0x03,
    MIPS = 0x08,
    ARM = 0x28,
    AMD64 = 0x3E,
    ARMV8 = 0xB7,
    RISCV = 0xF3,
}

const ELF_MAGIC: [u8; 4] = [0x7F, b'E', b'L', b'F'];

impl ElfIdentifier {
    fn new() -> Self {
        Self {
            ei_magic: ELF_MAGIC,
            ei_class: Bitness64,
            ei_data: LittleEndian,
            ei_version: 1,
            ei_osabi: OSABI::Linux,
            ei_abiversion: 0,
            ei_pad: [0x00; 7],
        }
    }
}

impl ElfHeader {
    pub fn new(entry: u64, ph_offset: u64, ph_num: u16, sh_offset: u64, sh_num: u16, section_header_table_index: u16) -> Self {
        assert_eq!(size_of::<Self>(), 64);

        Self {
            e_ident: ElfIdentifier::new(),
            e_type: ObjectType::Executable,
            e_machine: Machine::X86,
            e_version: 1,
            e_entry: entry,
            e_phoff: ph_offset,
            e_shoff: sh_offset,
            e_flags: 0,
            e_ehsize: size_of::<ElfHeader>() as u16,
            e_phentsize: size_of::<ProgramHeader>() as u16,
            e_phnum: ph_num,
            e_shentsize: size_of::<SectionHeader>() as u16,
            e_shnum: sh_num,
            e_shstrndx: section_header_table_index,
        }
    }
}

bitflags! {
    #[repr(transparent)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    struct PFlags: u32 {
        const Executable = 0x01;
        const Writable = 0x02;
        const Readable = 0x04;
    }
}

/// The program header table tells the system how to create a process image. It is found at file offset e_phoff,
/// and consists of e_phnum entries, each with size e_phentsize. The layout is slightly different in
/// 32-bit ELF vs 64-bit ELF, because the p_flags are in a different structure location for alignment reasons.
/// Each entry is structured as:
#[repr(C, packed)]
struct ProgramHeader {
    /// Identifies the type of the segment.
    p_type: u32,
    /// Segment-dependent flags (position for 64-bit structure).
    p_flags: PFlags,
    /// Offset of the segment in the file image.
    p_offset: u64,
    /// Virtual address of the segment in memory.
    p_vaddr: u64,
    /// On systems where physical address is relevant, reserved for segment's physical address.
    p_paddr: u64,
    /// Size in bytes of the segment in the file image. May be 0.
    p_filesz: u64,
    /// Size in bytes of the segment in memory. May be 0.
    p_memsz: u64,
    /// 0 and 1 specify no alignment.
    /// Otherwise should be a positive, integral power of 2, with p_vaddr equating p_offset modulus p_align.
    p_align: u64,
}

impl ProgramHeader {
    pub fn new() -> Self {
        assert_eq!(size_of::<ProgramHeader>(), 0x38);


        todo!()
    }
}

#[derive(AsBytes, Copy, Clone)]
#[repr(u32)]
#[allow(non_camel_case_types)]
enum SectionType{
    /// Section header table entry unused
    SHT_NULL = 0x00,
    /// Program data
    SHT_PROGBITS = 0x01,
    ///	Symbol table
    SHT_SYMTAB = 0x02,
    /// String table
    SHT_STRTAB = 0x03,
    /// Relocation entries with addends
    SHT_RELA = 0x04,
    /// Symbol hash table
    SHT_HASH = 0x05,
    /// Dynamic linking information
    SHT_DYNAMIC = 0x06,
    /// Notes
    SHT_NOTE = 0x07,
    /// Program space with no data (bss)
    SHT_NOBITS = 0x08,
    /// Relocation entries, no addends
    SHT_REL = 0x09,
    /// Reserved
    SHT_SHLIB = 0x0A,
    /// Dynamic linker symbol table
    SHT_DYNSYM = 0x0B,
    /// Array of constructors
    SHT_INIT_ARRAY = 0x0E,
    /// Array of destructors
    SHT_FINI_ARRAY = 0x0F,
    /// Array of pre-constructors
    SHT_PREINIT_ARRAY = 0x10,
    /// Section group
    SHT_GROUP = 0x11,
    /// Extended section indices
    SHT_SYMTAB_SHNDX = 0x12,
    /// Number of defined types.
    SHT_NUM = 0x13,
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    struct SectionFlags: u64 {
        const SHF_WRITE = 0x01;
        const SHF_ALLOC = 0x02;
        const SHF_EXECINSTR = 0x04;
        const SHF_MERGE = 0x10;
        const SHF_STRINGS = 0x20;
        const SHF_INFO_LINK = 0x40;
        const SHF_LINK_ORDER = 0x80;
        const SHF_OS_NONCONFORMING = 0x100;
        const SHF_GROUP = 0x200;
        const SHF_TLS = 0x400;
    }
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
struct SectionHeader {
    /// An offset to a string in the .shstrtab section that represents the name of this section.
    sh_name: u32,
    /// Identifies the type of this header.
    sh_type: SectionType,
    /// Identifies the attributes of the section.
    sh_flags: SectionFlags,
    /// Virtual address of the section in memory, for sections that are loaded.
    sh_addr: u64,
    /// Offset of the section in the file image.
    sh_offset: u64,
    /// Size in bytes of the section in the file image. May be 0.
    sh_link: u32,
    ///  Contains the section index of an associated section.
    ///  This field is used for several purposes, depending on the type of section.
    sh_info: u32,
    /// Contains extra information about the section.
    /// This field is used for several purposes, depending on the type of section.
    sh_addralign: u64,
    /// Contains the required alignment of the section. This field must be a power of two.
    shentsize: u64,
}

impl SectionHeader {
    pub fn new() -> Self {
        assert_eq!(size_of::<ProgramHeader>(), 0x40);

        todo!()
    }
}

#[derive(AsBytes, Copy, Clone)]
#[repr(C, packed)]
pub struct ElfFile {
    header: ElfHeader,
    // p_headers: Vec<ProgramHeader>,
    // s_headers: Vec<SectionHeader>,
}

impl ElfFile {
    pub fn new() -> Self {
        Self {
            header: ElfHeader::new(0, 0, 0, 0, 0, 0),
        }
    }

    pub fn emit(self, w: &mut impl Write) {
        // let header_bytes: [u8; 64] = unsafe { mem::transmute(self.header) };
        w.write_all(self.header.as_bytes());

        // write!(w, "{}", self.header.serialize().unwrap()).unwrap();
    }
}


