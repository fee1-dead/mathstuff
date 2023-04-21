use std::fmt::Debug;

use crate::constant::Constant;
use crate::rational_expressions::RationalExpr;
use crate::{BasicAlgebraicExpr, ComputeResult, SimpleExpr, Undefined};
use num::{BigInt, One, Signed, Zero};
use smallvec::{smallvec, SmallVec};
use tracing::debug;

pub trait Operation: Copy + Debug {
    // https://en.wikipedia.org/wiki/Absorbing_element
    // we are talking about zero
    const HAS_ABSORBING_ELEMENT: bool;

    fn is_absorbing_element(self, x: &SimpleExpr) -> bool;

    // multiplicative identity / additive identity
    fn identity(self) -> SimpleExpr;

    fn is_identity(self, x: &Constant) -> bool;

    fn is_list(self, x: &SimpleExpr) -> bool;

    fn try_extract_list(self, x: SimpleExpr) -> Result<Vec<SimpleExpr>, SimpleExpr>;

    fn make_list(self, x: Vec<SimpleExpr>) -> SimpleExpr;

    fn extract_or_make_list(self, x: SimpleExpr) -> Vec<SimpleExpr> {
        self.try_extract_list(x).unwrap_or_else(|x| vec![x])
    }

    fn do_constant(self, x: Constant, y: Constant) -> Constant;

    /// The backbone of simplify_pair. If we can simplify by collecting like terms in addition or powers in multiplication, we do so.
    /// if not, we return `Ok(None)`.
    fn simplify_pair_collect(
        self,
        a: SimpleExpr,
        b: SimpleExpr,
    ) -> ComputeResult<Option<SmallVec<[SimpleExpr; 2]>>>;

    fn simplify_pair(
        self,
        a: SimpleExpr,
        b: SimpleExpr,
    ) -> ComputeResult<SmallVec<[SimpleExpr; 2]>> {
        if self.is_list(&a) || self.is_list(&b) {
            let a = self.extract_or_make_list(a);
            let b = self.extract_or_make_list(b);
            return self.merge(a, b).map(Into::into);
        }

        Ok(match (a, b) {
            (
                SimpleExpr {
                    inner: BasicAlgebraicExpr::Numeric(a),
                },
                SimpleExpr {
                    inner: BasicAlgebraicExpr::Numeric(b),
                },
            ) => {
                let result = self.do_constant(a, b);
                if self.is_identity(&result) {
                    SmallVec::new()
                } else {
                    smallvec![result.into()]
                }
            }
            (
                SimpleExpr {
                    inner: BasicAlgebraicExpr::Numeric(a),
                },
                b,
            )
            | (
                b,
                SimpleExpr {
                    inner: BasicAlgebraicExpr::Numeric(a),
                },
            ) if self.is_identity(&a) => {
                smallvec![b]
            }
            (a, b) => {
                // NOTE: when in addition, we merge x + x = 2x, 3x + 4x = 7x, etc.
                // but when in multiplication, we merge x * x = x^2, x^3 * x^4 = x^7, etc.

                if let Some(res) = self.simplify_pair_collect(a.clone(), b.clone())? {
                    res
                } else if b < a {
                    smallvec![b, a]
                } else {
                    smallvec![a, b]
                }
            }
        })
    }

    // requirement: `exprs.len() >= 2`
    #[tracing::instrument(level = "debug", ret)]
    fn simplify_rec(self, list: Vec<SimpleExpr>) -> ComputeResult<Vec<SimpleExpr>> {
        let res: Result<[SimpleExpr; 2], _> = list.try_into();
        match res {
            Ok([a, b]) => self.simplify_pair(a, b).map(|x| x.into_vec()),
            Err(mut v) => {
                assert!(v.len() > 2);
                let first = v.remove(0);

                let first = self.extract_or_make_list(first);

                self.merge(first, v)
            }
        }
    }

