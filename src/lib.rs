pub mod token;

use pest::{Parser, iterators::Pair};
use pest_derive::*;
use std::str::FromStr;

use token::{Token, Tokenizer};

#[derive(Parser)]
#[grammar = "sums.pest"]
pub struct SumsParser;

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

// Panics if r not an Operator
pub fn rule_to_op(r: &Rule) -> Oper {
    match r {
        Rule::sub => Oper::Sub,
        Rule::div => Oper::Div,
        Rule::mul => Oper::Mul,
        Rule::add => Oper::Add,
        _ => unreachable!(),
    }
}

pub fn pair_to_expr(p: Pair<Rule>) -> Expr {
    let rule = p.as_rule();
    if Rule::number == rule {
        return Expr::Num(p.as_str().parse::<i64>().unwrap());
    }

    // iterate over the child rules
    let mut inner = p.into_inner();

    let left = pair_to_expr(inner.next().unwrap());
    let right = inner.next().map(pair_to_expr);

    match rule {
        Rule::item => left,
        Rule::brackets => Expr::Brackets(Box::new(left)),
        Rule::div | Rule::mul | Rule::sub | Rule::add => match right {
            Some(r) => op(rule_to_op(&rule), left, r),
            None => left,
        },
        _ => unreachable!(),
    }
}

impl FromStr for Expr {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut sp = SumsParser::parse(Rule::sub, s).map_err(|e| e.to_string())?;
        Ok(pair_to_expr(sp.next().unwrap()))
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
    use pest::Parser;

    use super::*;

    #[test]
    pub fn test_pest_returns_something() {
        let mut sp = SumsParser::parse(Rule::sub, "23+4").unwrap();
        assert_eq!(sp.next().unwrap().as_rule(), Rule::sub);
    }

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
