//! Auto-differentiation.

use std::collections::HashMap;
use std::iter::once;

use tracing::debug;

use crate::constant::Constant;
use crate::simplify::ops::{self, Operation};
use crate::simplify::{simplify_power, SimpleExpr};
use crate::{BasicAlgebraicExpr, ComputeResult};

pub trait DifferentiableFunction {
    fn diff(&self, params: Vec<SimpleExpr>) -> SimpleExpr;
}

#[derive(Default)]
pub struct Differentiator {
    pub functions: HashMap<String, Box<dyn DifferentiableFunction>>,
}

pub fn references(x: &BasicAlgebraicExpr, var: &str) -> bool {
    match x {
        BasicAlgebraicExpr::Product(v)
        | BasicAlgebraicExpr::Sum(v)
        | BasicAlgebraicExpr::Function(_, v) => v.iter().any(|x| references(x, var)),
        BasicAlgebraicExpr::Factorial(x) => references(x, var),
        BasicAlgebraicExpr::Pow(b) => references(&b.0, var) || references(&b.1, var),
        BasicAlgebraicExpr::Symbol(s) => s == var,
        BasicAlgebraicExpr::Numeric(_) => false,
    }
}

#[derive(Debug, Clone)]
pub enum DifferentiationError {
    PowerReferencesVar,
    FactorialReferencesVar,
    UnrecognizedFunction,
    Undefined,
}

impl From<crate::Undefined> for DifferentiationError {
    fn from(_: crate::Undefined) -> Self {
        DifferentiationError::Undefined
    }
}

impl Differentiator {
    pub fn new() -> Self {
        Self {
            functions: HashMap::new(),
        }
    }
    #[tracing::instrument(skip(self), level = "info")]
    pub fn differentiate(
        &self,
        x: SimpleExpr,
        wrt: &str,
    ) -> Result<SimpleExpr, DifferentiationError> {
        use BasicAlgebraicExpr::*;
        Ok(match x.into_inner() {
            Numeric(_) => SimpleExpr::new_constant(0.into()),
            Symbol(s) if s == wrt => SimpleExpr::new_constant(1.into()),
            Symbol(s) => SimpleExpr::new_symbol(s),
            Product(x) => {
                // split the product into two parts, factors that references x, and factors that do not.
                let (refs, mut norefs): (Vec<_>, Vec<_>) = x
                    .into_iter()
                    .map(SimpleExpr::assume_simplified)
                    .into_iter()
                    .partition(|x| references(x.as_inner(), wrt));

                // for products that reference x, we use the product rule.
                let sum: Vec<_> = refs
                    .clone()
                    .into_iter()
                    .enumerate()
                    .map(|(i, exp)| {
                        let others = refs
                            .iter()
                            .cloned()
                            .take(i)
                            .chain(refs.iter().cloned().skip(i + 1));
                        self.differentiate(exp, wrt).and_then(|x| {
                            ops::Product
                                .simplify(others.chain(once(x)).collect())
                                .map_err(Into::into)
                        })
                    })
                    .collect::<Result<_, _>>()?;

                let dx = ops::Sum.simplify(sum)?;

                if !dx.is_zero() {
                    norefs.push(dx);
                }

                ops::Product.simplify(norefs)?
            }
            // TODO this is a bit weird. We should transform `a^x` into `e^(ln(a) * x)` and then
            // differentiate that.
            Pow(x) => {
                let (base, exp) = *x;

                if references(&exp, wrt) {
                    return Err(DifferentiationError::PowerReferencesVar);
                }

                let [base, exp] = [base, exp].map(SimpleExpr::assume_simplified);

                debug!(?base, ?exp);

                return Ok(ops::Product.simplify(vec![
                    self.differentiate(base.clone(), wrt)?,
                    exp.clone(),
                    simplify_power(
                        base,
                        ops::Sum.simplify(vec![
                            exp,
                            SimpleExpr::new_constant(Constant::negative_one()),
                        ])?,
                    )?,
                ])?);
            }
            Sum(x) => ops::Sum.simplify(
                x.into_iter()
                    .map(SimpleExpr::assume_simplified)
                    .map(|x| self.differentiate(x, wrt))
                    .collect::<Result<_, DifferentiationError>>()?,
            )?,
            Factorial(x) => {
                if references(&x, wrt) {
                    return Err(DifferentiationError::FactorialReferencesVar);
                } else {
                    SimpleExpr::assume_simplified(Factorial(x))
                }
            }
            Function(x, args) => {
                if let Some(f) = self.functions.get(&x) {
                    let r: Result<[_; 1], _> = args.try_into();
                    if let Ok([arg]) = r {
                        let arg = SimpleExpr::assume_simplified(arg);
                        let a2 = arg.clone();
                        return Ok(ops::Product
                            .simplify(vec![self.differentiate(arg, wrt)?, f.diff(vec![a2])])?);
                    }
                }
                return Err(DifferentiationError::UnrecognizedFunction);
            }
        })
    }
}
