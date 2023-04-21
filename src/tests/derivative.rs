use crate::diff::Differentiator;
use crate::tests::{init, x};

use super::sx;

#[test]
pub fn simple_power_rule() {
    init();

    let one = Differentiator::new()
        .differentiate(sx.simplify(), "x")
        .unwrap();
    assert_eq!(one, 1);

    let two = Differentiator::new()
        .differentiate((x * x).simplify(), "x")
        .unwrap();
    assert_eq!(two, (2 * sx).simplify());

    let three = Differentiator::new()
        .differentiate((x * x * x).simplify(), "x")
        .unwrap();
    assert_eq!(three, (3 * x * x).simplify());
}
