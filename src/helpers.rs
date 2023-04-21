use std::ops::{Add, BitXor, Div, Mul, Neg, Sub};

use num::BigRational;

use crate::rational_expressions::SimplifiedRationalExpression;
use crate::simplify::SimpleExpr;
use crate::{BasicAlgebraicExpr, ComputeResult, Undefined};

impl PartialEq<SimpleExpr> for i64 {
    fn eq(&self, other: &SimpleExpr) -> bool {
        other.eq(self)
    }
}

impl PartialEq<i64> for SimpleExpr {
    fn eq(&self, other: &i64) -> bool {
        match self.as_inner() {
            BasicAlgebraicExpr::Numeric(x) => &**x == &BigRational::from_integer((*other).into()),
            _ => false,
        }
    }
}

impl From<SimplifiedRationalExpression> for ComputeResult {
    fn from(e: SimplifiedRationalExpression) -> Self {
        match e {
            SimplifiedRationalExpression::Frac(f) => Ok(SimpleExpr::new_constant(f.into())),
            SimplifiedRationalExpression::Num(n) => Ok(SimpleExpr::new_constant(n.into())),
            SimplifiedRationalExpression::Undefined => Err(Undefined),
        }
    }
}

impl From<i128> for BasicAlgebraicExpr {
    fn from(x: i128) -> Self {
        BasicAlgebraicExpr::Numeric(x.into())
    }
}

impl From<i128> for SimpleExpr {
    fn from(x: i128) -> Self {
        SimpleExpr::new_constant(x.into())
    }
}

impl Add for BasicAlgebraicExpr {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        BasicAlgebraicExpr::Sum(vec![self, rhs])
    }
}

impl Mul for BasicAlgebraicExpr {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        BasicAlgebraicExpr::Product(vec![self, rhs])
    }
}

impl Sub for BasicAlgebraicExpr {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        BasicAlgebraicExpr::Sum(vec![self, -rhs])
    }
}

impl Neg for BasicAlgebraicExpr {
    type Output = Self;
    fn neg(self) -> Self::Output {
        BasicAlgebraicExpr::Product(vec![BasicAlgebraicExpr::from(-1), self])
    }
}

// `/` operator
impl Div for BasicAlgebraicExpr {
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        BasicAlgebraicExpr::Product(vec![
            self,
            BasicAlgebraicExpr::Pow(Box::new((rhs, (-1).into()))),
        ])
    }
}

// `^` operator
impl BitXor for BasicAlgebraicExpr {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self::Output {
        BasicAlgebraicExpr::Pow(Box::new((self, rhs)))
    }
}
