use crate::BasicAlgebraicExpr as Expr;

pub fn assert_parses(x: &str, exp: Expr) {
    let actual = super::parse_into_expression(x).expect("expected parse success");
    assert_eq!(exp, actual);
}

pub fn symbol(x: impl Into<String>) -> Expr {
    Expr::Symbol(x.into())
}

pub fn pow(base: Expr, exp: Expr) -> Expr {
    Expr::Pow(Box::new((base, exp)))
}

pub fn const_int(x: i128) -> Expr {
    Expr::Numeric(x.into())
}

pub fn fun(name: impl Into<String>, args: impl IntoIterator<Item = Expr>) -> Expr {
    Expr::Function(name.into(), args.into_iter().collect())
}

#[test]
pub fn parsing() {
    use Expr::*;
    assert_parses(
        "x^3 + 3 * x^2 * y + 3 * x * y^2 + y^3",
        Sum(vec![
            pow(symbol("x"), const_int(3)),
            Product(vec![
                const_int(3),
                pow(symbol("x"), const_int(2)),
                symbol("y"),
            ]),
            Product(vec![
                const_int(3),
                symbol("x"),
                pow(symbol("y"), const_int(2)),
            ]),
            pow(symbol("y"), const_int(3)),
        ]),
    );

    assert_parses(
        "3 * x * (x + 1) * y^2 * z^n",
        Product(vec![
            const_int(3),
            symbol("x"),
            Sum(vec![symbol("x"), const_int(1)]),
            pow(symbol("y"), const_int(2)),
            pow(symbol("z"), symbol("n")),
        ]),
    );

    assert_parses(
        "a * Sin[x]^2 + 2 * b * Sin[x] + 3 * c",
        Sum(vec![
            Product(vec![
                symbol("a"),
                pow(fun("Sin", [symbol("x")]), const_int(2)),
            ]),
            Product(vec![const_int(2), symbol("b"), fun("Sin", [symbol("x")])]),
            Product(vec![const_int(3), symbol("c")]),
        ]),
    );
}
