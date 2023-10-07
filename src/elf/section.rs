use bitflags::bitflags;
use std::mem::size_of;
use zerocopy::AsBytes;

#[derive(AsBytes, Copy, Clone)]
#[repr(u32)]
#[allow(non_camel_case_types)]
enum SectionType {
    /// Section header table entry unused
    SHT_NULL = 0x00,
    /// Program data
    SHT_PROGBITS = 0x01,
    /// Symbol table
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

#[derive(AsBytes, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
struct SectionFlags(u64);

bitflags! {
    impl SectionFlags: u64 {
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

#[derive(AsBytes, Copy, Clone)]
#[repr(C, packed)]
pub struct SectionHeader {
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
    sh_size: u64,
    ///  Contains the section index of an associated section.
    ///  This field is used for several purposes, depending on the type of section.
    sh_link: u32,
    /// Contains extra information about the section.
    /// This field is used for several purposes, depending on the type of section.
    sh_info: u32,
    /// Contains the required alignment of the section. This field must be a power of two.
    sh_addralign: u64,
    /// Contains the size, in bytes, of each entry, for sections that contain fixed-size entries.
    /// Otherwise, this field contains zero.
    sh_entsize: u64,
}

impl SectionHeader {
    pub fn new() -> Self {
        assert_eq!(size_of::<SectionHeader>(), 0x40);

        todo!()
    }
}
