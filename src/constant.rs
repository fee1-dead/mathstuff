use std::fmt;
use std::iter::{Product, Sum};
use std::ops::{Add, Deref, DerefMut, Div, Mul, Neg, Rem, Sub};

use num::bigint::Sign;
use num::traits::Pow;
use num::{BigInt, BigRational, Num, One, Signed, Zero};

#[derive(PartialEq, Eq, Clone, PartialOrd, Ord, Hash)]
pub struct Constant(BigRational);

impl Constant {
    pub fn is_integer(&self) -> bool {
        self.0.is_integer()
    }
    pub fn as_integer(&self) -> Option<&BigInt> {
        self.is_integer().then(|| self.0.numer())
    }
    pub fn into_inner(self) -> BigRational {
        self.0
    }

    pub fn negative_one() -> Self {
        Self(BigRational::new(
            BigInt::new(Sign::Minus, vec![1]),
            BigInt::one(),
        ))
    }
}

impl From<i128> for Constant {
    fn from(x: i128) -> Self {
        Self(BigRational::from_integer(x.into()))
    }
}

impl Deref for Constant {
    type Target = BigRational;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Constant {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl fmt::Debug for Constant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(x) = self.as_integer() {
            x.fmt(f)
        } else {
            write!(f, "{} / {}", self.numer(), self.denom())
        }
    }
}

impl Neg for Constant {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self(-self.0)
    }
}

impl Div for Constant {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Self(self.0 / rhs.0)
    }
}

impl Add for Constant {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl Sub for Constant {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl Rem for Constant {
    type Output = Self;

    fn rem(self, rhs: Self) -> Self::Output {
        Self(self.0 % rhs.0)
    }
}

impl Mul for Constant {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self(self.0 * rhs.0)
    }
}

impl One for Constant {
    fn one() -> Self {
        Self(BigRational::one())
    }

    fn is_one(&self) -> bool {
        self.0.is_one()
    }

    fn set_one(&mut self) {
        self.0.set_one()
    }
}

impl Zero for Constant {
    fn zero() -> Self {
        Self(BigRational::zero())
    }

    fn is_zero(&self) -> bool {
        self.0.is_zero()
    }

    fn set_zero(&mut self) {
        self.0.set_zero()
    }
}

impl<'a> Pow<&'a BigInt> for Constant {
    type Output = Self;
    fn pow(self, rhs: &'a BigInt) -> Self::Output {
        Self(self.0.pow(rhs))
    }
}

impl Num for Constant {
    type FromStrRadixErr = <BigRational as Num>::FromStrRadixErr;

    fn from_str_radix(str: &str, radix: u32) -> Result<Self, Self::FromStrRadixErr> {
        BigRational::from_str_radix(str, radix).map(Self)
    }
}

impl Sum for Constant {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        Self(iter.map(|x| x.0).sum())
    }
}

impl Product for Constant {
    fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
        Self(iter.map(|x| x.0).product())
    }
}

impl Signed for Constant {
    fn abs(&self) -> Self {
        Self(self.0.abs())
    }

    fn abs_sub(&self, other: &Self) -> Self {
        Self(self.0.abs_sub(&other.0))
    }

    fn signum(&self) -> Self {
        Self(self.0.signum())
    }

    fn is_positive(&self) -> bool {
        self.0.is_positive()
    }

    fn is_negative(&self) -> bool {
        self.0.is_negative()
    }
}

impl From<BigInt> for Constant {
    fn from(x: BigInt) -> Self {
        Self(x.into())
    }
}

impl From<BigRational> for Constant {
    fn from(x: BigRational) -> Self {
        Self(x)
    }
}
