#[macro_export]
macro_rules! block {
        ($($instr:expr),*) => {
            $crate::passes::select::Block { instrs: vec![$($instr),*] }
        };
    }

#[macro_export]
macro_rules! addq {
    ($src:expr, $dst:expr) => {
        $crate::passes::select::Instr::Addq {
            src: $src,
            dst: $dst,
        }
    };
}

#[macro_export]
macro_rules! divq {
    ($divisor:expr) => {
        $crate::passes::select::Instr::Divq { divisor: $divisor }
    };
}

#[macro_export]
macro_rules! mulq {
    ($src:expr) => {
        $crate::passes::select::Instr::Mulq { src: $src }
    };
}

#[macro_export]
macro_rules! subq {
    ($src:expr, $dst:expr) => {
        $crate::passes::select::Instr::Subq {
            src: $src,
            dst: $dst,
        }
    };
}

#[macro_export]
macro_rules! cmpq {
    ($src:expr, $dst:expr) => {
        $crate::passes::select::Instr::Cmpq {
            src: $src,
            dst: $dst,
        }
    };
}
#[macro_export]
macro_rules! andq {
    ($src:expr, $dst:expr) => {
        $crate::passes::select::Instr::Andq {
            src: $src,
            dst: $dst,
        }
    };
}

#[macro_export]
macro_rules! orq {
    ($src:expr, $dst:expr) => {
        $crate::passes::select::Instr::Orq {
            src: $src,
            dst: $dst,
        }
    };
}

#[macro_export]
macro_rules! xorq {
    ($src:expr, $dst:expr) => {
        $crate::passes::select::Instr::Xorq {
            src: $src,
            dst: $dst,
        }
    };
}

#[macro_export]
macro_rules! notq {
    ($dst:expr) => {
        $crate::passes::select::Instr::Notq { dst: $dst }
    };
}

#[macro_export]
macro_rules! negq {
    ($dst:expr) => {
        $crate::passes::select::Instr::Negq { dst: $dst }
    };
}

#[macro_export]
macro_rules! movq {
    ($src:expr, $dst:expr) => {
        $crate::passes::select::Instr::Movq {
            src: $src,
            dst: $dst,
        }
    };
}

#[macro_export]
macro_rules! load_lbl {
    ($lbl:expr, $dst: expr) => {
        $crate::passes::select::Instr::LoadLbl {
            lbl: $lbl,
            dst: $dst,
        }
    };
}

#[macro_export]
macro_rules! pushq {
    ($src:expr) => {
        $crate::passes::select::Instr::Pushq { src: $src }
    };
}

#[macro_export]
macro_rules! popq {
    ($dst:expr) => {
        $crate::passes::select::Instr::Popq { dst: $dst }
    };
}

#[macro_export]
macro_rules! callq_direct {
    ($lbl:expr, $arity:expr) => {
        $crate::passes::select::Instr::CallqDirect {
            lbl: $lbl,
            arity: $arity,
        }
    };
}

#[macro_export]
macro_rules! callq_indirect {
    ($src:expr, $arity:expr) => {
        $crate::passes::select::Instr::CallqIndirect {
            src: $src,
            arity: $arity,
        }
    };
}

#[macro_export]
macro_rules! jmp {
    ($lbl:expr) => {
        $crate::passes::select::Instr::Jmp { lbl: $lbl }
    };
}

#[macro_export]
macro_rules! setcc {
    ($cnd:expr) => {
        $crate::passes::select::Instr::Setcc { cnd: $cnd }
    };
}

#[macro_export]
macro_rules! jcc {
    ($lbl:expr, $cnd:expr) => {
        $crate::passes::select::Instr::Jcc {
            lbl: $lbl,
            cnd: $cnd,
        }
    };
}

#[macro_export]
macro_rules! retq {
    () => {
        $crate::passes::select::Instr::Retq
    };
}

#[macro_export]
macro_rules! syscall {
    ($arity:expr) => {
        $crate::passes::select::Instr::Syscall { arity: $arity }
    };
}

#[macro_export]
macro_rules! imm32 {
    ($val:expr) => {
        $crate::passes::assign::Arg::Imm($crate::passes::select::Imm::Imm32($val as u32)).into()
    };
}

#[macro_export]
macro_rules! reg {
    ($reg:ident) => {
        $crate::passes::assign::Arg::Reg($crate::passes::select::Reg::$reg).into()
    };
}

#[macro_export]
macro_rules! var {
    ($sym:expr) => {
        $crate::passes::select::VarArg::XVar($sym)
    };
}

#[macro_export]
macro_rules! deref {
    ($reg:ident, $off:expr) => {
        $crate::passes::assign::Arg::Deref {
            reg: $crate::passes::select::Reg::$reg,
            off: $off,
        }
    };
}
