use std::fmt::{self, Write};

use num::{One, Signed};

use crate::constant::Constant;
use crate::{BasicAlgebraicExpr, PrecedenceContext};


pub fn print_expr_to_string(x: &BasicAlgebraicExpr) -> String {
    let mut p = Printer::new_string();
    p.print(x).expect("String format does not have errors");
    p.into_inner()
}
pub struct Printer<W: Write> {
    writer: W,
}

fn is_denominator(x: &BasicAlgebraicExpr) -> bool {
    if let BasicAlgebraicExpr::Pow(x) = x
    && let BasicAlgebraicExpr::Numeric(x) = &x.1
    && x.is_negative() {
        true
    } else {
        false
    }
}

fn can_combine_with_next(x: &BasicAlgebraicExpr) -> bool {
    match x {
        BasicAlgebraicExpr::Numeric(_)
        | BasicAlgebraicExpr::Symbol(_)
        | BasicAlgebraicExpr::Sum(_)
        | BasicAlgebraicExpr::Pow(_)
        | BasicAlgebraicExpr::Function(..) => true,
        BasicAlgebraicExpr::Product(x) => x.last().map_or(true, can_combine_with_next),
        BasicAlgebraicExpr::Factorial(_) => false,
    }
}

fn can_combine_with_prev(x: &BasicAlgebraicExpr) -> bool {
    match x {
        BasicAlgebraicExpr::Product(x) => x.first().map_or(true, can_combine_with_prev),
        BasicAlgebraicExpr::Pow(_)
        | BasicAlgebraicExpr::Symbol(_)
        | BasicAlgebraicExpr::Sum(_)
        | BasicAlgebraicExpr::Function(..) => true,
        BasicAlgebraicExpr::Factorial(_) | BasicAlgebraicExpr::Numeric(_) => false,
    }
}

impl<W: Write> Printer<W> {
    pub fn print(&mut self, x: &BasicAlgebraicExpr) -> fmt::Result {
        self.print_with_precedence(x, PrecedenceContext::NoPrecedence)
    }

    pub fn maybe_enter_parens(
        &mut self,
        f: impl FnOnce(&mut Self) -> fmt::Result,
        parens: bool,
    ) -> fmt::Result {
        if parens {
            self.writer.write_char('(')?;
        }
        f(self)?;
        if parens {
            self.writer.write_char(')')?;
        }
        Ok(())
    }

    pub fn enter_parens(&mut self, f: impl FnOnce(&mut Self) -> fmt::Result) -> fmt::Result {
        self.writer.write_char('(')?;
        f(self)?;
        self.writer.write_char(')')
    }

    pub fn print_constant(&mut self, c: &Constant) -> fmt::Result {
        if let Some(int) = c.as_integer() {
            write!(self.writer, "{int}")
        } else {
            let rational = &**c;
            let num = rational.numer();
            let denom = rational.denom();
            write!(self.writer, "frac({num}, {denom})")
        }
    }

