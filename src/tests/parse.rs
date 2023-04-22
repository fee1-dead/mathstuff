use crate::parse::{parse_into_expression, Token, Tokenizer};
use crate::BasicAlgebraicExpr;

fn int_expr(x: i128) -> BasicAlgebraicExpr {
    BasicAlgebraicExpr::Numeric(x.into())
}

#[test]
pub fn tokenize() {
    let mut tokenizer = Tokenizer::new("2 + 3 * 4");
    assert_eq!(
        tokenizer.scan_tokens(),
        vec![
            Token::Number(2.into()),
            Token::Add,
            Token::Number(3.into()),
            Token::Mul,
            Token::Number(4.into()),
        ]
    );

    let mut tokenizer = Tokenizer::new("(2 + 3) * 4");
    assert_eq!(
        tokenizer.scan_tokens(),
        vec![
            Token::LeftParen,
            Token::Number(2.into()),
            Token::Add,
            Token::Number(3.into()),
            Token::RightParen,
            Token::Mul,
            Token::Number(4.into()),
        ]
    );

    let mut tokenizer = Tokenizer::new("Sin[2x] - 3xyz^(4+5)");
    assert_eq!(
        tokenizer.scan_tokens(),
        vec![
            Token::Symbol("Sin".to_string()),
            Token::LeftBr,
            Token::Number(2.into()),
            Token::Symbol("x".to_string()),
            Token::RightBr,
            Token::Sub,
            Token::Number(3.into()),
            Token::Symbol("xyz".to_string()),
            Token::Pow,
            Token::LeftParen,
            Token::Number(4.into()),
            Token::Add,
            Token::Number(5.into()),
            Token::RightParen,
        ]
    );
}

#[test]
pub fn parse() {
    let x = parse_into_expression("2 + 3 * 4").unwrap();
    assert_eq!(
        BasicAlgebraicExpr::Sum(vec![
            int_expr(2),
            BasicAlgebraicExpr::Product(vec![int_expr(3), int_expr(4)])
        ]),
        x
    );

    let x = parse_into_expression("(2 + 3) * 4").unwrap();
    assert_eq!(
        BasicAlgebraicExpr::Product(vec![
            BasicAlgebraicExpr::Sum(vec![int_expr(2), int_expr(3)]),
            int_expr(4),
        ]),
        x
    );
}
