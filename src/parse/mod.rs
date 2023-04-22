use std::fmt::{Display, Write};
// a high precedence means that it will be evaluated before lower precedences,
// and thus parsed before other operation types as well.
//
// Precedence list
// 1. Parentheses, function calls
// 2. Exponentiation
// 3. Multiplication, division, modulo
// 4. Addition, subtraction
//
// Tokens are either numbers or symbols. Function calls must be following symbols
use std::iter::Peekable;
use std::str::Chars;

use num::BigInt;

use crate::BasicAlgebraicExpr;

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum Token {
    Number(BigInt),
    Symbol(String),
    LeftBr,
    RightBr,
    LeftParen,
    RightParen,
    Add,
    Sub,
    Div,
    Mul,
    Pow,
    Factorial,
    Comma,
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ch = match self {
            Token::LeftBr => '[',
            Token::RightBr => ']',
            Token::LeftParen => '(',
            Token::RightParen => ')',
            Token::Add => '+',
            Token::Sub => '-',
            Token::Div => '/',
            Token::Mul => '*',
            Token::Pow => '^',
            Token::Factorial => '!',
            Token::Comma => ',',
            Token::Number(n) => {
                return write!(f, "{n}");
            }
            Token::Symbol(s) => {
                return f.write_str(s);
            }
        };

        f.write_char(ch)
    }
}

pub struct Tokenizer<'a> {
    s: &'a str,
    chars: Peekable<Chars<'a>>,
    start: usize,
    current: usize,
}

impl<'a> Tokenizer<'a> {
    pub fn new(s: &'a str) -> Self {
        Self {
            s,
            chars: s.chars().peekable(),
            start: 0,
            current: 0,
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if ch == ' ' || ch == '\n' {
                self.advance();
            } else {
                break;
            }
        }
    }
    fn peek(&mut self) -> Option<char> {
        self.chars.peek().copied()
    }
    fn advance(&mut self) -> Option<char> {
        self.current += 1;
        self.chars.next()
    }
    fn number(&mut self) -> Option<Token> {
        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() {
                self.advance();
            } else {
                break;
            }
        }
        Some(Token::Number(
            self.s[self.start..self.current]
                .parse()
                .expect("TODO, TODO, TODO TODO TODO"),
        ))
    }
    fn symbol(&mut self) -> Option<Token> {
        while let Some(ch) = self.peek() {
            if ch.is_ascii_alphabetic() {
                self.advance();
            } else {
                break;
            }
        }
        Some(Token::Symbol(self.s[self.start..self.current].to_string()))
    }
    fn scan_token(&mut self) -> Option<Token> {
        self.skip_whitespace();
        self.start = self.current;

        match self.advance()? {
            '(' => Some(Token::LeftParen),
            ')' => Some(Token::RightParen),
            '[' => Some(Token::LeftBr),
            ']' => Some(Token::RightBr),
            '+' => Some(Token::Add),
            '-' => Some(Token::Sub),
            '*' => Some(Token::Mul),
            '/' => Some(Token::Div),
            '^' => Some(Token::Pow),
            '!' => Some(Token::Factorial),
            ',' => Some(Token::Comma),
            x if x.is_ascii_digit() => self.number(),
            x if x.is_ascii_alphanumeric() => self.symbol(),
            _ => panic!("AAAAAAAAAAAAH"),
        }
    }

    pub fn scan_tokens(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        while let Some(token) = self.scan_token() {
            tokens.push(token);
        }
        tokens
    }
}

pub fn parse(s: &str) -> Vec<Token> {
    Tokenizer::new(s).scan_tokens()
}

use chumsky::prelude::*;

fn expression_parser() -> impl Parser<Token, BasicAlgebraicExpr, Error = Simple<Token>> {
    use super::BasicAlgebraicExpr as Expr;

    fn negative_one() -> Expr {
        Expr::Numeric((-1).into())
    }

    fn add(a: Expr, b: Expr) -> Expr {
        Expr::Sum(vec![a, b])
    }
    fn sub(a: Expr, b: Expr) -> Expr {
        Expr::Sum(vec![a, Expr::Product(vec![negative_one(), b])])
    }

    fn mul(a: Expr, b: Expr) -> Expr {
        Expr::Product(vec![a, b])
    }

    fn div(a: Expr, b: Expr) -> Expr {
        Expr::Product(vec![a, Expr::Pow(Box::new((b, negative_one())))])
    }

    let expr = recursive(|expr| {
        let int = filter_map(|sp, x| match x {
            Token::Number(n) => Ok(n),
            _ => Err(Simple::custom(sp, "expected number")),
        });

        let symbol = filter_map(|sp, x| match x {
            Token::Symbol(s) => Ok(s),
            _ => Err(Simple::custom(sp, "expected symbol")),
        });

        let call = symbol
            .clone()
            .then_ignore(just(Token::LeftBr))
            .then(
                expr.clone()
                    .separated_by(just(Token::Comma))
                    .allow_trailing(),
            )
            .then_ignore(just(Token::RightBr))
            .map(|(name, args)| Expr::Function(name, args));

        let atom = int
            .map(|i| Expr::Numeric(i.into()))
            .or(expr.delimited_by(just(Token::LeftParen), just(Token::RightParen)))
            .or(call)
            .or(symbol.map(|x| Expr::Symbol(x)));

        let factorial = atom
            .then(just(Token::Factorial).repeated())
            .foldl(|x, _| Expr::Factorial(Box::new(x)));

        let exp = factorial
            .separated_by(just(Token::Pow))
            .at_least(1)
            .map(|v| {
                let mut i = v.into_iter().rev();
                let mut exp = i.next().unwrap();
                for base in i {
                    exp = BasicAlgebraicExpr::Pow(Box::new((base, exp)));
                }

                exp
            });

        let unary = just(Token::Sub)
            .repeated()
            .then(exp)
            .foldr(|_, b| BasicAlgebraicExpr::Product(vec![negative_one(), b]));

        let product = unary
            .clone()
            .then(
                just(Token::Mul)
                    .to(mul as fn(_, _) -> _)
                    .or(just(Token::Div).to(div as fn(_, _) -> _))
                    .then(unary)
                    .repeated(),
            )
            .foldl(|lhs, (op, rhs)| op(lhs, rhs));

        let sum = product
            .clone()
            .then(
                just(Token::Add)
                    .to(add as fn(_, _) -> _)
                    .or(just(Token::Sub).to(sub as fn(_, _) -> _))
                    .then(product)
                    .repeated(),
            )
            .foldl(|lhs, (op, rhs)| op(lhs, rhs));

        sum
    });
    expr.then_ignore(end())
}

pub fn parse_into_expression(s: &str) -> Result<BasicAlgebraicExpr, Simple<Token>> {
    expression_parser().parse(parse(s)).map_err(|mut x| {
        let mut err = x.pop().unwrap();
        for e in x {
            err = err.merge(e);
        }
        err
    })
}
