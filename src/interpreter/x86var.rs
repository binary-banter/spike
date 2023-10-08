use crate::interpreter::IO;

use crate::language::x86var::{Block, Instr, Reg, VarArg, X86VarProgram};
use crate::passes::uniquify::UniqueSym;
use std::collections::HashMap;

struct X86Interpreter<'p, I: IO> {
    blocks: &'p HashMap<&'p str, Block<'p, VarArg<'p>>>,
    io: &'p mut I,
    regs: HashMap<Reg, i64>,
    vars: HashMap<UniqueSym<'p>, i64>,
    memory: HashMap<i64, i64>,
}

impl<'p> X86VarProgram<'p> {
    pub fn interpret(&self, entry: &'p str, io: &mut impl IO) -> i64 {
        let mut state = X86Interpreter {
            blocks: &self.blocks,
            io,
            regs: HashMap::from([(Reg::RBP, i64::MAX - 7), (Reg::RSP, i64::MAX - 7)]),
            vars: HashMap::default(),
            memory: HashMap::default(),
        };

        state.interpret_block(&state.blocks[&entry])
    }
}

impl<'p, I: IO> X86Interpreter<'p, I> {
    fn interpret_block(&mut self, block: &'p Block<VarArg>) -> i64 {
        for instr in &block.instrs {
            match instr {
                Instr::Addq { src, dst } => {
                    self.set_arg(dst, self.get_arg(src) + self.get_arg(dst))
                }
                Instr::Subq { src, dst } => {
                    self.set_arg(dst, self.get_arg(dst) - self.get_arg(src))
                }
                Instr::Negq { dst } => self.set_arg(dst, -self.get_arg(dst)),
                Instr::Movq { src, dst } => self.set_arg(dst, self.get_arg(src)),
                Instr::Pushq { src } => {
                    let rsp = self.regs.get_mut(&Reg::RSP).unwrap();
                    assert_eq!(*rsp % 8, 0, "Misaligned stack pointer.");
                    *rsp -= 8;
                    self.memory.insert(*rsp, self.get_arg(src));
                }
                Instr::Popq { dst } => {
                    let rsp = self.regs[&Reg::RSP];
                    assert_eq!(rsp % 8, 0, "Misaligned stack pointer.");
                    self.set_arg(dst, self.memory[&rsp]);
                    *self.regs.get_mut(&Reg::RSP).unwrap() += 8;
                }
                Instr::Jmp { lbl } => {
                    return self.interpret_block(&self.blocks[lbl]);
                }
                Instr::Callq { lbl, arity } => match (*lbl, arity) {
                    ("_read_int", 0) => {
                        self.regs.insert(Reg::RAX, self.io.read());
                    }
                    ("_print_int", 1) => {
                        self.io.print(self.regs[&Reg::RDI]);
                    }
                    ("exit", 1) => {
                        break;
                    }
                    _ => todo!(),
                },
                Instr::Retq => break, // todo: not quite correct
                Instr::Syscall => unreachable!(),
                Instr::Divq { divisor } => {
                    let rax = self.regs[&Reg::RAX];
                    let rdx = self.regs[&Reg::RDX];
                    let dividend = ((rax as i128) << 64) | rdx as i128;
                    let divisor = self.get_arg(divisor) as i128;

                    self.regs.insert(Reg::RAX, (dividend / divisor) as i64);
                    self.regs.insert(Reg::RDX, (dividend % divisor) as i64);
                }
                Instr::Mulq { src } => {
                    let rax = self.regs[&Reg::RAX] as i128;
                    let src = self.get_arg(src) as i128;

                    let res = rax * src;

                    self.regs.insert(Reg::RAX, (res & (-1i64 as i128)) as i64);
                    self.regs.insert(Reg::RDX, (res >> 64) as i64);
                }
                Instr::Jcc { .. } => todo!(),
            }
        }
        self.regs[&Reg::RAX]
    }

    fn get_arg(&self, a: &'p VarArg) -> i64 {
        match a {
            VarArg::Imm { val } => *val,
            VarArg::Reg { reg } => self.regs[reg],
            VarArg::Deref { reg, off } => self.memory[&(self.regs[reg] + off)],
            VarArg::XVar { sym } => self.vars[sym],
        }
    }

    fn set_arg(&mut self, a: &'p VarArg, v: i64) {
        match a {
            VarArg::Imm { .. } => panic!("Tried to write to immediate, are u insane?"),
            VarArg::Reg { reg } => {
                self.regs.insert(*reg, v);
            }
            VarArg::Deref { reg, off } => {
                self.memory.insert(self.regs[reg] + off, v);
            }
            VarArg::XVar { sym } => {
                self.vars.insert(*sym, v);
            }
        }
    }
}
