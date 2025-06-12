use super::expr::Expr;
use nom::{
    IResult,
    branch::alt,
    bytes::complete::{
        escaped_transform,
        tag,
        take_while,
        take_while1,
    },
    character::complete::{
        char,
        digit1,
        multispace0,
        none_of,
        one_of,
    },
    combinator::{
        cut,
        eof,
        map,
        opt,
        value,
    },
    error::{
        ContextError,
        ErrorKind,
        ParseError,
        context,
    },
    multi::many0,
    sequence::{
        delimited,
        pair,
        preceded,
        terminated,
    },
};
use std::{
    collections::{
        BTreeMap,
        HashMap,
    },
    sync::Arc,
};

fn parse_int_literal<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, Expr, E> {
    let (input, result) = map(digit1, |s: &str| Expr::Int(s.parse().unwrap()))(input)?;
    if let Some(c) = input.chars().next() {
        if is_symbol_char(c) {
            return Err(nom::Err::Error(E::from_error_kind(input, ErrorKind::Digit)));
        }
    }

    Ok((input, result))
}

fn parse_float_literal<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, Expr, E> {
    // Parse a signed float
    let (input, is_negative) = opt(tag("-"))(input)?;

    // Use builtin nom double
    let (input, result) = nom::number::complete::recognize_float(input)?;
    // Try to parse as an integer first
    let result: f64 = if let Ok(_i) = result.parse::<i64>() {
        // Fail
        return Err(nom::Err::Error(E::from_error_kind(input, ErrorKind::Digit)));
    } else {
        result.parse().unwrap()
    };

    if is_negative.is_some() {
        Ok((input, (-result).into()))
    } else {
        Ok((input, result.into()))
    }
}
fn parse_inner_str_double<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, String, E> {
    let (a, b) = escaped_transform(
        none_of("\\\""),
        '\\',
        alt((
            value("\n", tag("n")),
            value("\"", tag("\"")),
            value("\\", tag("\\")),
        )),
    )(i)?;

    Ok((a, b))
}

fn parse_string_literal<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, Expr, E> {
    map(
        context(
            "string",
            alt((
                // preceded(char('\''), cut(terminated(parse_inner_str_single, char('\'')))),
                preceded(
                    char('"'),
                    cut(terminated(parse_inner_str_double, char('"'))),
                ),
            )),
        ),
        |s| Expr::String(s.to_string()),
    )(input)
}

fn parse_list<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, Expr, E> {
    map(
        delimited(
            char('('),
            cut(many0(parse_expr)),
            cut(preceded(multispace0, char(')'))),
        ),
        Expr::List,
    )(input)
}

fn parse_block<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, Expr, E> {
    map(
        delimited(
            char('{'),
            cut(many0(parse_expr)),
            cut(preceded(multispace0, char('}'))),
        ),
        |x| Expr::Many(Arc::new(x)),
    )(input)
}

fn parse_map<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, Expr, E> {
    map(
        context(
            "map",
            delimited(
                tag("#["),
                cut(many0(pair(parse_expr, preceded(multispace0, parse_expr)))),
                cut(preceded(multispace0, char(']'))),
            ),
        ),
        |pairs| {
            let mut map = HashMap::new();
            for (k, v) in pairs {
                map.insert(k, v);
            }
            Expr::Map(map)
        },
    )(input)
}

fn parse_tree<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, Expr, E> {
    map(
        context(
            "tree",
            delimited(
                char('['),
                cut(many0(pair(parse_expr, preceded(multispace0, parse_expr)))),
                cut(preceded(multispace0, char(']'))),
            ),
        ),
        |pairs| {
            let mut tree = BTreeMap::new();
            for (k, v) in pairs {
                tree.insert(k, v);
            }
            Expr::Tree(tree)
        },
    )(input)
}

/// Parse a program from a string.
/// This will directly use the nom parser combinator library to parse the input
/// string, and return the parsed expression.
///
/// It's recommended to use the parsing method for `Expr` instead of this
/// method.
pub fn parse_program<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, Expr, E> {
    // return cut(parse_expr)(input);
    let (input, exprs) = parse_expr(input)?;

    // Parse eof
    let (input, _) = context("end of program", eof)(input)?;

    Ok((input, exprs))
}

/// Parse a program from a string.
/// This will directly use the nom parser combinator library to parse the input
/// string, and return the parsed expression.
///
/// It's recommended to use the parsing method for `Expr` instead of this
/// method.
pub fn parse_expr<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, Expr, E> {
    preceded(
        multispace0,
        terminated(alt((parse_compare, parse_atom)), multispace0),
    )(input)
}