    #[tracing::instrument(level = "debug")]
    fn simplify_entry(self, exprs: Vec<BasicAlgebraicExpr>) -> ComputeResult {
        let mut exprs: Vec<_> = exprs
            .into_iter()
            .map(BasicAlgebraicExpr::simplify)
            .collect::<Result<_, _>>()?;
        exprs.sort_unstable();
        self.simplify(exprs)
    }

    #[tracing::instrument(level = "debug", ret)]
    fn simplify(self, mut exprs: Vec<SimpleExpr>) -> ComputeResult {
        if Self::HAS_ABSORBING_ELEMENT {
            for exp in &exprs {
                if self.is_absorbing_element(exp) {
                    return Ok(0.into());
                }
            }
        }

        if exprs.is_empty() {
            return Ok(self.identity());
        }

        if exprs.len() == 1 {
            return Ok(exprs.pop().expect("len >= 1"));
        }

        let mut list = self.simplify_rec(exprs)?;
        // TODO replace with deref patterns
        Ok(match list.len() {
            0 => self.identity(),
            1 => list.pop().expect("len == 1"),
            _ => self.make_list(list),
        })
    }

    // entry point. Do not call in recursion. Call `merge_into` instead.
    fn merge(self, a: Vec<SimpleExpr>, b: Vec<SimpleExpr>) -> ComputeResult<Vec<SimpleExpr>> {
        let mut out = Vec::with_capacity(a.len() + b.len());
        self.merge_into(a, b, &mut out)?;
        Ok(out)
    }

