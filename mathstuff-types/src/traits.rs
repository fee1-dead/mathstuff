use std::ops::{Div, Neg};

use num::{BigInt, BigRational, One, Zero};

/// Performs checked inverse.
pub trait CheckedInv {
    fn checked_inv(&self) -> Option<Self>
    where
        Self: Sized;
}

pub trait CommutativeRing: Zero + One + Neg<Output = Self> + Clone {
    fn sub(self, other: Self) -> Self {
        self.add(other.neg())
    }
}

pub trait FromUsize {
    fn from_usize(n: usize) -> Self;
}

impl FromUsize for BigInt {
    fn from_usize(n: usize) -> Self {
        Self::from(n)
    }
}

impl FromUsize for BigRational {
    fn from_usize(n: usize) -> Self {
        Self::from(BigInt::from(n))
    }
}

/// any implementors of this trait have their set of field elements represented
/// by the possible values the implementor type can take.
pub trait Field: CommutativeRing + CheckedInv + Div<Output = Self> {
    fn div(self, other: Self) -> Option<Self> {
        other.checked_inv().map(|b| self.mul(b))
    }
}

impl CheckedInv for BigRational {
    fn checked_inv(&self) -> Option<Self> {
        if self.is_zero() {
            None
        } else {
            Some(self.recip())
        }
    }
}

/// The ring of integers (`Z`)
impl CommutativeRing for BigInt {}

impl CommutativeRing for i64 {}

/// The ring of rationals (`Q`)
impl CommutativeRing for BigRational {}

/// The field of rationals (`Q`)
impl Field for BigRational {}
