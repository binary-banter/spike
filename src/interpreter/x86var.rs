use crate::interpreter::IO;
use crate::language::x86var::{Block, Cmd, Instr, Reg, VarArg, X86VarProgram};
use std::collections::HashMap;

struct X86Interpreter<'program, I: IO> {
    blocks: HashMap<&'program str, &'program Block<VarArg>>,
    io: &'program mut I,
    regs: HashMap<Reg, i64>,
    vars: HashMap<&'program str, i64>,
    memory: HashMap<i64, i64>,
}

pub fn interpret_x86var(entry: &str, program: &X86VarProgram, io: &mut impl IO) -> i64 {
    let blocks = program
        .blocks
        .iter()
        .map(|(name, block)| (name.as_str(), block))
        .collect::<HashMap<_, _>>();

    let mut state = X86Interpreter {
        blocks,
        io,
        regs: HashMap::from([(Reg::RBP, i64::MAX), (Reg::RSP, i64::MAX)]),
        vars: HashMap::default(),
        memory: HashMap::default(),
    };

    state.interpret_block(state.blocks[entry])
}

impl<'program, I: IO> X86Interpreter<'program, I> {
    fn interpret_block(&mut self, block: &'program Block<VarArg>) -> i64 {
        for instr in &block.instrs {
            match instr {
                Instr::Instr { cmd, args } => match (cmd, args.as_slice()) {
                    (Cmd::Addq, [a1, a2]) => self.set_arg(a2, self.get_arg(a1) + self.get_arg(a2)),
                    (Cmd::Subq, [a1, a2]) => self.set_arg(a2, self.get_arg(a2) - self.get_arg(a1)),
                    (Cmd::Negq, [a1]) => self.set_arg(a1, -self.get_arg(a1)),
                    (Cmd::Movq, [a1, a2]) => self.set_arg(a2, self.get_arg(a1)),
                    (Cmd::Pushq, [a1]) => {
                        let rsp = self.regs.get_mut(&Reg::RSP).unwrap();
                        assert_eq!(*rsp % 8, 0, "Misaligned stack pointer.");
                        *rsp -= 8;
                        self.memory.insert(*rsp, self.get_arg(a1));
                    }
                    (Cmd::Popq, [a1]) => {
                        let rsp = self.regs[&Reg::RSP];
                        assert_eq!(rsp % 8, 0, "Misaligned stack pointer.");
                        self.set_arg(a1, self.memory[&rsp]);
                        *self.regs.get_mut(&Reg::RSP).unwrap() += 8;
                    }
                    (_, _) => panic!("Unsupported `Instr`."),
                },
                Instr::Jmp { lbl } => {
                    return self.interpret_block(self.blocks[lbl.as_str()]);
                }
                Instr::Callq { lbl, arity } => match (lbl.as_str(), arity) {
                    ("_read_int", 0) => {
                        self.regs.insert(Reg::RAX, self.io.read());
                    }
                    ("_print_int", 1) => {
                        self.io.print(self.regs[&Reg::RDI]);
                    }
                    _ => todo!(),
                },
                Instr::Retq => todo!(),
            }
        }
        self.regs[&Reg::RAX]
    }

    fn get_arg(&self, a: &'program VarArg) -> i64 {
        match a {
            VarArg::Imm { val } => *val,
            VarArg::Reg { reg } => self.regs[reg],
            VarArg::Deref { reg, off } => self.memory[&(self.regs[reg] + off)],
            VarArg::XVar { sym } => self.vars[sym.as_str()],
        }
    }

    fn set_arg(&mut self, a: &'program VarArg, v: i64) {
        match a {
            VarArg::Imm { .. } => panic!("Tried to write to immediate, are u insane?"),
            VarArg::Reg { reg } => {
                self.regs.insert(*reg, v);
            }
            VarArg::Deref { reg, off } => {
                self.memory.insert(self.regs[&reg] + off, v);
            }
            VarArg::XVar { sym } => {
                self.vars.insert(sym.as_str(), v);
            }
        }
    }
}
