pub mod token;

use std::str::FromStr;

use token::{Token, Tokenizer};

#[derive(PartialEq, Clone, Debug)]
pub enum Expr {
    Brackets(Box<Expr>),
    Op(Oper, Box<Expr>, Box<Expr>),
    Num(i64),
}

#[derive(PartialEq, Clone, Debug)]
pub enum Oper {
    Add,
    Sub,
    Div,
    Mul,
}

pub fn op(o: Oper, a: Expr, b: Expr) -> Expr {
    Expr::Op(o, Box::new(a), Box::new(b))
}

impl FromStr for Expr {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let t = Tokenizer::new(s);
        let (_tokenizer, expr) = sub(&t)?;

        Ok(expr)
    }
}

pub type ParseRes<'a, T> = Result<(Tokenizer<'a>, T), String>;

pub fn token_bool<'a, F: Fn(&Token) -> bool>(t: &Tokenizer<'a>, f: F) -> ParseRes<'a, Token> {
    let mut it = t.clone();

    match it.next() {
        Some(Ok(v)) if f(&v) => Ok((it, v)),
        _ => Err("Token bool test failed.".to_string()),
    }
}

pub fn brackets<'a>(t: &Tokenizer<'a>) -> ParseRes<'a, Expr> {
    let it = t.clone();

    let (it, _) = token_bool(&it, |t| *t == Token::BrOpen)?;
    let (it, res) = sub(&it)?;
    let (it, _) = token_bool(&it, |t| *t == Token::BrClose)?;

    Ok((it, Expr::Brackets(Box::new(res))))
}

pub fn item<'a>(t: &Tokenizer<'a>) -> ParseRes<'a, Expr> {
    if let Ok(v) = brackets(t) {
        return Ok(v);
    }

    let mut it = t.clone();
    match it.next() {
        Some(Ok(Token::Num(n))) => Ok((it, Expr::Num(n))),
        _ => Err("No Number or brackets found".to_string()),
    }
}

// ordered by bodmas
pub fn div<'a>(t: &Tokenizer<'a>) -> ParseRes<'a, Expr> {
    // left hand side
    let (it, left) = item(t)?;

    if let Ok((divit, _)) = token_bool(&it, |v| *v == Token::Div) {
        let (rit, right) = div(&divit)?;
        Ok((rit, op(Oper::Div, left, right)))
    } else {
        Ok((it, left))
    }
}

// ordered by bodmas
pub fn mul<'a>(t: &Tokenizer<'a>) -> ParseRes<'a, Expr> {
    // left hand side
    let (it, left) = div(t)?;

    if let Ok((divit, _)) = token_bool(&it, |v| *v == Token::Mul) {
        let (rit, right) = mul(&divit)?;
        Ok((rit, op(Oper::Mul, left, right)))
    } else {
        Ok((it, left))
    }
}

// ordered by bodmas
pub fn add<'a>(t: &Tokenizer<'a>) -> ParseRes<'a, Expr> {
    // left hand side
    let (it, left) = mul(t)?;

    if let Ok((divit, _)) = token_bool(&it, |v| *v == Token::Add) {
        let (rit, right) = add(&divit)?;
        Ok((rit, op(Oper::Add, left, right)))
    } else {
        Ok((it, left))
    }
}

// ordered by bodmas
pub fn sub<'a>(t: &Tokenizer<'a>) -> ParseRes<'a, Expr> {
    // left hand side
    let (it, left) = add(t)?;

    if let Ok((divit, _)) = token_bool(&it, |v| *v == Token::Sub) {
        let (rit, right) = sub(&divit)?;
        Ok((rit, op(Oper::Sub, left, right)))
    } else {
        Ok((it, left))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strs() {
        let e: Expr = "3 + 5 *   (7-3)".parse().unwrap();

        assert_eq!(
            e,
            op(
                Oper::Add,
                Expr::Num(3),
                op(
                    Oper::Mul,
                    Expr::Num(5),
                    Expr::Brackets(Box::new(op(Oper::Sub, Expr::Num(7), Expr::Num(3))))
                )
            )
        );
    }
}
