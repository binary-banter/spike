use crate::interpreter::IO;
use crate::passes::conclude::X86Concluded;

use crate::passes::select::{
    Block, Cnd, Instr, Reg, VarArg, X86Selected, CALLEE_SAVED, CALLER_SAVED,
};
use crate::passes::validate::TLit;
use crate::utils::gen_sym::UniqueSym;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;
use std::mem;
use zerocopy::AsBytes;

#[derive(Default)]
pub struct Status {
    /// CF
    carry: bool,
    /// PF
    parity_even: bool,
    /// ZF = 1 (is zero)
    zero: bool,
    /// SF
    sign: bool,
    /// OF
    overflow: bool,
}

/// Stats gathered by the interpreter.
#[derive(Debug, Deserialize, Serialize, Default)]
pub struct IStats {
    pub branches_taken: usize,
    pub instructions_executed: usize,
}

pub struct X86Interpreter<'p, I: IO> {
    pub blocks: &'p HashMap<UniqueSym<'p>, Block<'p, VarArg<'p>>>,
    pub io: &'p mut I,
    pub regs: HashMap<Reg, i64>,

    pub vars: HashMap<UniqueSym<'p>, i64>,
    pub var_stack: Vec<HashMap<UniqueSym<'p>, i64>>,

    pub memory: HashMap<i64, i64>,
    pub block_ids: HashMap<usize, UniqueSym<'p>>,
    pub read_buffer: Vec<u8>,
    pub write_buffer: Vec<u8>,
    pub status: Status,
    pub stats: IStats,
}

impl<'p> X86Concluded<'p> {
    pub fn interpret_with_stats(&self, io: &mut impl IO) -> (i64, IStats) {
        let block_ids = self.blocks.keys().map(|sym| (sym.id, *sym)).collect();

        let mut regs = HashMap::new();
        for reg in CALLEE_SAVED.into_iter().chain(CALLER_SAVED.into_iter()) {
            regs.insert(reg, 0);
        }

        let mut state = X86Interpreter {
            // todo: remove this clone
            blocks: &self
                .blocks
                .clone()
                .into_iter()
                .map(|(sym, block)| (sym, block.fmap(Into::into)))
                .collect(),
            io,
            regs,
            vars: HashMap::default(),
            var_stack: vec![],
            memory: HashMap::default(),
            block_ids,
            read_buffer: Vec::new(),
            write_buffer: Vec::new(),
            status: Status::default(),
            stats: IStats::default(),
        };

        let val = state.interpret_block(self.entry, 0);
        (val, state.stats)
    }

    pub fn interpret(&self, io: &mut impl IO) -> i64 {
        self.interpret_with_stats(io).0
    }
}

impl<'p> X86Selected<'p> {
    pub fn interpret(&self, io: &mut impl IO) -> i64 {
        let block_ids = self.blocks.keys().map(|sym| (sym.id, *sym)).collect();

        let mut regs = HashMap::new();
        for reg in CALLEE_SAVED.into_iter().chain(CALLER_SAVED.into_iter()) {
            regs.insert(reg, 0);
        }

        // We give 0x1000 stack space to test programs - this might not be enough (you weirdos)!
        regs.insert(Reg::RBP, i64::MAX - 7);
        regs.insert(Reg::RSP, (i64::MAX - 7) - 0x1000);

        let mut state = X86Interpreter {
            blocks: &self.blocks,
            io,
            regs,
            vars: HashMap::default(),
            var_stack: vec![],
            memory: HashMap::default(),
            block_ids,
            read_buffer: Vec::new(),
            write_buffer: Vec::new(),
            status: Default::default(),
            stats: IStats::default(),
        };

        state.interpret_block(self.entry, 0)
    }
}

impl<'p, I: IO> X86Interpreter<'p, I> {
    fn instr_to_addr(&self, block_name: UniqueSym, instr_id: usize) -> i64 {
        // Please do not make more than 2^32 blocks or blocks with more than 2^32 instructions!
        ((block_name.id << 32) | instr_id) as i64
    }