fn parse_compare<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, Expr, E> {
    let (input, lhs) = parse_sum(input)?;
    let (input, op) = opt(alt((
        tag("<="),
        tag(">="),
        tag("=="),
        tag("!="),
        tag("<"),
        tag(">"),
    )))(input)?;
    if let Some(op) = op {
        let (input, rhs) = parse_sum(input)?;
        Ok((
            input,
            match op {
                "<=" => Expr::symbol("<=").apply(&[lhs, rhs]),
                ">=" => Expr::symbol(">=").apply(&[lhs, rhs]),
                "==" => Expr::symbol("==").apply(&[lhs, rhs]),
                "!=" => Expr::symbol("!=").apply(&[lhs, rhs]),
                "<" => Expr::symbol("<").apply(&[lhs, rhs]),
                ">" => Expr::symbol(">").apply(&[lhs, rhs]),
                _ => unreachable!(),
            },
        ))
    } else {
        Ok((input, lhs))
    }
}

fn parse_sum<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, Expr, E> {
    let (input, lhs) = parse_mul(input)?;
    let (input, op) = opt(one_of("+-"))(input)?;
    if let Some(op) = op {
        let (input, rhs) = parse_sum(input)?;
        Ok((
            input,
            match op {
                '+' => Expr::symbol('+').apply(&[lhs, rhs]),
                '-' => Expr::symbol('-').apply(&[lhs, rhs]),
                _ => unreachable!(),
            },
        ))
    } else {
        Ok((input, lhs))
    }
}

fn parse_mul<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, Expr, E> {
    let (input, lhs) = parse_pow(input)?;
    let (input, _) = multispace0(input)?;
    let (input, op) = opt(one_of("*/%"))(input)?;
    if let Some(op) = op {
        let (input, rhs) = parse_mul(input)?;
        Ok((
            input,
            match op {
                '*' => Expr::symbol('*').apply(&[lhs, rhs]),
                '/' => Expr::symbol('/').apply(&[lhs, rhs]),
                '%' => Expr::symbol('%').apply(&[lhs, rhs]),
                _ => unreachable!(),
            },
        ))
    } else {
        Ok((input, lhs))
    }
}

fn parse_pow<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, Expr, E> {
    let (input, lhs) = parse_access(input)?;
    let (input, _) = multispace0(input)?;
    let (input, op) = opt(one_of("^"))(input)?;
    if let Some(op) = op {
        let (input, rhs) = parse_pow(input)?;
        Ok((
            input,
            match op {
                '^' => Expr::symbol('^').apply(&[lhs, rhs]),
                _ => unreachable!(),
            },
        ))
    } else {
        Ok((input, lhs))
    }
}

fn parse_access<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, Expr, E> {
    let (input, lhs) = parse_atom(input)?;
    let (input, _) = multispace0(input)?;

    let (input, op) = opt(one_of("@"))(input)?;
    if op.is_some() {
        let (input, rhs) = parse_atom(input)?;
        // Ok((input, match op {
        //     '@' => Expr::symbol('@').apply(&[lhs, rhs]),
        //     _ => unreachable!(),
        // }))

        let mut result = Expr::symbol('@').apply(&[lhs, rhs]);
        let (mut input, _) = multispace0(input)?;

        // See if there's another access
        while let Ok((i, _)) = tag::<&str, &str, E>("@")(input) {
            let (i, rhs) = parse_atom(i)?;
            result = Expr::symbol('@').apply(&[result.clone(), rhs]);
            let (i, _) = multispace0(i)?;
            input = i;
        }

        // println!("Result: {}", result);

        Ok((input, result))
    } else {
        Ok((input, lhs))
    }
}

fn parse_quote<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, Expr, E> {
    map(pair(tag("'"), parse_atom), |(_, expr)| expr.quote())(input)
}

fn is_symbol_char(c: char) -> bool {
    c.is_alphanumeric()
        || c == '_'
        || c == '?'
        || c == '!'
        || c == '.'
        || c == '-'
        || c == '+'
        || c == '*'
        || c == '/'
        || c == '%'
        || c == '<'
        || c == '>'
        || c == '='
        || c == '&'
        || c == '|'
        || c == '^'
        || c == '\\'
}

fn parse_symbol<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, Expr, E> {
    // Parse whitespace
    let (input, _) = multispace0(input)?;
    // Parse first character
    let (input, first) = take_while1(|x| is_symbol_char(x) && !x.is_numeric())(input)?;
    // Parse rest of characters
    let (input, rest) = take_while(is_symbol_char)(input)?;
    // Combine first and rest
    let symbol = format!("{}{}", first, rest);
    // println!("Got symbol: {:?}", symbol);
    // Return symbol
    Ok((input, Expr::symbol(symbol)))
}

fn parse_atom<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, Expr, E> {
    // Parse whitespace
    let (input, _) = multispace0(input)?;

    alt((
        value(Expr::None, tag("nil")),
        value(Expr::Bool(true), tag("true")),
        value(Expr::Bool(false), tag("false")),
        context("float", parse_float_literal),
        context("int", parse_int_literal),
        context("string", parse_string_literal),
        context("list", parse_list),
        context("block", parse_block),
        context("map", parse_map),
        context("tree", parse_tree),
        context("quote", parse_quote),
        context("symbol", parse_symbol),
    ))(input)
}
