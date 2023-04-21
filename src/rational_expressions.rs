use std::ops::Add;

use num::{BigInt, BigRational, One, Zero};

use crate::simplify::SimpleExpr;
use crate::{ComputeResult, Constant, Undefined};

#[derive(Debug)]
pub enum RationalExpr {
    Add(Vec<RationalExpr>),
    Mul(Vec<RationalExpr>),
    Div(Vec<RationalExpr>),
    Sub(Vec<RationalExpr>),
    Const(Constant),
    Pow(Box<RationalExpr>, BigInt),
}

impl From<Constant> for RationalExpr {
    fn from(r: Constant) -> Self {
        RationalExpr::Const(r)
    }
}

pub enum SimplifiedRationalExpression {
    Frac(BigRational),
    Num(BigInt),
    Undefined,
}

impl SimplifiedRationalExpression {
    pub fn into_algebraic_expr(self) -> ComputeResult {
        match self {
            Self::Frac(x) => Ok(SimpleExpr::new_constant(x.into())),
            Self::Num(x) => Ok(SimpleExpr::new_constant(x.into())),
            Self::Undefined => Err(Undefined),
        }
    }
}

impl From<Constant> for SimplifiedRationalExpression {
    fn from(x: Constant) -> Self {
        let denom = x.denom();
        if denom == &BigInt::from(0) {
            Self::Undefined
        } else if x.denom() == &BigInt::from(1) {
            Self::Num(x.to_integer())
        } else {
            Self::Frac(x.into_inner())
        }
    }
}

impl RationalExpr {
    fn simplify_to_const(self) -> Constant {
        use RationalExpr::*;
        match self {
            Const(x) => x,
            Add(x) => x.into_iter().map(|x| x.simplify_to_const()).sum(),
            Mul(x) => x.into_iter().map(|x| x.simplify_to_const()).product(),
            Sub(x) => x
                .into_iter()
                .map(|x| x.simplify_to_const())
                .fold(Zero::zero(), |acc, x| acc - x),
            Div(x) => x
                .into_iter()
                .map(|x| x.simplify_to_const())
                .fold(One::one(), |acc, x| acc / x),
            Pow(base, exp) => {
                let base = base.simplify_to_const();
                num::pow::Pow::pow(base, &exp)
            }
        }
    }

    pub fn simplify(self) -> SimplifiedRationalExpression {
        self.simplify_to_const().into()
    }
}

impl Add for RationalExpr {
    type Output = RationalExpr;

    fn add(self, other: RationalExpr) -> RationalExpr {
        use RationalExpr::*;
        match (self, other) {
            (Const(a), Const(b)) => Const(a + b),
            (Add(mut a), Add(b)) => {
                a.extend(b);
                Add(a)
            }
            (Add(mut a), b) | (b, Add(mut a)) => {
                a.push(b);
                Add(a)
            }
            (a, b) => Add(vec![a, b]),
        }
    }
}
