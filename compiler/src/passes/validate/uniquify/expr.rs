use crate::passes::parse::{Expr, ExprParsed, InstrParsed, Meta, Spanned};
use crate::passes::select::VarArg;
use crate::passes::validate::error::TypeError;
use crate::passes::validate::uniquify::r#type::uniquify_type;
use crate::passes::validate::uniquify::{gen_spanned_sym, try_get};
use crate::passes::validate::{uniquify, ExprUniquified, InstrUniquified};
use crate::utils::push_map::PushMap;
use crate::utils::unique_sym::UniqueSym;
use crate::*;

pub fn uniquify_expr<'p>(
    expr: Spanned<ExprParsed<'p>>,
    scope: &mut PushMap<&'p str, UniqueSym<'p>>,
) -> Result<Spanned<ExprUniquified<'p>>, TypeError> {
    let inner = match expr.inner {
        Expr::Let {
            sym,
            typ,
            bnd,
            bdy,
            mutable,
        } => {
            let unique_bnd = uniquify_expr(*bnd, scope)?;
            let unique_sym = gen_spanned_sym(sym.clone());
            let unique_bdy = scope.push(sym.inner, unique_sym.inner, |scope| {
                uniquify_expr(*bdy, scope)
            })?;

            Expr::Let {
                sym: unique_sym,
                mutable,
                typ: typ.map(|typ| uniquify_type(typ, scope)).transpose()?,
                bnd: Box::new(unique_bnd),
                bdy: Box::new(unique_bdy),
            }
        }
        Expr::Var { sym } => Expr::Var {
            sym: uniquify::try_get(sym, scope)?,
        },
        Expr::Assign { sym, bnd } => Expr::Assign {
            sym: uniquify::try_get(sym, scope)?,
            bnd: Box::new(uniquify_expr(*bnd, scope)?),
        },
        Expr::Struct { sym, fields } => Expr::Struct {
            sym: uniquify::try_get(sym, scope)?,
            fields: fields
                .into_iter()
                .map(|(sym, expr)| uniquify_expr(expr, scope).map(|expr| (sym, expr)))
                .collect::<Result<_, _>>()?,
        },

        Expr::Lit { val } => Expr::Lit { val },
        Expr::UnaryOp { op, expr } => Expr::UnaryOp {
            op,
            expr: Box::new(uniquify_expr(*expr, scope)?),
        },
        Expr::BinaryOp {
            op,
            exprs: [e1, e2],
        } => Expr::BinaryOp {
            op,
            exprs: [uniquify_expr(*e1, scope)?, uniquify_expr(*e2, scope)?].map(Box::new),
        },
        Expr::If { cnd, thn, els } => Expr::If {
            cnd: Box::new(uniquify_expr(*cnd, scope)?),
            thn: Box::new(uniquify_expr(*thn, scope)?),
            els: Box::new(uniquify_expr(*els, scope)?),
        },
        Expr::Apply { fun, args } => Expr::Apply {
            fun: Box::new(uniquify_expr(*fun, scope)?),
            args: args
                .into_iter()
                .map(|arg| uniquify_expr(arg, scope))
                .collect::<Result<_, _>>()?,
        },
        Expr::Loop { bdy } => Expr::Loop {
            bdy: Box::new(uniquify_expr(*bdy, scope)?),
        },
        Expr::Break { bdy } => Expr::Break {
            bdy: Box::new(uniquify_expr(*bdy, scope)?),
        },
        Expr::Seq { stmt, cnt } => Expr::Seq {
            stmt: Box::new(uniquify_expr(*stmt, scope)?),
            cnt: Box::new(uniquify_expr(*cnt, scope)?),
        },
        Expr::Continue => Expr::Continue,
        Expr::Return { bdy } => Expr::Return {
            bdy: Box::new(uniquify_expr(*bdy, scope)?),
        },
        Expr::AccessField { strct, field } => Expr::AccessField {
            strct: Box::new(uniquify_expr(*strct, scope)?),
            field,
        },
        Expr::Variant { .. } => todo!(),
        Expr::Switch { .. } => todo!(),
        ExprParsed::Asm { instrs } => ExprUniquified::Asm {
            instrs: instrs
                .into_iter()
                .map(|instr| uniquify_instr(instr, scope))
                .collect::<Result<_, _>>()?,
        },
    };

    Ok(Meta {
        inner,
        meta: expr.meta,
    })
}

