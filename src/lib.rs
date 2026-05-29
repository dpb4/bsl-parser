use std::collections::HashMap;
pub mod eval;
pub mod parse;
pub mod primitive;
use parse::*;
use primitive::*;

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
    expressions: Vec<Expression>,
    bindings: HashMap<&'a str, Expression>,
}

impl<'a> SequentialExecution<'a> {
    pub fn new_single(e: Expression) -> Self {
        Self {
            expressions: vec![e],
            bindings: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Expression {
    Literal(Primitive),
    Token(String),
    FunctionCall((FunctionName, Vec<Expression>)),
    Cond((Vec<(Expression, Expression)>, Box<Expression>)),
    // Local((SequentialExecution<'a>, Box<Expression>)),
}

impl<'a> Expression {
    fn into_owned(self) -> Self {
        match self {
            Self::Token(cs) => Self::Token(cs.to_owned()),
            s @ _ => s,
        }
    }
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
            "and" => Some(Self::And),
            "or" => Some(Self::Or),
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

// fn parse_expression_multiple(input: &str) -> Result<Vec<Expression>, &str> {
//     par::zero_plus(par::word())
//         .parse(input)?
//         .1
//         .into_iter()
//         .map(|s| parse_expression(s))
//         .collect()
// }

fn parse_literal<'a>() -> impl Parser<'a, Expression> {
    com::map(
        com::and_then(par::blob(), |b| Primitive::try_from_str(b)),
        |p| Expression::Literal(p),
    )
}

fn parse_token<'a>() -> impl Parser<'a, Expression> {
    com::map(par::token(), |t| Expression::Token(String::from(t)))
}

fn parse_fn_name<'a>() -> impl Parser<'a, FunctionName> {
    com::map(par::token(), |s| {
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
        |(name, args)| Expression::FunctionCall((name, args)),
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
            par::maybe_space_then(par::parse_a_until_b(single_case, else_case)),
        )),
        |(cases, else_case)| Expression::Cond((cases, Box::new(else_case))),
    )
}

fn parse_expression<'a>() -> impl Parser<'a, Expression> {
    BoxedParser::new(chain_or!(
        parse_literal(),
        parse_token(),
        parse_cond(),
        parse_fn_call()
    ))
}

fn parse_const_def<'a>() -> impl Parser<'a, TopLevelExpression> {
    com::map(
        par::maybe_space_then(par::paren(com::right(
            par::match_exact("define"),
            par::maybe_space_then(com::pair(
                par::token(),
                par::maybe_space_then(lazy!(parse_expression())),
            )),
        ))),
        |(name, value)| TopLevelExpression::ConstantDefinition((name.into(), value)),
    )
}

fn parse_fn_def<'a>() -> impl Parser<'a, TopLevelExpression> {
    com::and_then(
        par::maybe_space_then(par::paren(com::right(
            par::match_exact("define"),
            par::maybe_space_then(com::pair(
                par::paren(com::pair(
                    par::token(),
                    par::one_plus(par::maybe_space_then(par::token())),
                )),
                par::maybe_space_then(lazy!(parse_expression())),
            )),
        ))),
        |((name, params), body)| {
            let (_, fn_name) = parse_fn_name().parse(name)?;
            let params = params.iter().map(|s| (*s).into()).collect();
            if let f @ FunctionName::Custom(_) = fn_name {
                Ok(TopLevelExpression::FunctionDefinition((f, params, body)))
            } else {
                Err("name clashes with a built-in keyword")
            }
        },
    )
}

fn parse_nv_expression<'a>() -> impl Parser<'a, TopLevelExpression> {
    com::map(parse_expression(), |e| {
        TopLevelExpression::NonVoidExpression(e)
    })
}

fn parse_top_level_expression<'a>() -> impl Parser<'a, TopLevelExpression> {
    chain_or!(parse_const_def(), parse_fn_def(), parse_nv_expression())
}

// fn parse_expression(input: &str) -> Result<Expression, &str> {
//     let p = if is_parend(input) {
//         // TODO should this be a word or a token?
//         par::paren(par::word()).parse(input)
//     } else {
//         par::word().parse(input)
//     };
//     // TODO check for multiple nested parenthesis
//     match p {
//         Ok((rest, first)) => {
//             if rest.is_empty() {
//                 if let Ok(p) = Primitive::try_from_str(first) {
//                     Ok(Expression::Immediate(p))
//                 } else {
//                     Ok(Expression::Token(first))
//                 }
//             } else if let Some(key) = Keyword::get_keyword(first) {
//                 match key {
//                     Keyword::Define => {
//                         let (_, (def, body)) =
//                             par::maybe_space_then(com::pair(par::word(), par::word()))
//                                 .parse(rest)?;
//                         let (_, (name, params)) =
//                             par::paren(com::pair(par::token(), par::space_separated(par::token())))
//                                 .parse(def)?;
//                         Ok(Expression::FunctionDefinition((
//                             name,
//                             params,
//                             Box::new(parse_expression(body)?),
//                         )))
//                     }
//                     Keyword::Cond => {
//                         let single_case = par::maybe_space_then(par::bracket(com::pair(
//                             par::word(),
//                             par::maybe_space_then(par::word()),
//                         )));
//                         let else_case = par::maybe_space_then(par::bracket(com::right(
//                             par::match_exact("else"),
//                             par::maybe_space_then(par::word()),
//                         )));
//                         let (_, (cases, else_case)) =
//                             dbg!(par::parse_a_until_b(single_case, else_case).parse(rest)?);
//                         let parsed_cases = cases
//                             .iter()
//                             .map(|(p, r)| {
//                                 parse_expression(*p)
//                                     .and_then(|e1| parse_expression(*r).and_then(|e2| Ok((e1, e2))))
//                             })
//                             .collect::<Result<Vec<_>, _>>()?;
//                         let else_answer = parse_expression(else_case)?;
//
//                         Ok(Expression::Cond((parsed_cases, Box::new(else_answer))))
//                     }
//                     Keyword::If => todo!(),
//                     Keyword::Local => todo!(),
//                     Keyword::Equals => todo!(),
//                     Keyword::Plus => todo!(),
//                     Keyword::Minus => todo!(),
//                     Keyword::Times => todo!(),
//                     Keyword::Divide => todo!(),
//                     Keyword::List => todo!(),
//                     Keyword::Cons => todo!(),
//                     Keyword::CheckExpect => todo!(),
//                     Keyword::First => todo!(),
//                     Keyword::Rest => todo!(),
//                 }
//             } else if let Ok(res) = parse_expression_multiple(rest) {
//                 Ok(Expression::FunctionCall((FunctionName::Custom(first), res)))
//             } else {
//                 dbg!(Err(rest))
//             }
//         }
//         Err(e) => dbg!(Err(e)),
//     }
// }

pub enum ParseErrorType {
    ArgumentCount(u8, u8, &'static str),
}

pub fn eval_sequential(se: SequentialExecution) -> Result<(), ()> {
    todo!()
}
pub fn testing() {
    let prog = r#"
(define myconst 321)
(define (foo a b c)
        (cond [(check1 a) a]
              [(check2 b) b]
              [else
                (zoo a c b)]))
"#;
    // let u = "(cond [asd qwd] [(foij as) dfwe] [awd qwd dsa] [else 123])";
    let _ = dbg!(parse_top_level_expression().parse(prog));
    let _ = dbg!(parse_top_level_expression().parse("\n(define myconst 321)\n"));
}
