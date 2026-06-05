pub type ParseResult<'a, Output> = Result<(ParseContext<'a>, Output), ParseError<'a>>;

#[derive(Debug, Clone)]
pub struct ParseContext<'a> {
    remaining: &'a str,
    current_index: usize,
    line: usize,
    col: usize,
}

impl<'a> From<&'a str> for ParseContext<'a> {
    fn from(value: &'a str) -> Self {
        Self {
            remaining: value,
            current_index: 0,
            line: 0,
            col: 0,
        }
    }
}

impl<'a> ParseContext<'a> {
    // pub fn inc_index(mut self, n: usize) -> Self {
    //     self.current_index += n;
    //     self.remaining = &self.remaining[n..];
    //     self
    // }

    pub fn produce(mut self, n: usize) -> (Self, &'a str) {
        let (new, remaining) = self.remaining.split_at(n);
        self.remaining = remaining;
        self.current_index += n;
        (self, new)
    }

    // pub fn get_current(self) ->
}

impl<'a> std::fmt::Display for ParseContext<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}\nindex: {}, line: {}, col: {}",
            self.remaining, self.current_index, self.line, self.col
        )
    }
}

#[derive(Debug, Clone)]
pub struct ParseError<'a> {
    ctx: ParseContext<'a>,
    msg: String,
}
impl<'a> std::fmt::Display for ParseError<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "parse error at:\n{}\nerror msg: {}", self.ctx, self.msg)
    }
}

impl<'a> ParseError<'a> {
    pub fn from_ctx<S: Into<String>>(ctx: ParseContext<'a>, msg: S) -> Self {
        Self {
            ctx,
            msg: msg.into(),
        }
    }
    pub fn append_msg<S: Into<String>>(self, new_line: S) -> Self {
        Self {
            ctx: self.ctx,
            msg: self.msg + &new_line.into(),
        }
    }
}
pub trait Parser<'a, Output> {
    fn parse(&self, ctx: ParseContext<'a>) -> ParseResult<'a, Output>;

    // fn map<'a, P, F, B>(&self, map_fn: F) -> impl Parser<B>
    // where
    //     P: Parser<A>,
    //     F: Fn(Output) -> B,
    // {
    //     move |input.into()| {
    //         self.parse(input)
    //             .map(|(next_ctx, result)| (next_ctx, map_fn(result)))
    //     }
    // }
}

