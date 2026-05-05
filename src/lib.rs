use std::collections::HashMap;
pub mod parse;
pub mod primitive;
use parse::*;
use primitive::*;

#[derive(Debug, Clone)]
pub struct ParseError<'a> {
    // TODO
    msg: &'static str,
    token: &'a str,
}

#[derive(Debug, Clone)]
pub struct EvalError<'a> {
    // TODO
    msg: &'static str,
    token: &'a str,
}

pub enum GenError<'a> {
    Parsing(ParseError<'a>),
    Evaluation(EvalError<'a>),
}

#[derive(Debug)]
pub struct SequentialExecution<'a> {
    expressions: Vec<Expression<'a>>,
    bindings: HashMap<&'a str, Expression<'a>>,
}

impl<'a> SequentialExecution<'a> {
    pub fn new_single(e: Expression<'a>) -> Self {
        Self {
            expressions: vec![e],
            bindings: HashMap::new(),
        }
    }
}

#[derive(Debug)]
pub enum Expression<'a> {
    Immediate(Primitive),
    Token(&'a str),
    ConstantDefinition(&'a str, Box<Expression<'a>>),
    FunctionDefinition((Vec<&'a str>, Box<Expression<'a>>)),
    FunctionCall(&'a str, Vec<Expression<'a>>),
    Local(SequentialExecution<'a>, Box<Expression<'a>>),
}

#[derive(Debug, Clone)]
enum Keyword {
    Define,
    If,
    Cond,
    Local,
    Equals,
    Plus,
    Minus,
    Times,
    Divide,
    List,
    Cons,
    CheckExpect,
    First,
    Rest,
}

impl Keyword {
    fn get_keyword(s: &str) -> Option<Self> {
        match &s.to_lowercase()[..] {
            "define" => Some(Self::Define),
            "if" => Some(Self::If),
            "cond" => Some(Self::Cond),
            "local" => Some(Self::Local),
            "=" => Some(Self::Equals),
            "+" => Some(Self::Plus),
            "-" => Some(Self::Minus),
            "*" => Some(Self::Times),
            "/" => Some(Self::Divide),
            "list" => Some(Self::List),
            "cons" => Some(Self::Cons),
            "check-expect" => Some(Self::CheckExpect),
            "first" => Some(Self::First),
            "rest" => Some(Self::Rest),
            _ => None,
        }
    }
}

fn is_parend(s: &str) -> bool {
    s.starts_with("(") && s.ends_with(")")
}

fn is_opening(c: char) -> bool {
    matches!(c, '(' | '[' | '\'' | '"')
}

fn is_closing(c: char) -> bool {
    matches!(c, ')' | ']' | '\'' | '"')
}

fn closes(o: char, c: char) -> bool {
    if is_opening(o) {
        closing(o) == c
    } else {
        false
    }
}
fn closing(c: char) -> char {
    match c {
        '(' => ')',
        '[' => ']',
        '\'' => '\'',
        '"' => '"',
        _ => panic!("that is not a matchable symbol"),
    }
}

// parse the next word or paired token string

fn parse_expression_multiple(input: &str) -> Result<Vec<Expression>, &str> {
    par::zero_plus(par::word())
        .parse(input)?
        .1
        .into_iter()
        .map(|s| parse_expression(s))
        .collect()
}

fn parse_expression(input: &str) -> Result<Expression, &str> {
    let p = if is_parend(input) {
        // TODO should this be a word or a token?
        par::paren(par::word()).parse(input)
    } else {
        par::word().parse(input)
    };
    // TODO check for multiple nested parenthesis
    match p {
        Ok((rest, first)) => {
            if rest.is_empty() {
                if let Ok(p) = Primitive::try_from_str(first) {
                    Ok(Expression::Immediate(p))
                } else {
                    Ok(Expression::Token(first))
                }
            } else if let Some(key) = Keyword::get_keyword(first) {
                match key {
                    Keyword::Define => {
                        let (body, params) = com::right(
                            par::optional_space(),
                            com::pair(
                                par::word(),
                                par::paren(par::one_plus(com::right(
                                    par::optional_space(),
                                    par::token(),
                                ))),
                            ),
                        )
                        .parse(rest)
                        .map(|p| p.1)?;
                        // .parse(par::word().parse(rest).map(|t| t.1)?)?;
                        Ok(Expression::FunctionDefinition((
                            params,
                            Box::new(parse_expression(body)?),
                        )))
                    }
                    Keyword::If => todo!(),
                    Keyword::Cond => todo!(),
                    Keyword::Local => todo!(),
                    Keyword::Equals => todo!(),
                    Keyword::Plus => todo!(),
                    Keyword::Minus => todo!(),
                    Keyword::Times => todo!(),
                    Keyword::Divide => todo!(),
                    Keyword::List => todo!(),
                    Keyword::Cons => todo!(),
                    Keyword::CheckExpect => todo!(),
                    Keyword::First => todo!(),
                    Keyword::Rest => todo!(),
                }
            } else if let Ok(res) = parse_expression_multiple(rest) {
                Ok(Expression::FunctionCall(first, res))
            } else {
                Err(rest)
            }
        }
        Err(e) => Err(e),
    }
}

pub enum ParseErrorType {
    ArgumentCount(u8, u8, &'static str),
}

pub fn eval_sequential(se: SequentialExecution) -> Result<(), ()> {
    todo!()
}
pub fn testing() {
    let u = "(define (foo a b) (+ 2 3))";
    // let u = "(add (foo a) (+ 2 3) var)";
    let _ = dbg!(parse_expression(u));
    // let _ = dbg!(par::token().parse("abc) wijd"));
}
