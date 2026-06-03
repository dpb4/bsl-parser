pub type ParseResult<'a, Output> = Result<(&'a str, Output), &'a str>;

pub trait Parser<'a, Output> {
    fn parse(&self, input: &'a str) -> ParseResult<'a, Output>;

    // fn map<'a, P, F, B>(&self, map_fn: F) -> impl Parser<'a, B>
    // where
    //     P: Parser<'a, A>,
    //     F: Fn(Output) -> B,
    // {
    //     move |input| {
    //         self.parse(input)
    //             .map(|(next_input, result)| (next_input, map_fn(result)))
    //     }
    // }
}

impl<'a, F, Output> Parser<'a, Output> for F
where
    F: Fn(&'a str) -> ParseResult<'a, Output>,
{
    fn parse(&self, input: &'a str) -> ParseResult<'a, Output> {
        self(input)
    }
}

pub struct BoxedParser<'a, Output> {
    parser: Box<dyn Parser<'a, Output> + 'a>,
}

impl<'a, Output> BoxedParser<'a, Output> {
    pub fn new<P>(parser: P) -> Self
    where
        P: Parser<'a, Output> + 'a,
    {
        BoxedParser {
            parser: Box::new(parser),
        }
    }
}

impl<'a, Output> Parser<'a, Output> for BoxedParser<'a, Output> {
    fn parse(&self, input: &'a str) -> ParseResult<'a, Output> {
        self.parser.parse(input)
    }
}

pub mod com {
    use super::Parser;

    pub fn pair<'a, P1, P2, R1, R2>(parser1: P1, parser2: P2) -> impl Parser<'a, (R1, R2)>
    where
        P1: Parser<'a, R1>,
        P2: Parser<'a, R2>,
    {
        move |input| {
            parser1.parse(input).and_then(|(next_input, result1)| {
                parser2
                    .parse(next_input)
                    .map(|(last_input, result2)| (last_input, (result1, result2)))
            })
        }
    }

    pub fn map<'a, P, F, A, B>(parser: P, map_fn: F) -> impl Parser<'a, B>
    where
        P: Parser<'a, A>,
        F: Fn(A) -> B,
    {
        move |input| {
            parser
                .parse(input)
                .map(|(next_input, result)| (next_input, map_fn(result)))
        }
    }

    pub fn and_then<'a, P, F, A, B>(parser: P, map_fn: F) -> impl Parser<'a, B>
    where
        P: Parser<'a, A>,
        F: Fn(A) -> Result<B, &'a str>,
    {
        move |input| {
            parser
                .parse(input)
                .and_then(|(next_input, val)| map_fn(val).map(|val2| (next_input, val2)))
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
        move |input| match parser1.parse(input) {
            r @ Ok(_) => r,
            Err(_) => parser2.parse(input),
        }
    }
}
pub mod par {
    use crate::closes;
    use crate::is_closing;
    use crate::is_opening;

    use super::com::*;
    use super::BoxedParser;
    use super::ParseResult;
    use super::Parser;

    pub fn identity<'a>() -> impl Parser<'a, ()> {
        move |input| Ok((input, ()))
    }

    pub fn match_exact<'a>(expected: &'static str) -> impl Parser<'a, &'static str> {
        move |input: &'a str| {
            if let Some(stripped) = input.strip_prefix(expected) {
                Ok((stripped, expected))
            } else {
                Err(input)
            }
        }
    }
    pub fn match_exact_end<'a>(expected: &'static str) -> impl Parser<'a, &'static str> {
        move |input: &'a str| {
            if let Some(stripped) = input.strip_suffix(expected) {
                Ok((stripped, expected))
            } else {
                Err(input)
            }
        }
    }
    pub fn pred<'a, P, A, F>(parser: P, predicate: F) -> impl Parser<'a, A>
    where
        P: Parser<'a, A>,
        F: Fn(&A) -> bool,
    {
        move |input| {
            if let Ok((next_input, value)) = parser.parse(input) {
                if predicate(&value) {
                    return Ok((next_input, value));
                }
            }
            Err(input)
        }
    }
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

    pub fn lazy<'a, P, A>(parser: P) -> impl Parser<'a, A>
    where
        P: Parser<'a, A>,
    {
        move |s| parser.parse(s)
    }

    pub fn bracket<'a, P, A>(parser: P) -> impl Parser<'a, A>
    where
        P: Parser<'a, A>,
    {
        left(right(match_exact("["), parser), match_exact("]"))
        // right(pair(match_exact("["), match_exact_end(u]u)), parser)
    }

    pub fn one_plus<'a, P, A>(parser: P) -> impl Parser<'a, Vec<A>>
    where
        P: Parser<'a, A>,
    {
        move |mut input| {
            let mut result = Vec::new();

            if let Ok((next_input, first_item)) = parser.parse(input) {
                input = next_input;
                result.push(first_item);
            } else {
                return Err(input);
            }

            while let Ok((next_input, next_item)) = parser.parse(input) {
                input = next_input;
                result.push(next_item);
            }

            Ok((input, result))
        }
    }

    pub fn zero_plus<'a, P, A>(parser: P) -> impl Parser<'a, Vec<A>>
    where
        P: Parser<'a, A>,
    {
        move |mut input| {
            let mut result = Vec::new();

            while let Ok((next_input, next_item)) = parser.parse(input) {
                input = next_input;
                result.push(next_item);
            }

            Ok((input, result))
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
        move |input| {
            let mut result_vec = vec![];
            let mut remaining = input;

            loop {
                match last.parse(remaining) {
                    Ok((r, v)) => {
                        return Ok((r, (result_vec, v)));
                    }
                    Err(_) => match multiple.parse(remaining) {
                        Ok((r, v)) => {
                            result_vec.push(v);
                            remaining = r;
                        }
                        Err(e) => return Err(e),
                    },
                }
            }
        }
    }

    pub fn parse_simple_word<'a>(input: &'a str) -> ParseResult<'a, &'a str> {
        let mut matched = 0;
        let mut chars = input.chars();

        match chars.next() {
            Some(_) => {
                matched += 1;
            }
            _ => {
                return Err(input);
            }
        };

        for next in chars {
            if next.is_alphanumeric() || next == '-' || next == '_' {
                // if !next.is_whitespace() {
                matched += 1;
            } else {
                break;
            }
        }

        Ok((&input[matched..], &input[..matched]))
    }

    pub fn parse_token<'a>(input: &'a str) -> ParseResult<'a, &'a str> {
        let mut matched = 0;
        let mut chars = input.chars();

        if let Some(c) = chars.next() {
            if c.is_alphabetic() || c == '_' {
                matched += 1
            } else {
                return Err(input);
            }
        } else {
            return Err(input);
        }

        for next in chars {
            if next.is_whitespace() || is_closing(next) {
                break;
            } else if next.is_alphanumeric() || next == '-' || next == '_' {
                matched += 1;
            } else {
                return Err(input);
            }
        }

        Ok((&input[matched..], &input[..matched]))
    }

    pub fn parse_any_char(input: &str) -> ParseResult<'_, char> {
        match input.chars().next() {
            Some(next) => Ok((&input[next.len_utf8()..], next)),
            _ => Err(input),
        }
    }

    pub fn parse_any_char_as_str(input: &str) -> ParseResult<'_, &str> {
        match input.get(0..1) {
            Some(next) => Ok((&input[1..], next)),
            _ => Err(input),
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
    pub fn token<'a>() -> impl Parser<'a, &'a str> {
        right(
            optional_space(),
            BoxedParser::new(move |input: &'a str| -> ParseResult<'a, &'a str> {
                parse_token(input)
            }),
        )
    }

    pub fn operator<'a>() -> impl Parser<'a, &'a str> {
        pred(parse_any_char_as_str, |c| {
            *c == "+" || *c == "-" || *c == "*" || *c == "/" || *c == "="
        })
    }

    pub fn number_literal<'a>() -> impl Parser<'a, &'a str> {
        move |input: &'a str| {
            let mut matched = 0;
            let mut chars = input.chars();

            if let Some(c) = chars.next() {
                if c.is_digit(10) || c == '-' {
                    matched += 1
                } else {
                    return Err(input);
                }
            } else {
                return Err(input);
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
                        return Err(input);
                    }
                } else {
                    return Err(input);
                }
            }

            Ok((&input[matched..], &input[..matched]))
        }
    }

    pub fn string_literal<'a>() -> impl Parser<'a, &'a str> {
        right(
            optional_space(),
            BoxedParser::new(move |input: &'a str| -> ParseResult<'a, &'a str> {
                match input.chars().next() {
                    Some(c) => match c {
                        '"' => {
                            let mut next_escaped = false;
                            let mut closed = false;
                            let mut end_index = 1;

                            for i in input.chars().skip(1) {
                                end_index += 1;
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
                                Ok((&input[end_index..], &input[..end_index]))
                            } else {
                                Err("unmatched pair")
                            }
                        }
                        _ => Err("not a string literal"),
                    },
                    None => Err("not a string literal"),
                }
            }),
        )
    }

    pub fn blob<'a>() -> impl Parser<'a, &'a str> {
        right(
            optional_space(),
            BoxedParser::new(move |input: &'a str| -> ParseResult<'a, &'a str> {
                match input.chars().next() {
                    Some(c) => match c {
                        '(' | '[' | '\'' | 'u' => {
                            let mut pair_stack = vec![];

                            let mut next_escaped = false;
                            let mut end_index = 0;

                            for i in input.chars() {
                                if next_escaped {
                                    next_escaped = false;
                                } else if i == '\\' {
                                    next_escaped = true;
                                } else if !pair_stack.is_empty()
                                    && closes(*pair_stack.last().unwrap(), i)
                                {
                                    pair_stack.pop();
                                } else if is_opening(i) {
                                    pair_stack.push(i);
                                }
                                end_index += 1;

                                if pair_stack.is_empty() {
                                    break;
                                }
                            }

                            if pair_stack.is_empty() {
                                Ok((&input[end_index..], &input[..end_index]))
                            } else {
                                Err("unmatched pair")
                            }
                        }
                        _ => parse_simple_word(input),
                    },
                    None => Err("no word"),
                }
            }),
        )
    }
}