impl<'a, F, Output> Parser<'a, Output> for F
where
    F: Fn(ParseContext<'a>) -> ParseResult<'a, Output>,
{
    fn parse(&self, ctx: ParseContext<'a>) -> ParseResult<'a, Output> {
        self(ctx)
    }
}

// pub struct BoxedParser<Output> {
//     parser: Box<dyn Parser<Output>>,
// }

// impl<Output> BoxedParser<Output> {
//     pub fn new<P>(parser: P) -> Self
//     where
//         P: Parser<Output>,
//     {
//         BoxedParser {
//             parser: Box::new(parser),
//         }
//     }
// }

// impl<'a, Output> Parser<Output> for BoxedParser<Output> {
//     fn parse(&self, ctx: ParseContext) -> ParseResult<Output> {
//         self.parser.parse(ctx)
//     }
// }

pub mod com {
    use crate::parse::{ParseContext, ParseError};

    use super::Parser;

    pub fn pair<'a, P1, P2, R1, R2>(parser1: P1, parser2: P2) -> impl Parser<'a, (R1, R2)>
    where
        P1: Parser<'a, R1>,
        P2: Parser<'a, R2>,
    {
        move |ctx| {
            parser1.parse(ctx).and_then(|(next_ctx, result1)| {
                parser2
                    .parse(next_ctx)
                    .map(|(last_input, result2)| (last_input, (result1, result2)))
            })
        }
    }

    pub fn map<'a, P, F, A, B>(parser: P, map_fn: F) -> impl Parser<'a, B>
    where
        P: Parser<'a, A>,
        F: Fn(A) -> B,
    {
        move |ctx| {
            parser
                .parse(ctx)
                .map(|(next_ctx, result)| (next_ctx, map_fn(result)))
        }
    }

    pub fn apply<'a, P, F, A, B, S>(parser: P, try_fn: F) -> impl Parser<'a, B>
    where
        S: Into<String>,
        P: Parser<'a, A>,
        F: Fn(A) -> Result<B, S>,
    {
        move |ctx| {
            parser
                .parse(ctx)
                // .and_then(|(next_ctx, val)| map_fn(val).map(|val2| (next_ctx, val2)))
                .and_then(|(next_ctx, val)| match try_fn(val) {
                    Ok(val_b) => Ok((next_ctx, val_b)),
                    Err(err_msg) => Err(ParseError::from_ctx(next_ctx, err_msg)),
                })
        }
    }
    pub fn left<'a, P1, P2, R1, R2>(parser1: P1, parser2: P2) -> impl Parser<'a, R1>
    where
        P1: Parser<'a, R1>,
        P2: Parser<'a, R2>,
    {
        map(pair(parser1, parser2), |(left, _)| left)
    }

    pub fn right<'a, P1, P2, R1, R2>(parser1: P1, parser2: P2) -> impl Parser<'a, R2>
    where
        P1: Parser<'a, R1>,
        P2: Parser<'a, R2>,
    {
        map(pair(parser1, parser2), |(_, right)| right)
    }

    pub fn or<'a, P1, P2, R>(parser1: P1, parser2: P2) -> impl Parser<'a, R>
    where
        P1: Parser<'a, R>,
        P2: Parser<'a, R>,
    {
        move |ctx: ParseContext<'a>| match parser1.parse(ctx.clone()) {
            r @ Ok(_) => r,
            Err(_) => parser2.parse(ctx),
        }
    }
    pub fn pred<'a, P, A, F>(parser: P, predicate: F) -> impl Parser<'a, A>
    where
        P: Parser<'a, A>,
        F: Fn(&A) -> bool,
    {
        move |ctx: ParseContext<'a>| match parser.parse(ctx.clone()) {
            Ok((next_ctx, val)) => {
                if predicate(&val) {
                    Ok((next_ctx, val))
                } else {
                    Err(ParseError::from_ctx(ctx, "predicate failed"))
                }
            }
            e @ _ => e,
        }
    }
    pub fn lazy<'a, P, A>(parser: P) -> impl Parser<'a, A>
    where
        P: Parser<'a, A>,
    {
        move |s| parser.parse(s)
    }
    pub fn one_plus<'a, P, A>(parser: P) -> impl Parser<'a, Vec<A>>
    where
        P: Parser<'a, A>,
    {
        move |ctx| {
            let mut result = Vec::new();
            let mut cur_ctx;
            match parser.parse(ctx) {
                Ok((next_ctx, first_item)) => {
                    cur_ctx = next_ctx;
                    result.push(first_item);
                }
                Err(pe) => {
                    return Err(pe.append_msg("failed at first attempt in one_plus"));
                }
            };

            loop {
                match parser.parse(cur_ctx) {
                    Ok((next_ctx, next_item)) => {
                        cur_ctx = next_ctx;
                        result.push(next_item);
                    }
                    Err(pe) => {
                        cur_ctx = pe.ctx;
                        break;
                    }
                }
            }

            Ok((cur_ctx, result))
        }
    }

    pub fn zero_plus<'a, P, A>(parser: P) -> impl Parser<'a, Vec<A>>
    where
        P: Parser<'a, A>,
    {
        move |ctx| {
            let mut result = Vec::new();
            let mut cur_ctx = ctx;

            loop {
                match parser.parse(cur_ctx) {
                    Ok((next_ctx, next_item)) => {
                        cur_ctx = next_ctx;
                        result.push(next_item);
                    }
                    Err(pe) => {
                        cur_ctx = pe.ctx;
                        break;
                    }
                }
            }

            Ok((cur_ctx, result))
        }
    }
    pub fn parse_a_until_b<'a, P1, P2, R1, R2>(
        multiple: P1,
        last: P2,
    ) -> impl Parser<'a, (Vec<R1>, R2)>
    where
        P1: Parser<'a, R1>,
        P2: Parser<'a, R2>,
    {
        move |ctx| {
            let mut result_vec = vec![];
            let mut next_ctx = ctx;

            loop {
                match last.parse(next_ctx) {
                    Ok((r, v)) => {
                        return Ok((r, (result_vec, v)));
                    }
                    Err(pe) => match multiple.parse(pe.ctx) {
                        Ok((r, v)) => {
                            result_vec.push(v);
                            next_ctx = r;
                        }
                        Err(e) => return Err(e),
                    },
                }
            }
        }
    }
}
pub mod par {