fn uniquify_instr<'p>(
    instr: InstrParsed<'p>,
    scope: &PushMap<&'p str, UniqueSym<'p>>,
) -> Result<InstrUniquified<'p>, TypeError> {
    let map = |arg: VarArg<Spanned<&'p str>>| {
        Ok(match arg {
            VarArg::Imm(imm) => VarArg::Imm(imm),
            VarArg::Reg(reg) => VarArg::Reg(reg),
            VarArg::Deref { reg, off } => VarArg::Deref { reg, off },
            VarArg::XVar(sym) => VarArg::XVar(try_get(sym, scope)?),
        })
    };

    let instr = match instr {
        InstrParsed::Add { src, dst, size } => InstrUniquified::Add {
            src: map(src)?,
            dst: map(dst)?,
            size,
        },
        InstrParsed::Sub { src, dst, size } => InstrUniquified::Sub {
            src: map(src)?,
            dst: map(dst)?,
            size,
        },
        InstrParsed::Div { divisor, size } => InstrUniquified::Div {
            divisor: map(divisor)?,
            size,
        },
        InstrParsed::IDiv { divisor, size } => InstrUniquified::IDiv {
            divisor: map(divisor)?,
            size,
        },
        InstrParsed::Mul { src, size } => InstrUniquified::Mul {
            src: map(src)?,
            size,
        },
        InstrParsed::IMul { src, size } => InstrUniquified::IMul {
            src: map(src)?,
            size,
        },
        InstrParsed::Neg { dst, size } => InstrUniquified::Neg {
            dst: map(dst)?,
            size,
        },
        InstrParsed::Mov { src, dst, size } => InstrUniquified::Mov {
            src: map(src)?,
            dst: map(dst)?,
            size,
        },
        InstrParsed::MovSX { src, dst, size } => InstrUniquified::MovSX {
            src: map(src)?,
            dst: map(dst)?,
            size,
        },
        InstrParsed::Push { src, size } => InstrUniquified::Push {
            src: map(src)?,
            size,
        },
        InstrParsed::Pop { dst, size } => InstrUniquified::Pop {
            dst: map(dst)?,
            size,
        },

        InstrParsed::Syscall { arity } => InstrUniquified::Syscall { arity },
        InstrParsed::Cmp { src, dst, size } => InstrUniquified::Cmp {
            src: map(src)?,
            dst: map(dst)?,
            size,
        },
        InstrParsed::And { src, dst, size } => InstrUniquified::And {
            src: map(src)?,
            dst: map(dst)?,
            size,
        },
        InstrParsed::Or { src, dst, size } => InstrUniquified::Or {
            src: map(src)?,
            dst: map(dst)?,
            size,
        },
        InstrParsed::Xor { src, dst, size } => InstrUniquified::Xor {
            src: map(src)?,
            dst: map(dst)?,
            size,
        },
        InstrParsed::Not { dst, size } => InstrUniquified::Not {
            dst: map(dst)?,
            size,
        },
        InstrParsed::Setcc { .. }
        | InstrParsed::Ret { .. }
        | InstrParsed::Jmp { .. }
        | InstrParsed::Jcc { .. }
        | InstrParsed::LoadLbl { .. }
        | InstrParsed::CallDirect { .. }
        | InstrParsed::CallIndirect {.. } => unreachable!(),
    };

    Ok(instr)
}
