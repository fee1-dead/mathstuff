#![feature(let_chains, if_let_guard)]

use std::error::Error;

use constant::Constant;

use num::One;
use simplify::SimpleExpr;

mod cmp;
pub mod constant;
pub mod diff;
mod helpers;
pub mod parse;
pub mod polynomials;
pub mod print;
mod rational_expressions;
pub mod simplify;

#[derive(PartialEq, Eq, Debug, Clone, Copy, Hash)]
pub enum Constants {
    Pi,
    E,
}

#[derive(Debug)]
pub struct Undefined;

pub type ComputeResult<T = SimpleExpr> = Result<T, Undefined>;

/// Precedence (highest to lowest):
/// 1. Function/Factorial
/// 2. Exponentiation
/// 3. Product
/// 4. Sum
#[derive(PartialEq, Eq, Clone, Debug, Hash)]
pub enum BasicAlgebraicExpr {
    Numeric(Constant),
    // Const(Constants),
    Symbol(String),
    Product(Vec<BasicAlgebraicExpr>),
    Sum(Vec<BasicAlgebraicExpr>),
    Pow(Box<(BasicAlgebraicExpr, BasicAlgebraicExpr)>),
    Factorial(Box<BasicAlgebraicExpr>),
    Function(String, Vec<BasicAlgebraicExpr>),
}

impl BasicAlgebraicExpr {
    pub fn precedence_ctxt(&self) -> PrecedenceContext {
        use PrecedenceContext::*;
        match self {
            Self::Numeric(_) | Self::Symbol(_) => NoPrecedence,
            Self::Product(_) => Product,
            Self::Sum(_) => Sum,
            Self::Pow(_) => Pow,
            Self::Factorial(_) | Self::Function(_, _) => FunctionOrFactorial,
        }
    }

    pub fn assert_simple(self) -> SimpleExpr {
        SimpleExpr::assert(self)
    }
}

impl One for BasicAlgebraicExpr {
    fn one() -> Self {
        BasicAlgebraicExpr::Numeric(Constant::one())
    }
}

/// An enum representing operator precedence. Useful for printing stuff.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PrecedenceContext {
    /// Has no precedence. (wrapped in parens or function args)
    NoPrecedence,
    /// Sum context, currently equivalent to `NoPrecedence`
    Sum,
    /// Product context. Sums must be wrapped in parens
    Product,
    /// Exponentiation
    Pow,
    /// These operations are performed to their immediate left, so if their left
    /// is a compound expression we certainly want to wrap them in parenthesis.
    FunctionOrFactorial,
}

pub fn parse(x: &str) -> Result<BasicAlgebraicExpr, Box<dyn Error>> {
    let expr = parse::parse_into_expression(x).map_err(|_| "failed to parse expr")?;
    Ok(expr)
}

pub fn parse_and_simplify(x: &str) -> Result<ComputeResult, chumsky::error::Simple<parse::Token>> {
    let expr = parse::parse_into_expression(x)?;
    Ok(expr.simplify())
}

#[cfg(test)]
mod tests;
