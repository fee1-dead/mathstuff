use crate::parse::{Token, Tokenizer};

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
            Token::Number(4.into())
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
            Token::Number(4.into())
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
            Token::RightParen
        ]
    );
}
