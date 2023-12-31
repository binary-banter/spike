use crate::passes::parse::{Expr, ExprParsed, InstrParsed, Meta, Spanned};
use crate::passes::select::VarArg;
use crate::passes::validate::error::TypeError;
use crate::passes::validate::uniquify::r#type::uniquify_type;
use crate::passes::validate::uniquify::{gen_spanned_sym, try_get};
use crate::passes::validate::{uniquify, ExprUniquified, InstrUniquified};
use crate::utils::gen_sym::UniqueSym;
use crate::utils::push_map::PushMap;
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
            VarArg::Imm { val } => VarArg::Imm { val },
            VarArg::Reg { reg } => VarArg::Reg { reg },
            VarArg::Deref { reg, off } => VarArg::Deref { reg, off },
            VarArg::XVar { sym } => VarArg::XVar {
                sym: try_get(sym, scope)?,
            },
        })
    };

    let instr = match instr {
        InstrParsed::Addq { src, dst } => addq!(map(src)?, map(dst)?),
        InstrParsed::Subq { src, dst } => subq!(map(src)?, map(dst)?),
        InstrParsed::Divq { divisor } => divq!(map(divisor)?),
        InstrParsed::Mulq { src } => mulq!(map(src)?),
        InstrParsed::Negq { dst } => negq!(map(dst)?),
        InstrParsed::Movq { src, dst } => movq!(map(src)?, map(dst)?),
        InstrParsed::Pushq { src } => pushq!(map(src)?),
        InstrParsed::Popq { dst } => popq!(map(dst)?),
        InstrParsed::Retq => retq!(),
        InstrParsed::Syscall { arity } => syscall!(arity),
        InstrParsed::Cmpq { src, dst } => cmpq!(map(src)?, map(dst)?),
        InstrParsed::Andq { src, dst } => andq!(map(src)?, map(dst)?),
        InstrParsed::Orq { src, dst } => orq!(map(src)?, map(dst)?),
        InstrParsed::Xorq { src, dst } => xorq!(map(src)?, map(dst)?),
        InstrParsed::Notq { dst } => notq!(map(dst)?),
        InstrParsed::Setcc { cnd } => setcc!(cnd),
        InstrParsed::CallqDirect { .. } => todo!(),
        InstrParsed::Jmp { .. } => todo!(),
        InstrParsed::Jcc { .. } => todo!(),
        InstrParsed::LoadLbl { .. } => todo!(),
        InstrParsed::CallqIndirect { .. } => todo!(),
    };

    Ok(instr)
}