    #[tracing::instrument(level = "debug", ret)]
    fn merge_into(
        self,
        mut a: Vec<SimpleExpr>,
        mut b: Vec<SimpleExpr>,
        out: &mut Vec<SimpleExpr>,
    ) -> ComputeResult<()> {
        if b.is_empty() {
            out.extend(a);
            return Ok(());
        }

        if a.is_empty() {
            out.extend(b);
            return Ok(());
        }

        let mut a_rest = a.split_off(1);
        let mut b_rest = b.split_off(1);
        let a = a.pop().unwrap();
        let b = b.pop().unwrap();

        let would_swap = a > b;

        let simplified = self.simplify_pair(a, b)?;

        match simplified.len() {
            0 => self.merge_into(a_rest, b_rest, out)?,
            1 => {
                out.extend(simplified);
                self.merge_into(a_rest, b_rest, out)?;
            }
            2 => {
                let [first, second]: [_; 2] = simplified.into_inner().unwrap();

                if would_swap {
                    a_rest.insert(0, second);
                } else {
                    b_rest.insert(0, second);
                };

                out.push(first);
                self.merge_into(a_rest, b_rest, out)?;
            }
            _ => unreachable!("nested operations should have been flattened already"),
        }

        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Product;

impl Operation for Product {
    const HAS_ABSORBING_ELEMENT: bool = true;

    fn is_absorbing_element(self, expr: &SimpleExpr) -> bool {
        expr.is_zero()
    }

    fn is_identity(self, expr: &Constant) -> bool {
        expr.is_one()
    }

    fn identity(self) -> SimpleExpr {
        1.into()
    }

    fn is_list(self, x: &SimpleExpr) -> bool {
        matches!(x.inner, BasicAlgebraicExpr::Product(_))
    }

    // TODO make these collects less painful (maybe just all use BAEs instead)
    fn try_extract_list(self, x: SimpleExpr) -> Result<Vec<SimpleExpr>, SimpleExpr> {
        match x.inner {
            BasicAlgebraicExpr::Product(x) => Ok(x.into_iter().map(SimpleExpr::new).collect()),
            _ => Err(x),
        }
    }

    fn make_list(self, x: Vec<SimpleExpr>) -> SimpleExpr {
        SimpleExpr::new(BasicAlgebraicExpr::Product(
            x.into_iter().map(|x| x.inner).collect(),
        ))
    }

    fn do_constant(self, x: Constant, y: Constant) -> Constant {
        x * y
    }

    fn simplify_pair_collect(
        self,
        a: SimpleExpr,
        b: SimpleExpr,
    ) -> ComputeResult<Option<SmallVec<[SimpleExpr; 2]>>> {
        Ok(
            if let Some(base) = a.base().filter(|x| Some(*x) == b.base()) {
                let exponent = Sum.simplify(vec![
                    a.exponent().expect("base() is not None"),
                    b.exponent().expect("base() is not None"),
                ])?;
                let result = super::simplify_power(base.clone(), exponent)?;
                Some(if let BasicAlgebraicExpr::Numeric(c) = &result.inner && c.is_one() {
                smallvec![]
            } else {
                smallvec![result]
            })
            } else {
                None
            },
        )
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Sum;

impl Operation for Sum {
    const HAS_ABSORBING_ELEMENT: bool = false;
    fn is_absorbing_element(self, _: &SimpleExpr) -> bool {
        false
    }
    fn identity(self) -> SimpleExpr {
        0.into()
    }
    fn is_identity(self, expr: &Constant) -> bool {
        expr.is_zero()
    }
    fn is_list(self, x: &SimpleExpr) -> bool {
        matches!(x.inner, BasicAlgebraicExpr::Sum(_))
    }
    fn try_extract_list(self, x: SimpleExpr) -> Result<Vec<SimpleExpr>, SimpleExpr> {
        match x.inner {
            // TODO maybe use bytemuck
            BasicAlgebraicExpr::Sum(x) => Ok(x.into_iter().map(SimpleExpr::new).collect()),
            _ => Err(x),
        }
    }
    fn make_list(self, x: Vec<SimpleExpr>) -> SimpleExpr {
        SimpleExpr::new(BasicAlgebraicExpr::Sum(
            x.into_iter().map(|x| x.inner).collect(),
        ))
    }

    #[tracing::instrument(level = "debug")]
    fn do_constant(self, x: Constant, y: Constant) -> Constant {
        x + y
    }

    #[tracing::instrument(level = "debug")]
    // TODO should return smallvec?
    fn simplify_pair_collect(
        self,
        a: SimpleExpr,
        b: SimpleExpr,
    ) -> ComputeResult<Option<SmallVec<[SimpleExpr; 2]>>> {
        let Ok((rationala, a_sym)) = a.split_product() else { return Ok(None) };
        let Ok((rationalb, b_sym)) = b.split_product() else { return Ok(None) };

        debug!(?rationala, ?rationalb, ?a_sym, ?b_sym);

        Ok(if a_sym == b_sym {
            let sum = (rationala + rationalb).simplify().into_algebraic_expr()?;
            debug!(?sum, ?a_sym);
            Some(smallvec![Product.simplify(vec![sum, a_sym])?])
        } else {
            None
        })
    }
}

impl BasicAlgebraicExpr {
    #[inline]
    pub const fn is_constant(&self) -> bool {
        matches!(self, BasicAlgebraicExpr::Numeric(_))
    }

    #[inline]
    pub fn is_zero(&self) -> bool {
        matches!(self, BasicAlgebraicExpr::Numeric(x) if x.is_zero())
    }

    pub fn simplify(self) -> ComputeResult {
        use BasicAlgebraicExpr::*;
        use SimpleExpr as E;
        Ok(match self {
            Numeric(c) if c.denom().is_zero() => return Err(Undefined),
            Numeric(c) => E::new_constant(c),
            Symbol(s) => E::new_symbol(s),
            Pow(x) => super::simplify_power((*x).0.simplify()?, (*x).1.simplify()?)?,
            Sum(x) => self::Sum.simplify_entry(x)?,
            Product(x) => self::Product.simplify_entry(x)?,
            Factorial(_) => todo!(),
            Function(..) => todo!(),
        })
    }
}