    pub fn print_product(
        &mut self,
        exprs: &[BasicAlgebraicExpr],
        p: PrecedenceContext,
    ) -> fmt::Result {
        self.maybe_enter_parens(
            |this| {
                if exprs.iter().any(is_denominator) {
                    write!(this.writer, "frac(")?;

                    let mut denom = Printer::new_string();
                    let mut num_empty = true;
                    let mut num_prev_can_combine = false;
                    let mut denom_prev_can_combine = false;
                    for exp in exprs {
                        if is_denominator(exp) {
                            let BasicAlgebraicExpr::Pow(x) = exp else {
                                panic!("expected power")
                            };

                            if !denom.writer.is_empty() {
                                // TODO: if numeric power is not one, then we would print a power,
                                // which means it would always be able to combine with previous.
                                if denom_prev_can_combine && can_combine_with_prev(exp) {
                                    denom.writer.push(' ');
                                } else {
                                    denom.writer.push_str(" dot.op ");
                                }
                            }

                            denom_prev_can_combine = can_combine_with_next(&x.0);

                            denom.print_with_precedence(&x.0, PrecedenceContext::Product)?;

                            // TODO detect (x)^(-y)
                            if let BasicAlgebraicExpr::Numeric(x) = &x.1 {
                                let abs = x.abs();
                                if !abs.is_one() {
                                    denom.writer.push_str("^(");
                                    denom.print_constant(&abs)?;
                                    denom.writer.push(')');
                                    denom_prev_can_combine = true;
                                }
                            }
                        } else {
                            if !num_empty {
                                if num_prev_can_combine && can_combine_with_prev(exp) {
                                    this.writer.write_char(' ')?;
                                } else {
                                    this.writer.write_str(" dot.op ")?;
                                }

                                num_empty = false;
                            }
                            num_prev_can_combine = can_combine_with_next(exp);
                            this.print_with_precedence(exp, PrecedenceContext::Product)?;
                        }
                    }
                    let denom = denom.into_inner();
                    write!(this.writer, ",{denom})")?;
                } else {
                    let mut prev_can_combine = false;
                    for (n, item) in exprs.iter().enumerate() {
                        if n != 0 {
                            if prev_can_combine && can_combine_with_prev(item) {
                                this.writer.write_char(' ')?;
                            } else {
                                this.writer.write_str(" dot.op ")?;
                            }
                        }
                        prev_can_combine = can_combine_with_next(item);
                        this.print_with_precedence(item, PrecedenceContext::Product)?;
                    }
                }
                Ok(())
            },
            PrecedenceContext::Product < p,
        )
    }

    pub fn print_with_precedence(
        &mut self,
        x: &BasicAlgebraicExpr,
        p: PrecedenceContext,
    ) -> fmt::Result {
        let new_ctxt = x.precedence_ctxt();
        match x {
            BasicAlgebraicExpr::Factorial(x) => {
                self.print_with_precedence(x, new_ctxt)?;
                write!(self.writer, "!")?;
            }
            BasicAlgebraicExpr::Numeric(x) => {
                self.print_constant(x)?;
            }
            BasicAlgebraicExpr::Symbol(x) => {
                self.writer.write_str(x)?;
            }
            BasicAlgebraicExpr::Sum(items) => {
                self.maybe_enter_parens(
                    |this| {
                        for (n, item) in items.iter().enumerate() {
                            if n != 0 {
                                if let BasicAlgebraicExpr::Product(x) = item
                                && let [BasicAlgebraicExpr::Numeric(n), rest @ ..] = &**x
                                && n.is_negative()
                            {
                                write!(this.writer, "-")?;

                                if n.abs().is_one() {
                                    this.print_product(rest, new_ctxt)?;
                                    continue;
                                }
                            } else {
                                write!(this.writer, "+")?;
                            }
                            }
                            this.print_with_precedence(item, new_ctxt)?;
                        }
                        Ok(())
                    },
                    new_ctxt < p,
                )?;
            }
            BasicAlgebraicExpr::Product(items) => {
                self.print_product(items, p)?;
            }
            BasicAlgebraicExpr::Pow(x) => {
                let (base, exp) = &**x;
                let base_ctxt = base.precedence_ctxt();
                self.maybe_enter_parens(
                    |this| this.print_with_precedence(base, PrecedenceContext::NoPrecedence),
                    base_ctxt != PrecedenceContext::NoPrecedence
                        && base_ctxt < PrecedenceContext::Pow,
                )?;
                self.writer.write_char('^')?;
                self.enter_parens(|this| {
                    this.print_with_precedence(exp, PrecedenceContext::NoPrecedence)
                })?;
            }
            BasicAlgebraicExpr::Function(name, params) => {
                write!(self.writer, "\"{name}\"[")?;
                for (n, param) in params.iter().enumerate() {
                    if n != 0 {
                        self.writer.write_char(',')?;
                    }

                    self.print_with_precedence(param, PrecedenceContext::NoPrecedence)?;
                }
                self.writer.write_char(']')?;
            }
        }
        Ok(())
    }

    pub fn into_inner(self) -> W {
        self.writer
    }
}

impl Printer<String> {
    pub fn new_string() -> Self {
        Self {
            writer: String::new(),
        }
    }
}
