#![feature(let_chains, if_let_guard)]

use constant::Constant;

use num::BigInt;
use simplify::SimpleExpr;

mod cmp;
pub mod constant;
pub mod diff;
mod helpers;
pub mod parse;
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

pub fn parse_and_simplify(x: &str) -> Result<ComputeResult, chumsky::error::Simple<parse::Token>> {
    let expr = parse::parse_into_expression(x)?;
    Ok(expr.simplify())
}

#[cfg(test)]
mod tests;
