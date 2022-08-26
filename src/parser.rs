use nom::{
    branch::alt,
    bytes::complete::{tag, take_while, take_while1, take_while_m_n},
    character::complete::alpha1,
    combinator::map_res,
    sequence::tuple,
    IResult,
};

use crate::nodes::{Call, Symbol};
use crate::primitives::Color;

pub trait ParserStorage<ID, T, E> {
    fn add(&mut self, value: T) -> ID;
    fn get(&self, id: ID) -> Result<&T, E>;
    fn get_mut(&mut self, id: ID) -> Result<&mut T, E>;
}

pub trait ParserContext<'source>:
    ParserStorage<Self::ID, Call<Self::ID>, Self::E>
    + ParserStorage<Self::ID, Symbol<'source>, Self::E>
    + ParserStorage<Self::ID, i64, Self::E>
{
    type ID;
    type E;
}

fn from_hex(input: &str) -> Result<u8, std::num::ParseIntError> {
    u8::from_str_radix(input, 16)
}

fn is_hex_digit(c: char) -> bool {
    c.is_ascii_hexdigit()
}

fn hex_primary(input: &str) -> IResult<&str, u8> {
    map_res(take_while_m_n(2, 2, is_hex_digit), from_hex)(input)
}

pub fn hex_color(input: &str) -> IResult<&str, Color> {
    let (input, _) = tag("#")(input)?;
    let (input, (red, green, blue)) = tuple((hex_primary, hex_primary, hex_primary))(input)?;

    Ok((input, Color { red, green, blue }))
}

pub fn number_i64_raw(input: &str) -> IResult<&str, i64> {
    let (input, sign) = alt((tag("+"), tag("-"), tag("")))(input)?;
    let (input, value) = map_res(
        take_while1(&|c: char| c.is_ascii_digit()),
        &|input: &str| input.parse::<i64>(),
    )(input)?;
    Ok((input, if sign == "-" { -value } else { value }))
}

pub fn number_i64<'source, ID, E, C: ParserStorage<ID, i64, E>>(
    context: &mut C,
    input: &'source str,
) -> IResult<&'source str, ID> {
    let (input, value) = number_i64_raw(input)?;
    let id = context.add(value);
    Ok((input, id))
}

fn is_identifier_char(c: char) -> bool {
    c.is_alphanumeric() || (c == '_')
}

fn identifier_head(input: &str) -> IResult<&str, &str> {
    alt((alpha1, tag("_")))(input)
}

fn identifier_tail(input: &str) -> IResult<&str, &str> {
    take_while(is_identifier_char)(input)
}

pub fn symbol_raw(og_input: &str) -> IResult<&str, Symbol> {
    let (input, (head, tail)) = tuple((identifier_head, identifier_tail))(og_input)?;

    let name = &og_input[0..head.len() + tail.len()];

    Ok((input, Symbol { name }))
}
pub fn symbol<'source, ID, E, C: ParserStorage<ID, Symbol<'source>, E>>(
    context: &mut C,
    input: &'source str,
) -> IResult<&'source str, ID> {
    let (input, symbol) = symbol_raw(input)?;
    let id = context.add(symbol);
    Ok((input, id))
}

fn is_operator_char(c: char) -> bool {
    "~!@$%^&*-+=|?/\\:".contains(c)
}

pub fn operator_raw(input: &str) -> IResult<&str, Symbol> {
    let (input, name) = take_while_m_n(1, 3, is_operator_char)(input)?;
    Ok((input, Symbol { name }))
}
pub fn operator<'source, ID, E, C: ParserStorage<ID, Symbol<'source>, E>>(
    context: &mut C,
    input: &'source str,
) -> IResult<&'source str, ID> {
    let (input, symbol) = operator_raw(input)?;
    let id = context.add(symbol);
    Ok((input, id))
}

pub fn expr<'source, C: ParserContext<'source>>(
    context: &mut C,
    input: &'source str,
) -> IResult<&'source str, C::ID> {
    if let Ok((input, _)) = tag::<&str, &str, nom::error::Error<_>>("(")(input) {
        let (input, left) = expr(context, input)?;
        let (input, op) = operator(context, input)?;
        let (input, right) = expr(context, input)?;
        let call = context.add(Call::new(op, vec![left, right]));
        return Ok((input, call));
    }
    if let Ok(res) = number_i64(context, input) {
        return Ok(res);
    }
    if let Ok(res) = symbol(context, input) {
        return Ok(res);
    }
    todo!("Unexpected '{}'", input);
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::assertions::{assert_err_is, assert_is_err};

    #[test]
    fn parse_color() {
        assert_eq!(
            hex_color("#2F14DF"),
            Ok((
                "",
                Color {
                    red: 47,
                    green: 20,
                    blue: 223,
                }
            ))
        );
    }

    #[test]
    fn parse_non_color() {
        let err = assert_is_err(hex_color("#lol"));
        assert_eq!(
            format!("{}", err),
            "Parsing Error: Error { input: \"lol\", code: TakeWhileMN }"
        );
    }

    #[test]
    fn parse_symbol() {
        assert_eq!(symbol_raw("hello"), Ok(("", Symbol { name: "hello" })));
    }

    #[test]
    fn parse_symbol_with_underscores() {
        assert_eq!(symbol_raw("he_llo"), Ok(("", Symbol { name: "he_llo" })));
        assert_eq!(symbol_raw("_e_llo"), Ok(("", Symbol { name: "_e_llo" })));
    }

    #[test]
    fn parse_non_symbol() {
        assert_err_is(
            symbol_raw("#lol"),
            "Parsing Error: Error { input: \"#lol\", code: Tag }",
        );
        assert_err_is(
            symbol_raw("123"),
            "Parsing Error: Error { input: \"123\", code: Tag }",
        );
    }

    #[test]
    fn parse_operator() {
        assert_eq!(operator_raw("||"), Ok(("", Symbol { name: "||" })));
        assert_eq!(operator_raw("+"), Ok(("", Symbol { name: "+" })));
        assert_eq!(operator_raw("*"), Ok(("", Symbol { name: "*" })));
    }

    #[test]
    fn parse_non_operator() {
        assert_err_is(
            operator_raw("#lol"),
            "Parsing Error: Error { input: \"#lol\", code: TakeWhileMN }",
        );
        assert_err_is(
            operator_raw("123"),
            "Parsing Error: Error { input: \"123\", code: TakeWhileMN }",
        );
    }

    #[test]
    fn parse_number_i64() {
        assert_eq!(number_i64_raw("123"), Ok(("", 123i64)));
        assert_eq!(number_i64_raw("-0"), Ok(("", 0i64)));
        assert_eq!(number_i64_raw("-1"), Ok(("", -1i64)));
    }

    #[test]
    fn parse_non_number_i64() {
        assert_err_is(
            number_i64_raw("#lol"),
            "Parsing Error: Error { input: \"#lol\", code: TakeWhile1 }",
        );
        assert_err_is(
            number_i64_raw("||"),
            "Parsing Error: Error { input: \"||\", code: TakeWhile1 }",
        );
        assert_err_is(
            number_i64_raw("e2"),
            "Parsing Error: Error { input: \"e2\", code: TakeWhile1 }",
        );
    }
}
