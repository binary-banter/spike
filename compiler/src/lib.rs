#![allow(clippy::module_inception)]

use crate::passes::parse::parse::parse_program;
use std::fs::File;
use std::path::Path;
use crate::passes::emit::elf::ElfFile;

pub mod interpreter;
pub mod passes;
pub mod utils;

pub fn compile(program: &str, output: &Path) -> miette::Result<()> {
    let program = parse_program(program)?
        .type_check()?
        .uniquify()
        .reveal()
        .atomize()
        .explicate()
        .select()
        .add_liveness()
        .compute_interference()
        .color_interference()
        .assign_homes()
        .patch()
        .conclude();

    let (entry, program) = program.emit();

    let elf = ElfFile::new(entry, &program);
    let mut file = File::create(output).unwrap();
    elf.write(&mut file);

    Ok(())
}
