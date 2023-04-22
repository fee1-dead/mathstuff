use std::fmt::{self, Write};

use num::{Signed, One};

use crate::simplify::SimpleExpr;
use crate::{BasicAlgebraicExpr, PrecedenceContext};

pub fn to_latex(x: &SimpleExpr) -> String {
    let mut f = String::new();
    latex_print(x.as_inner(), &mut f);
    f
}

pub fn print_expr_to_string(x: &BasicAlgebraicExpr) -> String {
    let mut p = Printer::new_string();
    p.print(x).expect("String format does not have errors");
    p.into_inner()
}
pub struct Printer<W: Write> {
    writer: W,
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

    pub fn print_product(
        &mut self,
        exprs: &[BasicAlgebraicExpr],
        p: PrecedenceContext,
    ) -> fmt::Result {
        self.maybe_enter_parens(|this| {
            for (n, item) in exprs.iter().enumerate() {
                if n != 0 {
                    write!(this.writer, " dot.op ")?;
                }
                this.print_with_precedence(item, PrecedenceContext::Product)?;
            }
            Ok(())
        }, PrecedenceContext::Product < p)
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
            BasicAlgebraicExpr::Numeric(x) if let Some(int) = x.as_integer() => {
                write!(self.writer, "{int}")?;
            }
            BasicAlgebraicExpr::Numeric(x) => {
                let rational = &**x;
                let num = rational.numer();
                let denom = rational.denom();
                write!(self.writer, "frac({num}, {denom})")?;
            }
            BasicAlgebraicExpr::Symbol(x) => {
                self.writer.write_str(x)?;
            }
            BasicAlgebraicExpr::Sum(items) => {
                self.maybe_enter_parens(|this| {
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
                }, new_ctxt < p)?;
            }
            BasicAlgebraicExpr::Product(items) => {
                self.print_product(items, p)?;
            }
            BasicAlgebraicExpr::Pow(x) => {
                let (base, exp) = &**x;
                let base_ctxt = base.precedence_ctxt();
                self.maybe_enter_parens(|this| {
                    this.print_with_precedence(base, PrecedenceContext::NoPrecedence)
                }, base_ctxt != PrecedenceContext::NoPrecedence && base_ctxt < PrecedenceContext::Pow)?;
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

pub fn latex_print(x: &BasicAlgebraicExpr, f: &mut String) {
    match x {
        BasicAlgebraicExpr::Numeric(x) if let Some(i) = x.as_integer() =>  {
            f.push_str(&i.to_string());
        }
        BasicAlgebraicExpr::Numeric(x) => {
            let rational = &**x;
            let num = rational.numer();
            let denom = rational.denom();
            f.push_str(&format!("\\frac {{ {num} }} {{ {denom} }}"));
        }
        BasicAlgebraicExpr::Symbol(x) => {
            f.push_str(&x);
        }
        BasicAlgebraicExpr::Product(x) => {
            if x.len() == 0 {
                unreachable!()
            } else {
                for (i, x) in x.iter().enumerate() {
                    if i != 0 {
                        f.push_str(" \\cdot ");
                    }
                    latex_print(x, f);
                }
            }
        }
        BasicAlgebraicExpr::Sum(x) => {
            for (i, x) in x.iter().enumerate() {
                if i != 0 {
                    f.push_str(" + ");
                }
                latex_print(x, f);
            }
        }
        BasicAlgebraicExpr::Pow(x) => {
            f.push_str("(");
            latex_print(&x.0, f);
            f.push_str(")^{");
            latex_print(&x.1, f);
            f.push_str("}");
        }
        BasicAlgebraicExpr::Factorial(x) => {
            latex_print(x, f);
            f.push_str("!");
        }
        BasicAlgebraicExpr::Function(x, y) => {
            f.push_str(&x);
            f.push_str("(");
            for (i, x) in y.iter().enumerate() {
                if i != 0 {
                    f.push_str(", ");
                }
                latex_print(x, f);
            }
            f.push_str(")");
        }
    }
}