    fn interpret_addr(&mut self, addr: i64) -> i64 {
        let block_id = (addr >> 32) as usize;
        let instr_id = (addr & 0xFF_FF_FF_FF) as usize;
        self.interpret_block(self.block_ids[&block_id], instr_id)
    }

    pub fn interpret_block(&mut self, block_name: UniqueSym<'p>, offset: usize) -> i64 {
        let block = &self.blocks[&block_name];

        for (instr_id, instr) in block.instrs.iter().enumerate().skip(offset) {
            self.stats.instructions_executed += 1;
            match instr {
                Instr::Addq { src, dst } => {
                    self.set_arg(dst, self.get_arg(src) + self.get_arg(dst));
                }
                Instr::Subq { src, dst } => {
                    self.set_arg(dst, self.get_arg(dst) - self.get_arg(src));
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
                    return self.interpret_block(*lbl, 0);
                }
                Instr::Retq => {
                    let rsp = self.regs[&Reg::RSP];
                    assert_eq!(rsp % 8, 0, "Misaligned stack pointer.");
                    let addr = self.memory[&rsp];
                    *self.regs.get_mut(&Reg::RSP).unwrap() += 8;

                    // Pop var context
                    self.vars = self.var_stack.pop().expect(
                        "Found more returns than we have had calls so far, ur program is weird m8",
                    );

                    return self.interpret_addr(addr);
                }
                Instr::Syscall { .. } => match self.regs[&Reg::RAX] {
                    0x00 => self.syscall_read(),
                    0x01 => self.syscall_write(),
                    0x3C => {
                        return self.regs[&Reg::RDI];
                    }
                    _ => unreachable!(),
                },
                Instr::Divq { divisor } => {
                    let rax = self.regs[&Reg::RAX];
                    let rdx = self.regs[&Reg::RDX];
                    let dividend = (i128::from(rdx) << 64) | i128::from(rax);
                    let divisor = i128::from(self.get_arg(divisor));

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
                Instr::Jcc { lbl, cnd } => {
                    self.stats.branches_taken += 1;
                    if self.evaluate_cnd(*cnd) {
                        return self.interpret_block(*lbl, 0);
                    }
                }
                Instr::Cmpq { src, dst } => {
                    assert!(
                        !matches!(dst, VarArg::Imm { .. }),
                        "Destination cannot be an immediate."
                    );

                    let src = self.get_arg(src);
                    let dst = self.get_arg(dst);

                    let (res, overflow) = dst.overflowing_sub(src);

                    // Maybe this can be done "prettier", but honestly it works.
                    let src = u64::from_ne_bytes(src.to_ne_bytes());
                    let dst = u64::from_ne_bytes(dst.to_ne_bytes());

                    self.status = Status {
                        carry: src > dst,
                        parity_even: res % 2 == 0,
                        zero: res == 0,
                        sign: res < 0,
                        overflow,
                    }
                }
                Instr::Andq { src, dst } => {
                    self.set_arg(dst, self.get_arg(src) & self.get_arg(dst));
                }
                Instr::Orq { src, dst } => self.set_arg(dst, self.get_arg(src) | self.get_arg(dst)),
                Instr::Xorq { src, dst } => {
                    self.set_arg(dst, self.get_arg(src) ^ self.get_arg(dst));
                }
                Instr::Notq { dst } => self.set_arg(dst, !self.get_arg(dst)),
                Instr::Setcc { cnd } => {
                    let rax = self.regs[&Reg::RAX];
                    let cnd = i64::from(self.evaluate_cnd(*cnd));
                    self.regs.insert(Reg::RAX, rax & !0xFF | cnd);
                }
                Instr::LoadLbl { sym, dst } => {
                    let val = self.instr_to_addr(*sym, 0);
                    self.set_arg(dst, val);
                }
                Instr::CallqDirect { lbl, .. } => {
                    let ret_addr = self.instr_to_addr(block_name, instr_id + 1);

                    let rsp = self.regs.get_mut(&Reg::RSP).unwrap();
                    assert_eq!(*rsp % 8, 0, "Misaligned stack pointer.");
                    *rsp -= 8;
                    self.memory.insert(*rsp, ret_addr);

                    //Push old var context
                    self.var_stack.push(mem::take(&mut self.vars));

                    return self.interpret_block(*lbl, 0);
                }
                Instr::CallqIndirect { src, .. } => {
                    let ret_addr = self.instr_to_addr(block_name, instr_id + 1);

                    let rsp = self.regs.get_mut(&Reg::RSP).unwrap();
                    assert_eq!(*rsp % 8, 0, "Misaligned stack pointer.");
                    *rsp -= 8;
                    self.memory.insert(*rsp, ret_addr);

                    let block = self.get_arg(src);

                    //Push old var context
                    self.var_stack.push(mem::take(&mut self.vars));

                    return self.interpret_addr(block);
                }
            }
        }
        panic!("A block ran out of instructions.");
    }

    fn evaluate_cnd(&self, cnd: Cnd) -> bool {
        match cnd {
            Cnd::Above => !self.status.carry && !self.status.zero,
            Cnd::AboveOrEqual | Cnd::NotCarry => !self.status.carry,
            Cnd::Below | Cnd::Carry => self.status.carry,
            Cnd::BelowOrEqual => self.status.carry || self.status.zero,
            Cnd::EQ => self.status.zero,
            Cnd::GT => !self.status.zero && self.status.sign == self.status.overflow,
            Cnd::GE => self.status.sign == self.status.overflow,
            Cnd::LT => self.status.sign != self.status.overflow,
            Cnd::LE => self.status.zero || self.status.sign != self.status.overflow,
            Cnd::NE => !self.status.zero,
            Cnd::NotOverflow => !self.status.overflow,
            Cnd::NotSign => !self.status.sign,
            Cnd::Overflow => self.status.overflow,
            Cnd::ParityEven => self.status.parity_even,
            Cnd::ParityOdd => !self.status.parity_even,
            Cnd::Sign => self.status.sign,
        }
    }

    fn get_arg(&self, a: &'p VarArg) -> i64 {
        match a {
            VarArg::Imm { val } => *val,
            VarArg::Reg { reg } => self.regs[reg],
            VarArg::Deref { reg, off } => self.memory[&(self.regs[reg] + off)],
            VarArg::XVar { sym } => *self
                .vars
                .get(sym)
                .unwrap_or_else(|| panic!("Expected to find variable {sym}")),
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

    fn syscall_read(&mut self) {
        let file = self.regs[&Reg::RDI];
        assert_eq!(file, 0, "Only reading from stdin is supported right now.");
        let buffer = self.regs[&Reg::RSI];
        let buffer_len = self.regs[&Reg::RDX];
        assert!(buffer_len >= 1);

        if self.read_buffer.is_empty() {
            self.read_buffer = format!("{}\n", self.io.read().int()).into_bytes();
            self.read_buffer.reverse();
        }
        let val = self.read_buffer.pop().unwrap();
        self.memory.insert(buffer, val as i64);
        self.regs.insert(Reg::RAX, 1);
    }

    fn syscall_write(&mut self) {
        let file = self.regs[&Reg::RDI];
        assert_eq!(file, 1, "Only writing to stdout is supported right now.");
        let buffer = self.regs[&Reg::RSI];
        let buffer_len = self.regs[&Reg::RDX];
        assert_eq!(
            buffer_len, 1,
            "Only writing 1 byte at a time is supported right now."
        );

        let val = *self.memory.get(&buffer).unwrap();
        match val as u8 {
            b'\n' => {
                let val = std::str::from_utf8(self.write_buffer.as_bytes())
                    .unwrap()
                    .parse()
                    .unwrap();
                self.io.print(TLit::Int { val });
                self.write_buffer.clear();
            }
            val => {
                self.write_buffer.push(val);
            }
        }

        self.regs.insert(Reg::RAX, 1);
    }
}
