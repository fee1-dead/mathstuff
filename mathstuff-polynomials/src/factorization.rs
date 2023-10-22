use std::fmt::Display;
use std::num::NonZeroUsize;
use std::ops::RangeInclusive;

use mathstuff_types::print::{PrintWithVar, PrintableCoeff, DisplayWithVar, print_as_factor};
use num::integer::Roots;
use num::traits::Inv;
use num::{BigInt, BigRational, One, Zero, Integer};

use mathstuff_types::traits::{CommutativeRing, Field, FromUsize};
use mathstuff_types::Polynomial;

/*
/// Given a polynomial represented as a product of (x - a_i) given in `nums`, return
/// the full expanded polynomial.
pub fn binomial_product<Ring: CommutativeRing>(nums: Vec<Ring>) -> Polynomial<Ring> {

}
*/

// TODO make this an iterator interface
/// Find all positive divisors of an integer
///
/// # Examples
/// 
/// ```
/// # use math2::factorization::integer_divisors;
/// 
/// let mut divisors = integer_divisors(12);
/// divisors.sort_unstable();
/// assert_eq!(vec![1, 2, 3, 4, 6, 12], divisors);
/// 
/// let mut divisors = integer_divisors(36);
/// divisors.sort_unstable();
/// assert_eq!(vec![1, 2, 3, 4, 6, 9, 12, 18, 36], divisors);
/// ```
pub fn integer_divisors<N: Roots + Clone>(x: N) -> Vec<N>
where
    RangeInclusive<N>: IntoIterator<Item = N>,
{
    let n = x.sqrt();
    let mut res = vec![ N::one(), x.clone() ];
    for div in (N::one() + N::one()) ..= n {
        if x.is_multiple_of(&div) {
            let other = x.div_floor(&div);
            if other != div {
                res.push(other);
            }
            res.push(div);
        }
    }
    res
}

// TODO generic
/// Returns the polynomial that interpolates the given points. This takes O(n^2) time.
///
/// # Example
///
/// ```
/// # use math2::factorization::lagrange_interpolation;
/// # use math2::Polynomial;
/// # use num::BigRational;
/// let n = |x: i32| BigRational::from_integer(x.into());
/// let points = vec![(n(1), n(1)), (n(2), n(-1))];
/// let p = lagrange_interpolation(points);
/// assert_eq!(p, Polynomial::new(vec![n(3), n(-2)]));
/// ```
pub fn lagrange_interpolation(points: Vec<(BigRational, BigRational)>) -> Polynomial<BigRational> {
    let points2 = points.clone();
    points2
        .into_iter()
        .map(|(x, y)| {
            let p: Polynomial<BigRational> = points
                .iter()
                .filter(|(x2, _)| &x != x2)
                .map(|(x, _)| Polynomial::new(vec![-x, BigRational::one()]))
                .product();
            p * y
                * points
                    .iter()
                    .filter(|(x2, _)| &x != x2)
                    .map(|(x2, _)| &x - x2)
                    .product::<BigRational>()
                    .inv()
        })
        .sum()
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SquareFreeFactorization<F> {
    pub leading_coeff: F,
    pub factors: Vec<(Polynomial<F>, NonZeroUsize)>,
}

impl<F: Field> SquareFreeFactorization<F> {
    /// Returns the square free factorization of this polynomial, using Yun's algorithm.
    pub fn factor_polynomial(x: Polynomial<F>) -> Self where F: FromUsize + PartialEq {
        if x.is_zero() {
            return SquareFreeFactorization {
                leading_coeff: F::zero(),
                factors: Vec::new(),
            };
        }

        let leading_coeff = x.leading_coefficient_cloned();
        let u = x.scalar_mul(leading_coeff.clone().checked_inv().unwrap());
        let mut factors = Vec::new();
        let mut r = u.clone().gcd(u.clone().derivative());
        let mut f = u.div_rem(r.clone()).0;
        let mut j = NonZeroUsize::new(1).unwrap();
        while !r.is_one() {
            let g = r.clone().gcd(f.clone());
            let s = f.div_rem(g.clone()).0;
            if !s.is_one() {
                factors.push((s, j));
            }
            r = r.div_rem(g.clone()).0;
            f = g;
            j = j.saturating_add(1);
        }
        if !f.is_one() {
            factors.push((f, j));
        }
        SquareFreeFactorization {
            leading_coeff,
            factors,
        }
    }
}

impl<T: PrintableCoeff> DisplayWithVar for SquareFreeFactorization<T> {
    fn fmt_with_var<'a>(this: &'a PrintWithVar<'a, Self>, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        print_as_factor(&this.thing().leading_coeff, f)?;
        for (poly, exp) in &this.thing().factors {
            write!(f, "({})", poly.print_with_var(this.var()))?;
            if exp.get() > 1 {
                write!(f, "^{}", exp)?;
            }
        }
        Ok(())
    }
}

pub struct Kronecker<Ring: CommutativeRing> {
    factors: Vec<Polynomial<Ring>>,
}

impl Kronecker<BigInt> {
    pub fn new(polynomial: Polynomial<BigInt>) -> Self {
        if polynomial.degree().map_or(true, |x| x <= 1) {
            return Kronecker {
                factors: vec![polynomial],
            };
        }
        todo!()
    }
}
