use fxhash::FxHashSet;
use num::One;

use crate::simplify::SimpleExpr;
use crate::{BasicAlgebraicExpr, Undefined};

/// Given an expression, identify potential candidates for variables.
///
/// # Examples
///
/// ```
/// # use mathstuff::polynomials::variables;
/// use mathstuff::{BasicAlgebraicExpr, parse};
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let val = parse("x^3 + 3 * x^2 * y + 3 * x * y^2 + y^3")?;
///
/// assert_eq!(variables(val), [
///     parse("x")?,
///     parse("y")?,
/// ].into_iter().collect());
///
/// let x = parse("3 * x * (x + 1) * y^2 * z^n")?;
///
/// assert_eq!(variables(x), [
///     parse("x")?,
///     parse("x + 1")?,
///     parse("y")?,
///     parse("z^n")?
/// ].into_iter().collect());
/// 
/// let x = parse("a * Sin[x]^2 + 2 * b * Sin[x] + 3 * c")?;
/// 
/// assert_eq!(variables(x), [
///     parse("a")?,
///     parse("b")?,
///     parse("c")?,
///     parse("Sin[x]")?
/// ].into_iter().collect());
/// 
/// let x = parse("1 / 2")?.simplify().unwrap().into_inner();
/// assert!(variables(x).is_empty());
///
/// # Ok(())
/// # }
/// ```
// TODO it doesn't feel very efficient to create single value hashsets
pub fn variables(x: BasicAlgebraicExpr) -> FxHashSet<BasicAlgebraicExpr> {
    match x {
        BasicAlgebraicExpr::Numeric(_) => FxHashSet::default(),
        BasicAlgebraicExpr::Pow(x) => {
            // If the power has an exponent that is an integer greater than one,
            // we return the exponent base.
            let var = if let BasicAlgebraicExpr::Numeric(c) = &x.1
            && let Some(int) = c.as_integer()
            && int > &One::one()
            {
                (*x).0
            } else {
                BasicAlgebraicExpr::Pow(x)
            };
            [var].into_iter().collect()
        }
        BasicAlgebraicExpr::Sum(values) => values.into_iter().flat_map(variables).collect(),
        BasicAlgebraicExpr::Product(values) => values
            .into_iter()
            .flat_map(|x| {
                if let BasicAlgebraicExpr::Sum(_) = x {
                    [x].into_iter().collect()
                } else {
                    variables(x)
                }
            })
            .collect(),
        BasicAlgebraicExpr::Factorial(_)
        | BasicAlgebraicExpr::Symbol(_)
        | BasicAlgebraicExpr::Function(..) => [x].into_iter().collect(),
    }
}

pub trait GeneralizedVars {
    fn contains(&self, x: &BasicAlgebraicExpr) -> bool;
}

impl GeneralizedVars for [BasicAlgebraicExpr] {
    fn contains(&self, x: &BasicAlgebraicExpr) -> bool {
        <[_]>::contains(self, x)
    }
}

impl GeneralizedVars for BasicAlgebraicExpr {
    fn contains(&self, x: &BasicAlgebraicExpr) -> bool {
        self == x
    }
}

impl<T: GeneralizedVars> GeneralizedVars for &T {
    fn contains(&self, x: &BasicAlgebraicExpr) -> bool {
        (**self).contains(x)
    }
}

/// Returns whether `u` is free of `t` i.e. that `u` does not contain `t`.
pub fn free_of(u: &BasicAlgebraicExpr, t: impl GeneralizedVars) -> bool {
    if t.contains(u) {
        return true;
    }

    match u {
        BasicAlgebraicExpr::Numeric(_) | BasicAlgebraicExpr::Symbol(_) => true,
        BasicAlgebraicExpr::Pow(x) => {
            let (base, exp) = &**x;
            free_of(base, &t) && free_of(exp, &t)
        }
        BasicAlgebraicExpr::Product(args)
        | BasicAlgebraicExpr::Sum(args)
        | BasicAlgebraicExpr::Function(_, args) => args.iter().all(|x| free_of(x, &t)),
        BasicAlgebraicExpr::Factorial(x) => free_of(x, t),
    }
}

/// Let `s` be a set of generalized variables. Extract the coefficient and variable parts of `u`.
/// Returns undefined is `u` is not a general monomial expression in `s`.
pub fn coeff_var_monomial(
    u: SimpleExpr,
    vars: impl GeneralizedVars,
) -> Result<(BasicAlgebraicExpr, BasicAlgebraicExpr), Undefined> {
    if free_of(&u, &vars) {
        return Ok((u.into_inner(), One::one()));
    }

    if vars.contains(&u) {
        return Ok((One::one(), u.into_inner()));
    }

    match u.into_inner() {
        BasicAlgebraicExpr::Numeric(_) | BasicAlgebraicExpr::Symbol(_) => {
            panic!("should have been caught by `free_of` above");
        }
        BasicAlgebraicExpr::Pow(x) => {
            if vars.contains(&x.0) {
                Ok((One::one(), BasicAlgebraicExpr::Pow(x)))
            } else {
                Err(Undefined)
            }
        }
        BasicAlgebraicExpr::Product(_values) => {
            // let mut ret = Vec::new();
            todo!()
        }
        BasicAlgebraicExpr::Sum(values) => {
            if let Ok([val]) = TryInto::<[_; 1]>::try_into(values) {
                coeff_var_monomial(val.assert_simple(), vars)
            } else {
                Err(Undefined)
            }
        }
        BasicAlgebraicExpr::Factorial(_) | BasicAlgebraicExpr::Function(..) => Err(Undefined),
    }
}

pub fn collect_terms(
    simple: SimpleExpr,
    vars: impl GeneralizedVars,
) -> Result<BasicAlgebraicExpr, Undefined> {
    let u = simple.into_inner();

    if vars.contains(&u) {
        return Ok(u);
    }

    let BasicAlgebraicExpr::Sum(_terms) = u else {
        coeff_var_monomial(u.clone().assert_simple(), vars)?;

        return Ok(u);
    };

    todo!()
}