    use crate::parse::ParseContext;
    use crate::parse::ParseError;

    use super::com::*;
    use super::ParseResult;
    use super::Parser;

    pub fn identity<'a>() -> impl Parser<'a, ()> {
        move |ctx| Ok((ctx, ()))
    }

    pub fn match_exact<'a>(expected: &'a str) -> impl Parser<'a, &'a str> {
        move |ctx: ParseContext<'a>| {
            let matches = ctx.remaining.get(..expected.len()).map(|s| s == expected);
            if let Some(b) = matches {
                if b {
                    Ok(ctx.produce(expected.len()))
                } else {
                    let s = ctx.remaining[..expected.len()].to_string();
                    Err(ParseError::from_ctx(
                        ctx,
                        format!("'{s}' does not match exact string {expected}",),
                    ))
                }
            } else {
                let s = ctx.remaining;
                Err(ParseError::from_ctx(
                    ctx,
                    format!("'{s}' does not match exact string {expected}",),
                ))
            }
        }
    }
    // pub fn match_exact_end<'a>(expected: &'static str) -> impl Parser<'a, &'static str> {
    //     move |input: ParseContext| {
    //         if let Some(stripped) = input.strip_suffix(expected) {
    //             Ok((stripped, expected))
    //         } else {
    //             Err(input)
    //         }
    //     }
    // }

    pub fn whitespace_char<'a>() -> impl Parser<'a, char> {
        pred(parse_any_char, |c| c.is_whitespace())
    }

    pub fn paren<'a, P, A>(parser: P) -> impl Parser<'a, A>
    where
        P: Parser<'a, A>,
    {
        // right(pair(match_exact("("), match_exact_end(")")), parser)
        left(right(match_exact("("), parser), match_exact(")"))
    }

    pub fn bracket<'a, P, A>(parser: P) -> impl Parser<'a, A>
    where
        P: Parser<'a, A>,
    {
        left(right(match_exact("["), parser), match_exact("]"))
        // right(pair(match_exact("["), match_exact_end(u]u)), parser)
    }

    pub fn parse_identifier<'a>(ctx: ParseContext<'a>) -> ParseResult<'a, &'a str> {
        let mut matched = 0;

        if let Some(c) = ctx.remaining.chars().next() {
            if c.is_alphabetic() {
                matched += 1
            } else {
                return Err(ParseError::from_ctx(
                    ctx,
                    format!("first character of identifier must be alphabetical (got {c})"),
                ));
            }
        } else {
            return Err(ParseError::from_ctx(
                ctx,
                "expected identifier, given empty string",
            ));
        }

        for next in ctx.remaining[1..].chars() {
            if next.is_whitespace() || next == ')' || next == ']' {
                break;
            } else if next.is_alphanumeric() || next == '-' || next == '_' {
                matched += 1;
            } else {
                return Err(ParseError::from_ctx(
                    ctx,
                    format!("identifier must be alphanumeric with '-' or '_' (found illegal character {next})"),
                ));
            }
        }

        Ok(ctx.produce(matched))
    }

    pub fn parse_any_char<'a>(ctx: ParseContext) -> ParseResult<char> {
        match ctx.remaining.chars().next() {
            Some(next) => Ok((ctx.produce(1).0, next)),
            _ => Err(ParseError::from_ctx(
                ctx,
                "expected any char, got empty string",
            )),
        }
    }
    pub fn parse_any_char_as_str<'a>(ctx: ParseContext<'a>) -> ParseResult<'a, &'a str> {
        match ctx.remaining.chars().next() {
            Some(_) => Ok(ctx.produce(1)),
            _ => Err(ParseError::from_ctx(
                ctx,
                "expected any char, got empty string",
            )),
        }
    }

    pub fn optional_space<'a>() -> impl Parser<'a, Vec<char>> {
        zero_plus(whitespace_char())
    }

    pub fn maybe_space_then<'a, P, A>(parser: P) -> impl Parser<'a, A>
    where
        P: Parser<'a, A>,
    {
        right(optional_space(), parser)
    }

    pub fn space_separated<'a, P, A>(parser: P) -> impl Parser<'a, Vec<A>>
    where
        P: Parser<'a, A>,
    {
        one_plus(left(parser, optional_space()))
    }

    // TODO should token consider spaces? probably not
    pub fn identifier<'a>() -> impl Parser<'a, &'a str> {
        right(optional_space(), parse_identifier)
    }

    pub fn operator<'a>() -> impl Parser<'a, &'a str> {
        pred(parse_any_char_as_str, |c| {
            *c == "+" || *c == "-" || *c == "*" || *c == "/" || *c == "="
        })
    }

    pub fn number_literal<'a>() -> impl Parser<'a, &'a str> {
        move |ctx: ParseContext<'a>| {
            let mut matched = 0;
            let mut chars = ctx.remaining.chars();

            if let Some(c) = chars.next() {
                if c.is_digit(10) || c == '-' {
                    matched += 1
                } else {
                    return Err(ParseError::from_ctx(
                        ctx,
                        format!("number literal must start with a digit or '-' (got {c})"),
                    ));
                }
            } else {
                return Err(ParseError::from_ctx(
                    ctx,
                    format!("number literal got empty string"),
                ));
            }

            let mut decimal = false;

            for next in chars {
                if next.is_whitespace() || next == ')' || next == ']' {
                    break;
                } else if next.is_digit(10) {
                    matched += 1;
                } else if next == '.' {
                    if decimal == false {
                        matched += 1;
                        decimal = true;
                    } else {
                        return Err(ParseError::from_ctx(
                            ctx,
                            "found multiple decimal points while parsing number literal",
                        ));
                    }
                } else {
                    return Err(ParseError::from_ctx(
                        ctx,
                        format!("found {next} while trying to parse number literal"),
                    ));
                }
            }

            Ok(ctx.produce(matched))
        }
    }

    pub fn string_literal<'a>() -> impl Parser<'a, &'a str> {
        right(
            optional_space(),
            move |ctx: ParseContext<'a>| -> ParseResult<&'a str> {
                let mut chars = ctx.remaining.chars();
                match chars.next() {
                    Some(c) => match c {
                        '"' => {
                            let mut next_escaped = false;
                            let mut closed = false;
                            let mut matched = 1;

                            for i in chars {
                                matched += 1;
                                if next_escaped {
                                    next_escaped = false;
                                } else if i == '\\' {
                                    next_escaped = true;
                                } else if i == '"' {
                                    closed = true;
                                    break;
                                }
                            }

                            if closed {
                                Ok(ctx.produce(matched))
                            } else {
                                Err(ParseError::from_ctx(
                                    ctx,
                                    "string literal is missing closing \"",
                                ))
                            }
                        }
                        _ => Err(ParseError::from_ctx(
                            ctx,
                            "not a string literal (expected opening \")",
                        )),
                    },
                    _ => Err(ParseError::from_ctx(
                        ctx,
                        "not a string literal (got empty string)",
                    )),
                }
            },
        )
    }
}
