use std::fmt::Debug;
use std::iter::{repeat_with, Product, Sum};
use std::ops::{Add, AddAssign, Div, Mul, Neg, Sub};

use num::{One, Zero};

pub mod traits;
pub mod print;

use traits::{CommutativeRing, Field, FromUsize};

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Polynomial<Ring> {
    pub(crate) coeffs: Vec<Ring>,
}

impl<Ring: CommutativeRing> Polynomial<Ring> {
    /// The coefficients of the polynomial in order of increasing degree.
    ///
    /// There must be no trailing zeros.
    pub const fn new(coeffs: Vec<Ring>) -> Self {
        Self { coeffs }
    }

    pub fn new_trim_zeroes(coeffs: Vec<Ring>) -> Self {
        let mut this = Self { coeffs };
        this.trim_zeros();
        this
    }

    pub fn trim_zeros(&mut self) {
        self.coeffs.truncate(
            self.coeffs.len() - self.coeffs.iter().rev().take_while(|x| x.is_zero()).count(),
        );
    }

    pub fn take(&mut self) -> Self {
        Self {
            coeffs: std::mem::take(&mut self.coeffs),
        }
    }

    pub fn from_elem_with_degree(x: Ring, degree: usize) -> Self {
        Self {
            coeffs: repeat_with(Ring::zero)
                .take(degree)
                .chain(std::iter::once(x))
                .collect(),
        }
    }

    pub fn into_iter(self) -> impl ExactSizeIterator<Item = Ring> {
        self.coeffs.into_iter()
    }

    /// The degree of the polynomial. `None` if the polynomial is zero.
    pub fn degree(&self) -> Option<usize> {
        self.coeffs.len().checked_sub(1)
    }

    /// The leading coefficient of the polynomial.
    ///
    /// `None` if the polynomial is zero. (should technically be `Ring::zero()`, but we avoid cloning by taking a reference)
    pub fn leading_coefficient(&self) -> Option<&Ring> {
        self.coeffs.last()
    }

    pub fn leading_coefficient_cloned(&self) -> Ring {
        self.coeffs.last().cloned().unwrap_or_else(Ring::zero)
    }

    pub fn coeff_at(&self, i: usize) -> &Ring {
        &self.coeffs[i]
    }

    /// `"x^2 + 1".raise_by(2) = "x^4 + x^2"`
    pub fn raise_by(&mut self, n: usize) {
        self.coeffs.resize_with(self.coeffs.len() + n, Ring::zero);
        self.coeffs.rotate_right(n);
    }

    pub fn raised_by(mut self, n: usize) -> Self {
        self.raise_by(n);
        self
    }

    /// Multiples all the coefficients by a given scalar.
    pub fn scalar_mul_mut(&mut self, x: Ring) {
        for coeff in self.coeffs.iter_mut() {
            let c = std::mem::replace(coeff, Ring::zero());
            *coeff = c * x.clone();
        }
    }

    #[must_use]
    pub fn scalar_mul(mut self, x: Ring) -> Self {
        self.scalar_mul_mut(x);
        self
    }

    /// Performs differentiation with respect to the polynomial's variable
    pub fn derive_in_place(&mut self)
    where
        Ring: FromUsize,
    {
        if self.coeffs.len() <= 1 {
            self.coeffs.clear();
            return;
        }
        for (i, coeff) in self.coeffs.iter_mut().enumerate().skip(2) {
            let c = std::mem::replace(coeff, Ring::zero());
            *coeff = c * Ring::from_usize(i);
        }
        self.coeffs.remove(0);
    }

    pub fn derivative(mut self) -> Self
    where
        Ring: FromUsize,
    {
        self.derive_in_place();
        self
    }

    /// Performs polynomial division, returns a (quotient, remainder) tuple.
    pub fn div_rem(self, other: Polynomial<Ring>) -> (Polynomial<Ring>, Polynomial<Ring>)
    where
        Ring: Div<Ring, Output = Ring>,
    {
        let mut quotient = Polynomial::zero();
        let mut remainder = self;

        let Some(mut m) = remainder.degree() else {
            // remainder is zero, just return (0, 0).
            return (quotient, remainder);
        };

        let n = other.degree().unwrap();

        let lcv = other.leading_coefficient_cloned();

        while m >= n {
            let lcr = remainder.leading_coefficient_cloned();
            let s = lcr.clone() / lcv.clone();
            quotient += Polynomial::from_elem_with_degree(s.clone(), m - n);
            let poly = (other.clone() - Polynomial::from_elem_with_degree(lcv.clone(), n))
                .scalar_mul(s)
                .raised_by(m - n);
            remainder = (remainder - Polynomial::from_elem_with_degree(lcr, m)) - poly;
            m = remainder.degree().unwrap_or(0);
        }

        (quotient, remainder)
    }

