use mathstuff_types::{Polynomial, print::DisplayWithVar};

use crate::factorization::SquareFreeFactorization;

macro_rules! v {
    ($($elem:expr),*) => {
        vec![
            $(num::BigRational::from_integer(num::BigInt::from($elem))),*
        ]
    };
}

/* macro_rules! p {
    (@handle_rest[$v:ident]($($lit:literal)? x $($tt:tt)*)) => {
        v[1] += num::BigRational::from_integer(num::BigInt::from(1 $(- 1 + $lit)?));
        p!(@handle_rest[$v]( $($tt)* ));
    };
    (@handle_rest[$v:ident]()) => {{}};
    ($($lit:literal)? x^$exp:literal)$( $(+)? $($lits:literal)? x^$exps:literal )* $($tt:tt)*) => {{
        let mut v = vec![ <num::BigRational as num::traits::Zero>::zero(); [$exp $(, $exps)*].into_iter().max().unwrap() + 1];
        v[1 $(- 1 + $exp)?] = num::BigRational::from_integer(num::BigInt::from(1 $(- 1 + $lit)?));
        $(
            v[$exps] += num::BigRational::from_integer(num::BigInt::from(1 $(- 1 + $lits)?));
        )*
        p!(@handle_rest[v]($($tt)*));
        Polynomial::new(v)
    }};
} */

#[test]
pub fn test_div() {
    // 3x^4 - 5x^2 + 3 / x + 2
    let a = Polynomial::new(vec![3, 0, -5, 0, 3]);
    let b = Polynomial::new(vec![2, 1]);

    let (q, r) = a.div_rem(b);
    assert_eq!(q, Polynomial::new(vec![-14, 7, -6, 3]));
    assert_eq!(r, Polynomial::new(vec![31]));

    let a = Polynomial::new(vec![4, -3, 2, 1]);
    let b = Polynomial::new(vec![-7, 1]);
    let (q, r) = a.div_rem(b);
    assert_eq!(q, Polynomial::new(vec![60, 9, 1]));
    assert_eq!(r, Polynomial::new(vec![424]));
}

#[test]
pub fn test_gcd() {
    let u = Polynomial::new(v![3, -6, -2, 17, 4, -3, 7, 5, 1]);
    let v = Polynomial::new(v![8, 9, 2, 8, 10, 1, 3, 6, 1]);
    assert_eq!(Polynomial::new(v![1, 1]), u.gcd(v));
}

#[test]
pub fn test_square_free_factorization() {
    let u = Polynomial::new(v![-16, -24, -4, 10, 6, 1]);
    assert_eq!(
        "x^5 + 6x^4 + 10x^3 - 4x^2 - 24x - 16",
        u.print_with_var("x").to_string().as_str()
    );

    let sqf = SquareFreeFactorization::factor_polynomial(u.clone());
    assert_eq!(
        "(x^2 - 2)(x + 2)^3",
        sqf.print_with_var("x").to_string().as_str()
    );

    let u = Polynomial::new(v![24, 132, 90, -525, -750]);
    assert_eq!(
        "-750x^4 - 525x^3 + 90x^2 + 132x + 24",
        u.print_with_var("x").to_string().as_str()
    );

    let sqf = SquareFreeFactorization::factor_polynomial(u.clone());
    assert_eq!(
        "-3(2x - 1)(5x + 2)^3",
        sqf.print_with_var("x").to_string().as_str()
    );
}
