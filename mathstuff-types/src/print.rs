use std::fmt::{self, Display};

use num::Signed;

use crate::traits::CommutativeRing;
use crate::Polynomial;

pub trait PrintableCoeff: Display + CommutativeRing + PartialEq + Signed {}

impl<X: Display + CommutativeRing + PartialEq + Signed> PrintableCoeff for X {}

fn print_if_not_one(x: &impl PrintableCoeff, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    if !x.is_one() {
        write!(f, "{x}")?;
    }
    Ok(())
}

pub fn print_as_factor(x: &impl PrintableCoeff, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    if x.is_negative() {
        f.write_str("-")?;
    }
    print_if_not_one(&x.abs(), f)
}

impl<T: PrintableCoeff> Polynomial<T> {
    pub fn print_with_var<'a>(&'a self, var: &'a str) -> PrintWithVar<'a, Polynomial<T>> {
        PrintWithVar::new(var, self)
    }
}

pub struct PrintWithVar<'a, F> {
    var: &'a str,
    thing: &'a F,
}

impl<'a, F> PrintWithVar<'a, F> {
    pub fn new(var: &'a str, thing: &'a F) -> Self {
        Self { var, thing }
    }

    pub fn var(&self) -> &'a str {
        self.var
    }

    pub fn thing(&self) -> &'a F {
        self.thing
    }
}

pub trait DisplayWithVar: Sized {
    fn print_with_var<'a>(&'a self, var: &'a str) -> PrintWithVar<'a, Self>
    where
        Self: Sized,
    {
        PrintWithVar::new(var, self)
    }

    fn fmt_with_var<'a>(this: &'a PrintWithVar<'a, Self>, f: &mut fmt::Formatter<'_>) -> fmt::Result;
}

impl<T: PrintableCoeff> DisplayWithVar for Polynomial<T> {
    fn fmt_with_var<'a>(this: &'a PrintWithVar<'a, Self>, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut first = true;
        let PrintWithVar { var, thing } = this;
        for (degree, coeff) in thing.coeffs.iter().enumerate().rev() {
            if coeff.is_zero() {
                continue;
            }

            if !first {
                f.write_str(if coeff.is_negative() { " - " } else { " + " })?;
            }

            let abs;
            let coeff = if first {
                coeff
            } else {
                abs = coeff.abs();
                &abs
            };

            first = false;

            if degree == 0 {
                write!(f, "{coeff}")?;
            } else {
                print_if_not_one(coeff, f)?;
                if degree == 1 {
                    write!(f, "{var}")?;
                } else {
                    write!(f, "{var}^{degree}")?;
                }
            }
        }

        Ok(())
    }
}

impl<T: DisplayWithVar> Display for PrintWithVar<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        T::fmt_with_var(self, f)
    }
}


