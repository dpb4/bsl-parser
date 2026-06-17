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
            line: 1,
            col: 1,
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
        self.col += n;

        // check for newlines to update line/col
        for c in new.chars() {
            if c == '\n' {
                self.line += 1;
                self.col = 1;
            }
        }

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
    recoverable: bool,
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
            recoverable: true,
        }
    }
    pub fn append_msg<S: Into<String>>(self, new_line: S) -> Self {
        Self {
            ctx: self.ctx,
            msg: self.msg + "\n" + &new_line.into(),
            recoverable: self.recoverable,
        }
    }
    pub fn unrecoverable(self) -> Self {
        Self {
            ctx: self.ctx,
            msg: self.msg,
            recoverable: false,
        }
    }
}

pub trait Parser<'a, Output> {
    fn parse(&self, ctx: ParseContext<'a>) -> ParseResult<'a, Output>;

    fn map<F, NewOutput>(self, map_fn: F) -> impl Parser<'a, NewOutput>
    where
        F: Fn(Output) -> NewOutput,
        Self: Sized,
    {
        com::map(self, map_fn)
    }

    fn map_with_context<F, B, S>(self, map_fn: F, error_context: S) -> impl Parser<'a, B>
    where
        S: Into<String> + Clone,
        F: Fn(Output) -> B,
        Self: Sized,
    {
        com::map_with_context(self, map_fn, error_context)
    }

    fn then<ParserOther, OutputOther>(
        self,
        other: ParserOther,
    ) -> impl Parser<'a, (Output, OutputOther)>
    where
        ParserOther: Parser<'a, OutputOther>,
        Self: Sized,
    {
        com::pair(self, other)
    }

    fn apply<F, NewOutput, S>(self, try_fn: F) -> impl Parser<'a, NewOutput>
    where
        S: Into<String>,
        F: Fn(Output) -> Result<NewOutput, S>,
        Self: Sized,
    {
        com::apply(self, try_fn)
    }

    fn or(self, parser2: impl Parser<'a, Output>) -> impl Parser<'a, Output>
    where
        Self: Sized,
    {
        com::or(self, parser2)
    }

    fn lazy(self) -> impl Parser<'a, Output>
    where
        Self: Sized,
    {
        com::lazy(self)
    }

    fn pred<F>(self, predicate: F) -> impl Parser<'a, Output>
    where
        F: Fn(&Output) -> bool,
        Self: Sized,
    {
        com::pred(self, predicate)
    }

    fn ignore_then<P, OutputKept>(self, parser_kept: P) -> impl Parser<'a, OutputKept>
    where
        P: Parser<'a, OutputKept>,
        Self: Sized,
    {
        com::right(self, parser_kept)
    }

    fn then_ignore<P, OutputIgnored>(self, parser_ignored: P) -> impl Parser<'a, Output>
    where
        P: Parser<'a, OutputIgnored>,
        Self: Sized,
    {
        com::left(self, parser_ignored)
    }

    fn one_plus(self) -> impl Parser<'a, Vec<Output>>
    where
        Self: Sized,
    {
        com::one_plus(self)
    }

    fn zero_plus(self) -> impl Parser<'a, Vec<Output>>
    where
        Self: Sized,
    {
        com::zero_plus(self)
    }

    fn parse_until<ParserLast, OutputLast>(
        self,
        last: ParserLast,
    ) -> impl Parser<'a, (Vec<Output>, OutputLast)>
    where
        ParserLast: Parser<'a, OutputLast>,
        Self: Sized,
    {
        com::parse_a_until_b(self, last)
    }
}

impl<'a, F, Output> Parser<'a, Output> for F
where
    F: Fn(ParseContext<'a>) -> ParseResult<'a, Output>,
{
    fn parse(&self, ctx: ParseContext<'a>) -> ParseResult<'a, Output> {
        self(ctx)
    }
}
pub type BoxedParser<'a, Output> = Box<dyn Parser<'a, Output> + 'a>;

impl<'a, Output> Parser<'a, Output> for BoxedParser<'a, Output> {
    fn parse(&self, ctx: ParseContext<'a>) -> ParseResult<'a, Output> {
        (**self).parse(ctx)
    }
}

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

    pub fn map_with_context<'a, P, F, A, B, S>(
        parser: P,
        map_fn: F,
        error_context: S,
    ) -> impl Parser<'a, B>
    where
        S: Into<String> + Clone,
        P: Parser<'a, A>,
        F: Fn(A) -> B,
    {
        move |ctx| {
            match parser.parse(ctx) {
                Ok((next_ctx, result)) => Ok((next_ctx, map_fn(result))),
                Err(p) => Err(p.append_msg(error_context.clone())),
            }
            // .map(|(next_ctx, result)| (next_ctx, map_fn(result)))
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

    pub fn join<'a, P, A, I>(parser: P) -> impl Parser<'a, A>
    where
        P: Parser<'a, I>,
        I: Parser<'a, A>,
    {
        move |ctx| {
            parser
                .parse(ctx)
                .and_then(|(ctx2, inner)| inner.parse(ctx2))
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
            Err(ParseError {
                recoverable: true, ..
            }) => parser2.parse(ctx),
            e @ Err(ParseError {
                recoverable: false, ..
            }) => e,
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
            e => e,
        }
    }

    pub fn lazy<'a, P, A>(parser: P) -> impl Parser<'a, A>
    where
        P: Parser<'a, A>,
    {
        move |s| parser.parse(s)
    }

    pub fn peek<'a, P, A>(parser: P) -> impl Parser<'a, A>
    where
        P: Parser<'a, A>,
    {
        move |ctx: ParseContext<'a>| {
            let c = dbg!(ctx.clone());
            parser.parse(ctx).map(|(_, result)| (c, result))
        }
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

            Ok(((cur_ctx), result))
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
        move |ctx: ParseContext<'a>| {
            let mut result_vec = vec![];
            let mut next_ctx = ctx;

            loop {
                let attempt_ctx = next_ctx.clone();

                match last.parse(next_ctx) {
                    Ok((r, v)) => {
                        return Ok((r, (result_vec, v)));
                    }
                    Err(_) => match multiple.parse(attempt_ctx) {
                        Ok((r, v)) => {
                            result_vec.push(v);
                            next_ctx = r;
                        }
                        Err(e) => return Err(e.append_msg("in parse_a_until_b, 'a' parser failed")),
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
                        format!("'{s}' does not match exact string '{expected}'",),
                    ))
                }
            } else {
                let s = ctx.remaining;
                Err(ParseError::from_ctx(
                    ctx,
                    format!(
                        "'{s}' does not match exact string '{expected}' (string not long enough)",
                    ),
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

    pub fn unrecoverable<'a, P, A>(parser: P) -> impl Parser<'a, A>
    where
        P: Parser<'a, A>,
    {
        move |ctx: ParseContext<'a>| parser.parse(ctx).map_err(|e| e.unrecoverable())
    }

    pub fn paren<'a, P, A>(parser: P) -> impl Parser<'a, A>
    where
        P: Parser<'a, A>,
    {
        match_exact("(")
            .ignore_then(parser)
            .then_ignore(match_exact(")"))
        // left(right(match_exact("("), parser), match_exact(")"))
    }

    pub fn bracket<'a, P, A>(parser: P) -> impl Parser<'a, A>
    where
        P: Parser<'a, A>,
    {
        match_exact("[")
            .ignore_then(parser)
            .then_ignore(match_exact("]"))

        // left(right(match_exact("["), parser), match_exact("]"))
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
            if next.is_whitespace() || next == ')' || next == ']' || next == ';' {
                break;
            } else if next.is_alphanumeric() || next == '-' || next == '_' || next == '?' {
                matched += 1;
            } else {
                return Err(ParseError::from_ctx(
                    ctx,
                    format!("identifier must be alphanumeric with '-' or '_' or '?' (found illegal character `{next}`)"),
                ));
            }
        }

        Ok(ctx.produce(matched))
    }

    pub fn parse_any_char(ctx: ParseContext) -> ParseResult<char> {
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

    pub fn space_or_comment<'a>() -> impl Parser<'a, ()> {
        move |ctx: ParseContext<'a>| {
            let mut matched = 0;
            let mut in_comment = false;

            for next in ctx.remaining.chars() {
                if in_comment {
                    if next == '\n' {
                        in_comment = false;
                    }
                } else if !next.is_whitespace() {
                    if next == ';' {
                        in_comment = true;
                    } else {
                        break;
                    }
                }
                matched += 1;
            }
            Ok((ctx.produce(matched).0, ()))
        }
    }

    // TODO clean this up, remove redudant calls to this fn
    pub fn optional_space<'a>() -> impl Parser<'a, ()> {
        // zero_plus(whitespace_char())
        space_or_comment()
    }

    pub fn maybe_space_then<'a, P, A>(parser: P) -> impl Parser<'a, A>
    where
        P: Parser<'a, A>,
    {
        optional_space().ignore_then(parser)
    }

    pub fn space_separated<'a, P, A>(parser: P) -> impl Parser<'a, Vec<A>>
    where
        P: Parser<'a, A>,
    {
        one_plus(parser.then_ignore(optional_space()))
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
                if c.is_ascii_digit() || c == '-' {
                    matched += 1
                } else {
                    return Err(ParseError::from_ctx(
                        ctx,
                        format!("number literal must start with a digit or '-' (got {c})"),
                    ));
                }
            } else {
                return Err(ParseError::from_ctx(ctx, "number literal got empty string"));
            }

            let mut decimal = false;

            for next in chars {
                if next.is_whitespace() || next == ')' || next == ']' {
                    break;
                } else if next.is_ascii_digit() {
                    matched += 1;
                } else if next == '.' {
                    if !decimal {
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
                        format!("found `{next}` while trying to parse number literal"),
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
                        "not a string literal (got empty input)",
                    )),
                }
            },
        )
    }
}
