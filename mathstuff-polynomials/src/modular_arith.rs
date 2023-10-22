use std::ops::{Add, Rem};


pub struct Mod<M> {
    pub modulus: M,
}

impl<'a, A, B, M> Add<(A, B)> for &'a Mod<M>
where
    A: Add<B>,
    A::Output: Rem<&'a M>,
{
    type Output = <A::Output as Rem<&'a M>>::Output;
    fn add(self, (a, b): (A, B)) -> Self::Output {
        (a + b) % &self.modulus
    }
}

pub struct ModdedInteger<I, M> {
    pub modulus: M,
    pub value: I,
}

impl<I: Add, M: Clone> Add for ModdedInteger<I, M>
where
    I::Output: Rem<M>,
{
    type Output = ModdedInteger<<<I as Add>::Output as Rem<M>>::Output, M>;
    fn add(self, rhs: Self) -> Self::Output {
        let value = (self.value + rhs.value) % self.modulus.clone();
        ModdedInteger {
            modulus: self.modulus,
            value,
        }
    }
}


