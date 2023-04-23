use num::{BigInt, One, Signed, Zero};

use crate::rational_expressions::RationalExpr;
use crate::{BasicAlgebraicExpr, ComputeResult, Constant, Undefined};

use self::ops::{Operation, Product};

pub(crate) mod ops;

fn s(x: BasicAlgebraicExpr) -> SimpleExpr {
    SimpleExpr::new(x)
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Hash)]
#[repr(transparent)]
pub struct SimpleExpr {
    inner: BasicAlgebraicExpr,
}

impl SimpleExpr {
    #[inline]
    pub const fn new_constant(c: Constant) -> Self {
        Self {
            inner: BasicAlgebraicExpr::Numeric(c),
        }
    }

    #[inline]
    pub const fn new_symbol(s: String) -> Self {
        Self::new(BasicAlgebraicExpr::Symbol(s))
    }

    pub const fn assume_simplified(x: BasicAlgebraicExpr) -> Self {
        // TODO validate simplified status
        Self::new(x)
    }

    pub fn is_constant(&self) -> bool {
        self.inner.is_constant()
    }

    pub fn is_zero(&self) -> bool {
        self.inner.is_zero()
    }

    pub fn into_inner(self) -> BasicAlgebraicExpr {
        self.inner
    }

    pub const fn as_inner(&self) -> &BasicAlgebraicExpr {
        &self.inner
    }

    // If this is a product, split this into (constant, symbolic) parts
    // otherwise, retutn (1, x)
    // if this is a constant, return none
    pub fn split_product(self) -> Result<(RationalExpr, SimpleExpr), SimpleExpr> {
        match self.inner {
            BasicAlgebraicExpr::Product(mut x) => {
                if let Some(sym_index) = x.iter().position(|x| !x.is_constant()) {
                    let mut symbols = x.split_off(sym_index);

                    Ok((
                        RationalExpr::Mul(
                            x.into_iter()
                                .map(|x| match x {
                                    BasicAlgebraicExpr::Numeric(c) => c.into(),
                                    _ => unreachable!(),
                                })
                                .collect(),
                        ),
                        // TODO deref patterns
                        match symbols.len() {
                            1 => s(symbols.pop().unwrap()),
                            _ => s(BasicAlgebraicExpr::Product(symbols)),
                        },
                    ))
                } else {
                    unreachable!("product with only constant parts should be simplified already");
                }
            }
            BasicAlgebraicExpr::Numeric(_) => Err(self),
            _ => Ok((RationalExpr::Const(1.into()), self.clone())),
        }
    }

    pub fn base(&self) -> Option<&SimpleExpr> {
        Some(match &self.inner {
            BasicAlgebraicExpr::Pow(x) => Self::from_ref(&x.0),
            BasicAlgebraicExpr::Numeric(_) => return None,
            _ => self,
        })
    }

    pub fn exponent(&self) -> Option<SimpleExpr> {
        Some(match &self.inner {
            BasicAlgebraicExpr::Pow(x) => Self::new(x.1.clone()),
            BasicAlgebraicExpr::Numeric(_) => return None,
            _ => SimpleExpr::new_constant(1.into()),
        })
    }
}

impl From<Constant> for SimpleExpr {
    fn from(c: Constant) -> Self {
        Self::new_constant(c)
    }
}

impl From<i32> for SimpleExpr {
    fn from(x: i32) -> Self {
        SimpleExpr::new_constant(BigInt::from(x).into())
    }
}

impl SimpleExpr {
    #[cfg(feature = "evcxr")]
    pub fn evcxr_display(&self) {
        match katex::render(&crate::print::to_latex(&self)) {
            Ok(html) => {
                println!("EVCXR_BEGIN_CONTENT text/html\n{html}\nEVCXR_END_CONTENT");
            }
            Err(e) => {
                println!("EVCXR_BEGIN_CONTENT text/html\n<p>Failed to render via katex: {e:?}</p>\nEVCXR_END_CONTENT");
            }
        }
    }

    const fn new(inner: BasicAlgebraicExpr) -> Self {
        Self { inner }
    }

    fn from_ref(x: &BasicAlgebraicExpr) -> &Self {
        let ptr = <*const BasicAlgebraicExpr>::from(x).cast::<Self>();

        unsafe { ptr.as_ref().unwrap() }
    }
}

fn simplify_integer_power(base: SimpleExpr, exp: &BigInt) -> ComputeResult {
    match base.inner {
        _ if exp.is_zero() => Ok(1.into()),
        _ if exp.is_one() => Ok(base),
        BasicAlgebraicExpr::Numeric(base) => RationalExpr::Pow(Box::new(base.into()), exp.clone())
            .simplify()
            .into(),
        BasicAlgebraicExpr::Pow(x) => {
            let (base, exp2) = *x;
            let exp =
                Product.simplify(vec![SimpleExpr::new_constant(exp.clone().into()), s(exp2)])?;
            if let BasicAlgebraicExpr::Numeric(n) = &exp.inner && let Some(n) = n.as_integer() {
                simplify_integer_power(s(base), n)
            } else {
                Ok(s(BasicAlgebraicExpr::Pow(Box::new((base, exp.inner)))))
            }
        }
        BasicAlgebraicExpr::Product(exprs) => Ok(s(BasicAlgebraicExpr::Product(
            exprs
                .into_iter()
                .map(|x| simplify_integer_power(s(x), exp).map(|x| x.inner))
                .collect::<ComputeResult<Vec<_>>>()?,
        ))),
        _ => Ok(s(BasicAlgebraicExpr::Pow(Box::new((
            base.inner,
            BasicAlgebraicExpr::Numeric(exp.clone().into()),
        ))))),
    }
}

pub(crate) fn simplify_power(base: SimpleExpr, exponent: SimpleExpr) -> ComputeResult {
    if base == 0 {
        match exponent.inner {
            BasicAlgebraicExpr::Numeric(i) if i.is_positive() => Ok(0.into()),
            // 0^0 or 0^(-n) is undefined
            BasicAlgebraicExpr::Numeric(_) => Err(Undefined),
            exp => Ok(s(BasicAlgebraicExpr::Pow(Box::new((base.inner, exp))))),
        }
    } else if base == 1 {
        // 1^x = 1
        Ok(SimpleExpr::new_constant(One::one()))
    } else if let BasicAlgebraicExpr::Numeric(exp) = &exponent.inner && let Some(exp) = exp.as_integer() {
        simplify_integer_power(base, exp)
    } else {
        Ok(s(BasicAlgebraicExpr::Pow(Box::new((base.inner, exponent.inner)))))
    }
}

fn simplify_factorial(x: SimpleExpr) -> ComputeResult {
    match &x.inner {
        BasicAlgebraicExpr::Numeric(x) if let Some(x) = x.as_integer() && x <= &BigInt::from(10000) => {
            // TODO: how might we configure this limit?
            let mut current = BigInt::one();
            let mut product = BigInt::one();
            while &current <= x {
                product *= &current;
                current += 1;
            }

            Ok(SimpleExpr::new_constant(product.into()))
        }
        _ => Ok(SimpleExpr::new(BasicAlgebraicExpr::Factorial(Box::new(x.inner)))),
    }
}
