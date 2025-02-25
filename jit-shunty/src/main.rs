#[derive(Debug)]
enum Token {
    Number(f64),
    LeftParen,
    RightParen,
    Plus,
    Minus,
    Multiply,
    Divide,
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
    let exp = parse("2+2.3*(3-2)").unwrap();
    let rpn = shunting_yard(exp).unwrap();
    println!("{rpn:?}");
    println!("{}", calculate(rpn).unwrap());
}