    /// Returns a *monic* polynomial that is a factor in both `self` and `other`.
    pub fn gcd(mut self, mut other: Self) -> Self
    where
        Ring: Field,
    {
        if self.is_zero() && other.is_zero() {
            return Polynomial::zero();
        }
        while !other.is_zero() {
            let r = self.clone().div_rem(other.clone()).1;
            self = other;
            other = r;
        }
        let lc = self
            .leading_coefficient_cloned()
            .clone()
            .checked_inv()
            .unwrap();
        self.scalar_mul(lc)
    }
}

impl<Ring: CommutativeRing> FromIterator<Ring> for Polynomial<Ring> {
    fn from_iter<T: IntoIterator<Item = Ring>>(iter: T) -> Self {
        Self::new(iter.into_iter().collect())
    }
}

impl<Ring: CommutativeRing> Zero for Polynomial<Ring> {
    fn is_zero(&self) -> bool {
        self.coeffs.is_empty()
    }

    fn zero() -> Self {
        Self::new(vec![])
    }

    fn set_zero(&mut self) {
        self.coeffs.clear();
    }
}

impl<Ring: CommutativeRing> One for Polynomial<Ring> {
    fn one() -> Self {
        Self::new(vec![Ring::one()])
    }
}

impl<Ring: CommutativeRing> Add for Polynomial<Ring> {
    type Output = Polynomial<Ring>;

    fn add(self, other: Self) -> Self {
        let mut coeffs = Vec::new();
        let (long, short) = if self.coeffs.len() > other.coeffs.len() {
            (self, other)
        } else {
            (other, self)
        };
        for (a, b) in long
            .coeffs
            .into_iter()
            .zip(short.coeffs.into_iter().chain(repeat_with(Ring::zero)))
        {
            coeffs.push(a + b);
        }
        Polynomial::new_trim_zeroes(coeffs)
    }
}

impl<Ring: CommutativeRing> Sub for Polynomial<Ring> {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        self + rhs.neg()
    }
}

impl<Ring: CommutativeRing> AddAssign for Polynomial<Ring> {
    fn add_assign(&mut self, rhs: Self) {
        let lhs = self.take();
        *self = lhs + rhs
    }
}

impl<Ring: CommutativeRing> Mul for Polynomial<Ring> {
    type Output = Polynomial<Ring>;
    fn mul(self, rhs: Self) -> Self::Output {
        let mut coeffs = Vec::new();
        for (i, a) in self.coeffs.into_iter().enumerate() {
            let mut new_coeffs = repeat_with(Ring::zero).take(i).collect::<Vec<_>>();
            for b in rhs.coeffs.iter().cloned() {
                new_coeffs.push(a.clone() * b);
            }
            coeffs.push(Polynomial::new(new_coeffs));
        }
        coeffs
            .into_iter()
            .fold(Polynomial::new(vec![]), |a, b| a + b)
    }
}

impl<Ring: CommutativeRing + PartialEq> Mul<Ring> for Polynomial<Ring> {
    type Output = Polynomial<Ring>;
    fn mul(mut self, rhs: Ring) -> Self::Output {
        if rhs.is_zero() {
            self.set_zero();
            self
        } else if rhs.is_one() {
            self
        } else {
            self.coeffs = self.coeffs.into_iter().map(|a| a * rhs.clone()).collect();
            self
        }
    }
}

impl<Ring: CommutativeRing> Product for Polynomial<Ring> {
    fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::one(), |a, b| a * b)
    }
}

impl<Ring: CommutativeRing> Sum for Polynomial<Ring> {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::zero(), |a, b| a + b)
    }
}

impl<Ring: CommutativeRing> Neg for Polynomial<Ring> {
    type Output = Polynomial<Ring>;
    fn neg(self) -> Self::Output {
        Polynomial::new(self.coeffs.into_iter().map(Neg::neg).collect())
    }
}

/// The ring of polynomials over a ring (`R[x]`)
impl<Ring: CommutativeRing> CommutativeRing for Polynomial<Ring> {}