use std::cmp::Ordering;
use std::slice;

use crate::BasicAlgebraicExpr;

impl PartialOrd for BasicAlgebraicExpr {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn cmp_list<T: Ord>(a: &[T], b: &[T]) -> Ordering {
    let elems = a.iter().zip(b.iter()).rev();

    for (a, b) in elems {
        match a.cmp(b) {
            Ordering::Equal => continue,
            x => return x,
        }
    }

    a.len().cmp(&b.len())
}

impl Ord for BasicAlgebraicExpr {
    fn cmp(&self, other: &Self) -> Ordering {
        use BasicAlgebraicExpr::*;
        match (self, other) {
            (a, b) if a == b => Ordering::Equal,
            (Numeric(a), Numeric(b)) => a.cmp(b),
            (Numeric(_), _) => Ordering::Less,
            (_, Numeric(_)) => Ordering::Greater,
            (Product(a), Product(b)) => cmp_list(a, b),
            (Product(a), b) => cmp_list(a, slice::from_ref(b)),
            (a, Product(b)) => cmp_list(slice::from_ref(a), b),
            (Pow(a), Pow(b)) => a.cmp(b),
            (Pow(a), b) => a.cmp(&Box::new((b.clone(), 1.into()))),
            (a, Pow(b)) => (&(a.clone(), 1.into())).cmp(b),
            (Sum(a), Sum(b)) => cmp_list(a, b),
            (Sum(a), b) => cmp_list(a, slice::from_ref(b)),
            (a, Sum(b)) => cmp_list(slice::from_ref(a), b),
            (Factorial(a), Factorial(b)) => a.cmp(b),
            (Factorial(a), b) => {
                if &**a == b {
                    Ordering::Greater
                } else {
                    (**a).cmp(b)
                }
            }
            (a, Factorial(b)) => {
                if &**b == a {
                    Ordering::Less
                } else {
                    a.cmp(&**b)
                }
            }
            // N.B. arguments are compared lexicographically instead of reversed as in cmp_list
            (Function(name1, args1), Function(name2, args2)) => {
                if name1 == name2 {
                    args1.cmp(args2)
                } else {
                    name1.cmp(name2)
                }
            }
            (Function(name1, _), Symbol(name2)) => {
                if name1 == name2 {
                    Ordering::Greater
                } else {
                    name1.cmp(name2)
                }
            }
            (Symbol(name1), Function(name2, _)) => {
                if name1 == name2 {
                    Ordering::Less
                } else {
                    name1.cmp(name2)
                }
            }
            (Symbol(name1), Symbol(name2)) => name1.cmp(name2),
        }
    }
}
