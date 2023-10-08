use bitflags::bitflags;
use std::mem::size_of;
use zerocopy::AsBytes;

#[allow(unused)]
#[allow(non_camel_case_types)]
#[repr(u32)]
#[derive(AsBytes)]
enum ProgramType {
    /// Program header table entry unused.
    PT_NULL,
    /// Loadable segment.
    PT_LOAD,
    /// Dynamic linking information.
    PT_DYNAMIC,
    /// Interpreter information.
    PT_INTERP,
    /// Auxiliary information.
    PT_NOTE,
    /// Reserved.
    PT_SHLIB,
    /// Segment containing program header table itself.
    PT_PHDR,
    /// Thread-Local Storage template.
    PT_TLS,
}

#[repr(transparent)]
#[derive(AsBytes, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct ProgramFlags(u32);

bitflags! {
    impl ProgramFlags: u32 {
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
#[derive(AsBytes)]
pub struct ProgramHeader {
    /// Identifies the type of the segment.
    p_type: ProgramType,
    /// Segment-dependent flags (position for 64-bit structure).
    p_flags: ProgramFlags,
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
    pub fn new(offset: u64, len: u64) -> Self {
        assert_eq!(size_of::<ProgramHeader>(), 0x38);

        Self {
            p_type: ProgramType::PT_LOAD,
            p_flags: ProgramFlags::Executable | ProgramFlags::Readable,
            p_offset: offset,
            p_vaddr: 0x400000,
            p_paddr: 0x400000,
            p_filesz: len,
            p_memsz: len,
            p_align: 0,
        }
    }
}
