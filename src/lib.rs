pub mod eval;
pub mod parse;
pub mod primitive;
use parse::*;
use primitive::*;

// use crate::eval::eval_nv_expression;

macro_rules! chain_or {
    ($a:expr) => {
        $a
    };

    ($a:expr, $b:expr $(, $rest:expr)*) => {
        chain_or!(@acc com::or($a, $b) $(, $rest)*)
    };

    (@acc $acc:expr) => {
        $acc
    };

    (@acc $acc:expr, $next:expr $(, $rest:expr)*) => {
        chain_or!(@acc com::or($acc, $next) $(, $rest)*)
    };
}

macro_rules! lazy {
    ($a:expr) => {
        |s| $a.parse(s)
    };
}

#[derive(Debug, Clone)]
pub enum Expression {
    Literal(Primitive),
    Identifier(String),
    FunctionCall((FunctionName, Vec<Expression>)),
    Cond((Vec<(Expression, Expression)>, Box<Expression>)),
    // Local((SequentialExecution<'a>, Box<Expression>)),
}

#[derive(Debug, Clone)]
pub enum FunctionName {
    BuiltIn(Keyword),
    Custom(String),
}

#[derive(Debug)]
pub enum TopLevelExpression {
    ConstantDefinition((String, Expression)),
    FunctionDefinition((FunctionName, Vec<String>, Expression)),
    NonVoidExpression(Expression),
}

// TODO:
// mod
// substring
// length

#[derive(Debug, Clone)]
pub enum Keyword {
    Define,
    If,
    Cond,
    Local,
    Equals,
    Plus,
    Minus,
    Times,
    Divide,
    And,
    Or,
    Not,
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
            "+" | "add" => Some(Self::Plus),
            "-" | "sub" => Some(Self::Minus),
            "*" | "mult" => Some(Self::Times),
            "/" | "div" => Some(Self::Divide),
            "and" => Some(Self::And),
            "or" => Some(Self::Or),
            "not" => Some(Self::Not),
            "list" => Some(Self::List),
            "cons" => Some(Self::Cons),
            "check-expect" => Some(Self::CheckExpect),
            "first" => Some(Self::First),
            "rest" => Some(Self::Rest),
            _ => None,
        }
    }
}

fn parse_literal<'a>() -> impl Parser<'a, Expression> {
    com::map(
        com::apply(
            par::maybe_space_then(chain_or!(
                par::string_literal(),
                par::number_literal(),
                par::identifier()
            )),
            |b| Primitive::try_from_str(b).ok_or("unable to parse primitive"),
        ),
        Expression::Literal,
    )
}

fn parse_token<'a>() -> impl Parser<'a, Expression> {
    com::map(par::identifier(), |t| {
        Expression::Identifier(String::from(t))
    })
}

fn parse_fn_name<'a>() -> impl Parser<'a, FunctionName> {
    com::map(com::or(par::identifier(), par::operator()), |s| {
        if let Some(k) = Keyword::get_keyword(s) {
            FunctionName::BuiltIn(k)
        } else {
            FunctionName::Custom(String::from(s))
        }
    })
}

fn parse_fn_call<'a>() -> impl Parser<'a, Expression> {
    com::map(
        par::maybe_space_then(par::paren(com::pair(
            parse_fn_name(),
            par::space_separated(lazy!(parse_expression())),
        ))),
        Expression::FunctionCall,
    )
}

fn parse_cond<'a>() -> impl Parser<'a, Expression> {
    let single_case = par::maybe_space_then(par::bracket(com::pair(
        lazy!(parse_expression()),
        par::maybe_space_then(lazy!(parse_expression())),
    )));
    let else_case = par::maybe_space_then(par::bracket(com::right(
        par::match_exact("else"), // TODO add space after else
        par::maybe_space_then(lazy!(parse_expression())),
    )));
    com::map(
        par::paren(com::right(
            par::match_exact("cond"), // TODO add space after cond
            par::maybe_space_then(com::parse_a_until_b(single_case, else_case)),
        )),
        |(cases, else_case)| Expression::Cond((cases, Box::new(else_case))),
    )
}

fn parse_expression<'a>() -> impl Parser<'a, Expression> {
    chain_or!(
        parse_literal(),
        parse_token(),
        parse_cond(),
        parse_fn_call()
    )
}

fn parse_const_def<'a>() -> impl Parser<'a, TopLevelExpression> {
    com::map(
        par::maybe_space_then(par::paren(com::right(
            par::match_exact("define"),
            par::maybe_space_then(com::pair(
                par::identifier(),
                par::maybe_space_then(lazy!(parse_expression())),
            )),
        ))),
        |(name, value)| TopLevelExpression::ConstantDefinition((name.into(), value)),
    )
}

fn parse_fn_def<'a>() -> impl Parser<'a, TopLevelExpression> {
    com::apply(
        par::maybe_space_then(par::paren(com::right(
            par::match_exact("define"),
            par::maybe_space_then(com::pair(
                par::paren(com::pair(
                    |c| parse_fn_name().parse(c),
                    com::one_plus(par::maybe_space_then(par::identifier())),
                )),
                par::maybe_space_then(lazy!(parse_expression())),
            )),
        ))),
        |((name, params), body)| {
            let params = params.iter().map(|s| (*s).into()).collect();
            if let f @ FunctionName::Custom(_) = name {
                Ok(TopLevelExpression::FunctionDefinition((f, params, body)))
            } else {
                Err("name clashes with a built-in keyword")
            }
        },
    )
}

pub fn parse_nv_expression<'a>() -> impl Parser<'a, TopLevelExpression> {
    com::map(parse_expression(), |e| {
        TopLevelExpression::NonVoidExpression(e)
    })
}

pub fn parse_top_level_expression<'a>() -> impl Parser<'a, TopLevelExpression> {
    chain_or!(parse_const_def(), parse_fn_def(), parse_nv_expression())
}

pub enum ParseErrorType {
    ArgumentCount(u8, u8, &'static str),
}

pub fn testing() {
    let _ = dbg!(par::string_literal().parse("\"abcd\"".into()));
    let _ = dbg!(parse_expression().parse("\"abcd\"".into()));
}
