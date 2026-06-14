#![feature(try_blocks)]

pub mod eval;
pub mod parse;
pub mod primitive;
use parse::*;
use primitive::*;

// TODO goals:
// allow for comments
// add locals
// improve primitive api? seems messy rn

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
    Local((Vec<TopLevelExpression>, Box<Expression>)),
}

// TODO replace FunctionName::Custom with this type
#[derive(Debug, Clone)]
pub struct ValidIdentifier(String);

#[derive(Debug, Clone)]
pub enum FunctionName {
    BuiltIn(Keyword),
    Custom(String),
}

#[derive(Debug, Clone)]
pub enum TopLevelExpression {
    ConstantDefinition((String, Expression)),
    FunctionDefinition((FunctionName, Vec<String>, Expression)),
    NonVoidExpression(Expression),
    StructDefinition((FunctionName, Vec<FunctionName>)),
}

// TODO:
// mod
// substring
// length
// define-struct
// check-expect

// TODO make a lazy-static hashmap for this
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
    Mod,
    DefineStruct,
    Length,
    Substring,
    PredZero,
    PredNatural,
    PredInteger,
    PredNumber,
    PredString,
    PredEmpty,
    PredCons,
    PredList,
    PredLambda,
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
            "mod" => Some(Self::Mod),
            "define-struct" => Some(Self::DefineStruct),
            "length" => Some(Self::Length),
            "substring" => Some(Self::Substring),
            "zero?" => Some(Self::PredZero),
            "natural?" => Some(Self::PredNatural),
            "integer?" => Some(Self::PredInteger),
            "number?" => Some(Self::PredNumber),
            "string?" => Some(Self::PredString),
            "empty?" => Some(Self::PredEmpty),
            "cons?" => Some(Self::PredCons),
            "list?" => Some(Self::PredList),
            "lambda?" => Some(Self::PredLambda),
            _ => None,
        }
    }
}

fn parse_literal<'a>() -> impl Parser<'a, Expression> {
    let possible_literals = par::string_literal()
        .or(par::number_literal())
        .or(par::identifier());

    par::maybe_space_then(
        possible_literals
            .apply(|b| Primitive::try_from_str(b).ok_or("unable to parse primitive"))
            .map(Expression::Literal),
    )
}

fn parse_token<'a>() -> impl Parser<'a, Expression> {
    par::identifier().map(|t| Expression::Identifier(String::from(t)))
}

fn parse_fn_name<'a>() -> impl Parser<'a, FunctionName> {
    par::identifier().or(par::operator()).map(|s| {
        if let Some(k) = Keyword::get_keyword(s) {
            FunctionName::BuiltIn(k)
        } else {
            FunctionName::Custom(String::from(s))
        }
    })
}

fn parse_fn_call<'a>() -> impl Parser<'a, Expression> {
    par::maybe_space_then(
        par::paren(parse_fn_name().then(par::space_separated(lazy!(parse_expression()))))
            .map(Expression::FunctionCall),
    )
}

fn parse_cond<'a>() -> impl Parser<'a, Expression> {
    let single_case = par::maybe_space_then(par::bracket(
        lazy!(parse_expression())
            .then_ignore(par::optional_space())
            .then(lazy!(parse_expression())),
    ));

    let else_case = par::maybe_space_then(par::bracket(
        par::match_exact("else")
            .ignore_then(par::optional_space())
            .ignore_then(lazy!(parse_expression())),
    ));

    par::paren(
        par::match_exact("cond")
            .ignore_then(par::optional_space())
            .ignore_then(com::parse_a_until_b(single_case, else_case)),
    )
    .map(|(cases, else_case)| Expression::Cond((cases, Box::new(else_case))))
}

fn parse_local<'a>() -> impl Parser<'a, Expression> {
    par::paren(
        par::match_exact("local")
            .ignore_then(par::optional_space())
            .ignore_then(lazy!(par::bracket(com::zero_plus(
                parse_top_level_expression()
            ))
            .then_ignore(par::optional_space())
            .then(parse_expression()))),
    )
    .map(|(local_defns, body)| Expression::Local((local_defns, Box::new(body))))
}
fn parse_expression<'a>() -> impl Parser<'a, Expression> {
    parse_literal()
        .or(parse_token())
        .or(parse_cond())
        .or(parse_fn_call())
    // .or(parse_local())
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
    parse_const_def()
        .or(parse_fn_def())
        .or(parse_nv_expression())
}

// pub fn testing() {
//     //     let _ = dbg!(parse_fn_def().parse(
//     //         r"
//     // (define (fib n)
//     //     (cond [(= n 0) 1]
//     //           [(= n 1) 1]
//     //           [else
//     //             (+ (fib (- n 1)) (fib (- n 2)))]))"
//     //             .into()
//     //     ));
//     let s = r"(cond [(= n 0) 1]
//     [(= n 1) 1]
//     [else
//       (+ (fib (- n 1)) (fib (- n 2)))])";
//     let single_case = par::maybe_space_then(par::bracket(
//         lazy!(parse_expression())
//             .then_ignore(par::optional_space())
//             .then(lazy!(parse_expression())),
//     ));

//     let else_case = par::maybe_space_then(par::bracket(
//         par::match_exact("else")
//             .ignore_then(par::optional_space())
//             .ignore_then(lazy!(parse_expression())),
//     ));

//     // let _ = dbg!(par::paren(
//     //     par::match_exact("cond").ignore_then(par::optional_space()) //         .ignore_then(com::parse_a_until_b(single_case, else_case)),
//     // )
//     // .parse(s.into()));
//     // let _ = dbg!(
//     //     par::maybe_space_then(com::parse_a_until_b(single_case, else_case))
//     //         .parse(" [(= n 1) 1]\n    [(= n 0) 1]\n    [else\n      3]".into())
//     // );
//     let _ = dbg!(parse_cond().parse(s.into()));
// }
#[cfg(test)]
mod tests {
    use crate::*;
    use core::assert_matches;

    fn expr_lit_empty(s: &str) {
        assert_matches!(
            parse_literal().parse(s.into()),
            Ok((_, Expression::Literal(Primitive::List(ConsList::Empty))))
        );
    }

    #[test]
    fn primitive_empty() {
        // let s = "empty";
        // assert_matches!(
        //     parse_literal().parse(s.into()),
        //     Ok((_, Expression::Literal(Primitive::List(ConsList::Empty))))
        // );
        // let s = " empty";
        // assert_matches!(
        //     parse_literal().parse(s.into()),
        //     Ok((_, Expression::Literal(Primitive::List(ConsList::Empty))))
        // );
        // let s = " empty ";
        // assert_matches!(
        //     parse_literal().parse(s.into()),
        //     Ok((_, Expression::Literal(Primitive::List(ConsList::Empty))))
        // );
        expr_lit_empty("empty");
        expr_lit_empty(" empty");
        expr_lit_empty(" empty ");
    }
}
