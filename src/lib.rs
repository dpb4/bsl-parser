#![feature(try_blocks)]

pub mod eval;
pub mod parse;
pub mod primitive;
use parse::*;
use phf::phf_map;
use primitive::*;
// TODO goals:
// allow for comments
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
// define-struct
// check-expect

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

static KEYWORDS: phf::Map<&'static str, Keyword> = phf_map! {
    "define" => Keyword::Define,
    "if" => Keyword::If,
    "cond" => Keyword::Cond,
    "local" => Keyword::Local,
    "=" => Keyword::Equals,
    "+" | "add" => Keyword::Plus,
    "-" | "sub" => Keyword::Minus,
    "*" | "mult" => Keyword::Times,
    "/" | "div" => Keyword::Divide,
    "and" => Keyword::And,
    "or" => Keyword::Or,
    "not" => Keyword::Not,
    "list" => Keyword::List,
    "cons" => Keyword::Cons,
    "check-expect" => Keyword::CheckExpect,
    "first" => Keyword::First,
    "rest" => Keyword::Rest,
    "mod" => Keyword::Mod,
    "define-struct" => Keyword::DefineStruct,
    "length" => Keyword::Length,
    "substring" => Keyword::Substring,
    "zero?" => Keyword::PredZero,
    "natural?" => Keyword::PredNatural,
    "integer?" => Keyword::PredInteger,
    "number?" => Keyword::PredNumber,
    "string?" => Keyword::PredString,
    "empty?" => Keyword::PredEmpty,
    "cons?" => Keyword::PredCons,
    "list?" => Keyword::PredList,
    "lambda?" => Keyword::PredLambda,
};

impl Keyword {
    fn get_keyword(s: &str) -> Option<Self> {
        KEYWORDS.get(&s.to_lowercase()).cloned()
    }
}

fn parse_literal<'a>() -> impl Parser<'a, Expression> {
    // TODO this will mess with error messages
    let possible_literals = par::string_literal()
        .or(par::number_literal())
        .or(par::identifier());

    par::maybe_space_then(
        possible_literals
            .apply(|b| {
                Primitive::try_from_str(b).ok_or(format!("unable to create primitive from `{b}`"))
            })
            .map(Expression::Literal),
    )
}

fn parse_token<'a>() -> impl Parser<'a, Expression> {
    par::identifier().map_with_context(
        |t| Expression::Identifier(String::from(t)),
        "at parse_token",
    )
}

fn parse_fn_name<'a>() -> impl Parser<'a, FunctionName> {
    par::identifier().or(par::operator()).map_with_context(
        |s| {
            if let Some(k) = Keyword::get_keyword(s) {
                FunctionName::BuiltIn(k)
            } else {
                FunctionName::Custom(String::from(s))
            }
        },
        "at parse_fn_name",
    )
}

fn parse_fn_call<'a>() -> impl Parser<'a, Expression> {
    par::maybe_space_then(
        par::paren(parse_fn_name().then(par::space_separated(lazy!(parse_expression()))))
            .map_with_context(Expression::FunctionCall, "at parse_fn_call"),
    )
}

fn parse_cond<'a>() -> impl Parser<'a, Expression> {
    let single_case = par::maybe_space_then(par::bracket(
        lazy!(parse_expression())
            .then_ignore(par::optional_space())
            .then(lazy!(parse_expression())),
    ))
    .map_with_context(|x| x, "at parse_cond/single_case");

    let else_case = par::maybe_space_then(par::bracket(
        par::match_exact("else")
            .ignore_then(par::optional_space())
            .ignore_then(lazy!(parse_expression())),
    ))
    .map_with_context(|x| x, "at parse_cond/else_case");

    par::optional_space().ignore_then(
        par::paren(
            par::match_exact("cond")
                .ignore_then(par::optional_space())
                .ignore_then(com::parse_a_until_b(single_case, else_case)),
        )
        .map_with_context(
            |(cases, else_case)| Expression::Cond((cases, Box::new(else_case))),
            "at parse_cond",
        ),
    )
    // par::optional_space()
    //     .ignore_then(com::parse_a_until_b(single_case, else_case))
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
    .map_with_context(
        |(local_defns, body)| Expression::Local((local_defns, Box::new(body))),
        "at parse_local",
    )
}

