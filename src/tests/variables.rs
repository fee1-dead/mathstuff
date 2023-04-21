use super::{sx, sy, x, y};

#[test]
pub fn add_vars() {
    let exp = x + 2 * y;
    assert_eq!(sx + 2 * sy, exp);
}
