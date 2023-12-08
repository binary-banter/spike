#[macro_export]
macro_rules! block {
        ($($instr:expr),*) => {
            $crate::passes::select::Block { instrs: vec![$($instr),*] }
        };
    }

#[macro_export]
macro_rules! add {
    ($src:expr, $dst:expr, $size:expr) => {
        $crate::passes::select::Instr::Add {
            src: $src,
            dst: $dst,
            size: $size,
        }
    };
}

#[macro_export]
macro_rules! sub {
    ($src:expr, $dst:expr, $size:expr) => {
        $crate::passes::select::Instr::Sub {
            src: $src,
            dst: $dst,
            size: $size,
        }
    };
}

#[macro_export]
macro_rules! div {
    ($divisor:expr, $size:expr) => {
        $crate::passes::select::Instr::Div {
            divisor: $divisor,
            size: $size,
        }
    };
}

#[macro_export]
macro_rules! idiv {
    ($divisor:expr, $size:expr) => {
        $crate::passes::select::Instr::IDiv {
            divisor: $divisor,
            size: $size,
        }
    };
}

#[macro_export]
macro_rules! mul {
    ($src:expr, $size:expr) => {
        $crate::passes::select::Instr::Mul {
            src: $src,
            size: $size,
        }
    };
}

#[macro_export]
macro_rules! imul {
    ($src:expr, $size:expr) => {
        $crate::passes::select::Instr::IMul {
            src: $src,
            size: $size,
        }
    };
}

#[macro_export]
macro_rules! neg {
    ($dst:expr, $size:expr) => {
        $crate::passes::select::Instr::Neg {
            dst: $dst,
            size: $size,
        }
    };
}

#[macro_export]
macro_rules! mov {
    ($src:expr, $dst:expr, $size:expr) => {
        $crate::passes::select::Instr::Mov {
            src: $src,
            dst: $dst,
            size: $size,
        }
    };
}

#[macro_export]
macro_rules! movsx {
    ($src:expr, $dst:expr, $size:expr) => {
        $crate::passes::select::Instr::MovSX {
            src: $src,
            dst: $dst,
            size: $size,
        }
    };
}

#[macro_export]
macro_rules! push {
    ($src:expr, $size:expr) => {
        $crate::passes::select::Instr::Push {
            src: $src,
            size: $size,
        }
    };
}

#[macro_export]
macro_rules! pop {
    ($dst:expr, $size:expr) => {
        $crate::passes::select::Instr::Pop {
            dst: $dst,
            size: $size,
        }
    };
}

#[macro_export]
macro_rules! ret {
    ($arity:expr) => {
        $crate::passes::select::Instr::Ret { arity: $arity }
    };
}

#[macro_export]
macro_rules! syscall {
    ($arity:expr) => {
        $crate::passes::select::Instr::Syscall { arity: $arity }
    };
}

#[macro_export]
macro_rules! cmp {
    ($src:expr, $dst:expr, $size:expr) => {
        $crate::passes::select::Instr::Cmp {
            src: $src,
            dst: $dst,
            size: $size,
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
macro_rules! jcc {
    ($lbl:expr, $cnd:expr) => {
        $crate::passes::select::Instr::Jcc {
            lbl: $lbl,
            cnd: $cnd,
        }
    };
}

#[macro_export]
macro_rules! and {
    ($src:expr, $dst:expr, $size:expr) => {
        $crate::passes::select::Instr::And {
            src: $src,
            dst: $dst,
            size: $size,
        }
    };
}

#[macro_export]
macro_rules! or {
    ($src:expr, $dst:expr, $size:expr) => {
        $crate::passes::select::Instr::Or {
            src: $src,
            dst: $dst,
            size: $size,
        }
    };
}

#[macro_export]
macro_rules! xor {
    ($src:expr, $dst:expr, $size:expr) => {
        $crate::passes::select::Instr::Xor {
            src: $src,
            dst: $dst,
            size: $size,
        }
    };
}

#[macro_export]
macro_rules! not {
    ($dst:expr, $size:expr) => {
        $crate::passes::select::Instr::Not {
            dst: $dst,
            size: $size,
        }
    };
}

#[macro_export]
macro_rules! setcc {
    ($cnd:expr) => {
        $crate::passes::select::Instr::Setcc { cnd: $cnd }
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
macro_rules! call_direct {
    ($lbl:expr, $arity:expr) => {
        $crate::passes::select::Instr::CallDirect {
            lbl: $lbl,
            arity: $arity,
        }
    };
}

#[macro_export]
macro_rules! call_indirect {
    ($src:expr, $arity:expr) => {
        $crate::passes::select::Instr::CallIndirect {
            src: $src,
            arity: $arity,
        }
    };
}

#[macro_export]
macro_rules! imm {
    ($val:expr) => {
        $crate::passes::assign::Arg::Imm($crate::passes::validate::Int::I64($val as i64)).into()
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