#[inline]
fn is_terminator(c: char) -> bool {
    c.is_whitespace() || c == ')' || c == ']' || c == ';'
}

fn looks_like_primitive(s: &str) -> bool {
    let firstchar = {
        let c = s.chars().next();
        if c.is_some() {
            let c = c.unwrap();
            c.is_numeric() || c == '\'' || c == '"' || c == '-'
        } else {
            false
        }
    };

    firstchar || s.starts_with("empty") && s.chars().nth(5).map_or(true, is_terminator)
}

fn parse_expression<'a>() -> impl Parser<'a, Expression> {
    let paren_parsers = com::join(
        com::peek(par::match_exact("(").ignore_then(par::identifier().or(par::operator())))
            .map_with_context(
                |s| -> BoxedParser<'a, Expression> {
                    match s {
                        "cond" => Box::new(parse_cond()),
                        "local" => Box::new(parse_local()),
                        _ => Box::new(parse_fn_call()),
                    }
                },
                "at parse_expression/paren_parsers",
            ),
    );
    let empty_parser = par::identifier().pred(|i| *i == "empty").map_with_context(
        |_| Expression::Literal(Primitive::List(ConsList::Empty)),
        "at parse_expression/empty_parser",
    );

    // let primitive_parser = com::peek(|ctx| par::parse_any_char(ctx))
    //     .pred(|c| c.is_numeric() || c == '\'' || c == '"' || c == '-');

    // I have to split these parsers like this to give specific error messages;
    // if the function call parser fails, was it a malformed function call or
    // just not a function call to begin with (ie a number, for example)?
    let function_call_like = com::peek((|c| par::parse_any_char(c)).pred(|c| *c == '('))
        .ignore_then(par::unrecoverable(paren_parsers))
        .map_with_context(|x| x, "at parse_expression/function_call_like");

    let literal_like = com::peek(
        (|c| par::parse_any_char(c))
            .pred(|c| *c == '\'' || *c == '"' || *c == '-' || c.is_ascii_digit()),
    )
    .ignore_then(par::unrecoverable(parse_literal()))
    .map_with_context(|x| x, "at parse_expression/literal_like");

    let token_like = par::unrecoverable(empty_parser.or(parse_token()))
        .map_with_context(|x| x, "at parse_expression/token_like");

    par::optional_space()
        .ignore_then(function_call_like.or(literal_like).or(token_like))
        .map_with_context(|x| x, "at parse_expression")

    // parse_literal().or(parse_token()).or(paren_parsers)
    // .or(parse_cond())
    // .or(parse_local())
    // .or(parse_fn_call())
}

fn parse_const_def<'a>() -> impl Parser<'a, TopLevelExpression> {
    com::map_with_context(
        par::maybe_space_then(par::paren(com::right(
            par::match_exact("define"),
            par::maybe_space_then(com::pair(
                par::identifier(),
                par::maybe_space_then(lazy!(parse_expression())),
            )),
        ))),
        |(name, value)| TopLevelExpression::ConstantDefinition((name.into(), value)),
        "at parse_const_def",
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
    com::map_with_context(
        parse_expression(),
        |e| TopLevelExpression::NonVoidExpression(e),
        "at parse_nv_expression",
    )
}

pub fn parse_top_level_expression<'a>() -> impl Parser<'a, TopLevelExpression> {
    let define_parsers = par::paren(par::identifier().map(
        |s| -> BoxedParser<'a, TopLevelExpression> {
            match s {
                "define" => todo!(),
                "define-struct" => todo!(),
                _ => todo!(),
            }
        },
    ));
    parse_const_def()
        .or(parse_fn_def())
        .or(parse_nv_expression())
        .map_with_context(|x| x, "at parse_top_level_expression")
}

pub fn testing() {
    //     let _ = dbg!(parse_fn_def().parse(
    //         r"
    // (define (fib n)
    //     (cond [(= n 0) 1]
    //           [(= n 1) 1]
    //           [else
    //             (+ (fib (- n 1)) (fib (- n 2)))]))"
    //             .into()
    //     ));
    let s = r"(define (map fn list)
  (cond [(empty? list) empty] ;hi 
        [else
          (cons (fn (first list))
                (map fn (rest list)))]))";

    let _ = dbg!(parse_top_level_expression().parse(s.into()));
}
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
