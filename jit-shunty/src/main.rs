use core::fmt;
use std::{collections::VecDeque, fmt::Display};

enum Token {
    Number(f64),
    LeftParen,
    RightParen,
    Plus,
    Minus,
    Multiply,
    Divide,
}

type Expression = Vec<Token>;
impl fmt::Debug for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Token::Number(n) => format!("{n}"),
                Token::LeftParen => "(".to_string(),
                Token::RightParen => ")".to_string(),
                Token::Plus => "+".to_string(),
                Token::Minus => "-".to_string(),
                Token::Multiply => "*".to_string(),
                Token::Divide => "/".to_string(),
            }
        )
    }
}

fn parse(input: &str) -> Result<Vec<Token>, String> {
    let mut r = Vec::new();
    let mut buf = String::new();
    for (i, c) in input.chars().enumerate() {
        if c == ' ' {
            continue;
        }
        if "0123456789.".contains(c) {
            buf.push(c);
            continue;
        }
        if !buf.is_empty() {
            r.push(Token::Number(
                buf.parse().map_err(|e| format!("parsing number: {e}"))?,
            ));
            buf.clear();
        }
        r.push(match c {
            '(' => Token::LeftParen,
            ')' => Token::RightParen,
            '+' => Token::Plus,
            '-' => Token::Minus,
            '*' => Token::Multiply,
            '/' => Token::Divide,
            _ => return Err(format!("incorrect character {c} at postition {i}")),
        })
    }
    if !buf.is_empty() {
        r.push(Token::Number(
            buf.parse().map_err(|e| format!("parsing number: {e}"))?,
        ));
    }
    Ok(r)
}

fn precedence(t: &Token) -> u32 {
    match t {
        Token::Number(_) => 0,
        Token::LeftParen => 0,
        Token::RightParen => 0,
        Token::Plus => 1,
        Token::Minus => 1,
        Token::Multiply => 2,
        Token::Divide => 2,
    }
}

fn shunting_yard(input: Vec<Token>) -> Result<Vec<Token>, String> {
    let mut out = Vec::new();
    let mut ops = Vec::new();
    'tokens: for token in input {
        match token {
            Token::Number(n) => out.push(Token::Number(n)),
            Token::LeftParen => ops.push(Token::LeftParen),
            Token::RightParen => {
                while let Some(op) = ops.pop() {
                    out.push(match op {
                        op @ Token::Plus
                        | op @ Token::Minus
                        | op @ Token::Multiply
                        | op @ Token::Divide => op,
                        Token::LeftParen => continue 'tokens,
                        _ => return Err("unexpected token".to_string()),
                    })
                }
                return Err("missing parentesis".to_string());
            }
            op @ Token::Plus | op @ Token::Minus | op @ Token::Multiply | op @ Token::Divide => {
                while let Some(op2) = ops.pop() {
                    if precedence(&op2) < precedence(&op) {
                        ops.push(op2);
                        break;
                    }
                    out.push(op2);
                }
                ops.push(op);
            }
        }
    }
    while let Some(op) = ops.pop() {
        out.push(op);
    }
    Ok(out)
}

fn token_op(t: Token) -> impl Fn(f64, f64) -> f64 {
    match t {
        Token::Number(_) | Token::LeftParen | Token::RightParen => |_, _| 0.0,
        Token::Plus => |x, y| x + y,
        Token::Minus => |x, y| x - y,
        Token::Multiply => |x, y| x * y,
        Token::Divide => |x, y| x / y,
    }
}

fn token_comp(t: Token) -> Vec<u8> {
    match t {
        Token::Number(_) | Token::LeftParen | Token::RightParen => Vec::new(),
        Token::Plus => vec![
            0xfd, 0x0f, 0x11, 0x44, 0x24, 0xf0, // movsd [rsp-0x10], xmm0
            0xfd, 0x0f, 0x11, 0x4c, 0x24, 0xf8, // movsd [rsp-0x8], xmm1
            0xfd, 0x0f, 0x58, 0xc1, // addsd xmm0, xmm1
            0xc3, // ret
        ],
        Token::Minus => vec![
            0xfd, 0x0f, 0x11, 0x44, 0x24, 0xf0, // movsd [rsp-0x10], xmm0
            0xfd, 0x0f, 0x11, 0x4c, 0x24, 0xf8, // movsd [rsp-0x8], xmm1
            0xfd, 0x0f, 0x5c, 0xc1, // subsd xmm0, xmm1
            0xc3, // ret
        ],
        Token::Multiply => vec![
            0xfd, 0x0f, 0x11, 0x44, 0x24, 0xf0, // movsd [rsp-0x10], xmm0
            0xfd, 0x0f, 0x11, 0x4c, 0x24, 0xf8, // movsd [rsp-0x8], xmm1
            0xfd, 0x0f, 0x59, 0xc1, // mulsd xmm0, xmm1
            0xc3, // ret
        ],
        Token::Divide => vec![
            0xfd, 0x0f, 0x11, 0x44, 0x24, 0xf0, // movsd [rsp-0x10], xmm0
            0xfd, 0x0f, 0x11, 0x4c, 0x24, 0xf8, // movsd [rsp-0x8], xmm1
            0xf2, 0x0f, 0x5e, 0xc1, // addsd xmm0, xmm1
            0xc3, // ret
        ],
    }
}

fn calculate(input: Vec<Token>) -> Result<f64, String> {
    let mut s = Vec::new();
    for t in input {
        match t {
            t @ Token::Number(_) => s.push(t),
            Token::LeftParen | Token::RightParen => {
                return Err("unexpected parenthesis".to_string())
            }
            op @ Token::Plus | op @ Token::Minus | op @ Token::Multiply | op @ Token::Divide => {
                if let (Some(Token::Number(y)), Some(Token::Number(x))) = (s.pop(), s.pop()) {
                    s.push(Token::Number(token_op(op)(x, y)));
                }
            }
        }
    }
    if let Some(Token::Number(result)) = s.pop() {
        Ok(result)
    } else {
        Err("something bad".to_string())
    }
}

fn main() {
    let input = "2+8*(5-3)+30*2+10";
    println!("{input}");
    let exp = parse(input).unwrap();
    let rpn = shunting_yard(exp).unwrap();
    println!("{rpn:?}");
    println!("{}", calculate(rpn).unwrap());
}
