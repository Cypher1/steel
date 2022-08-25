use nom::{
  IResult,
  bytes::complete::{tag, take_while, take_while_m_n},
  combinator::map_res,
  branch::alt,
  character::complete::alpha1,
  sequence::tuple
};

use crate::primitives::Color;
use crate::ast::Symbol;

fn from_hex(input: &str) -> Result<u8, std::num::ParseIntError> {
  u8::from_str_radix(input, 16)
}

fn is_hex_digit(c: char) -> bool {
  c.is_digit(16)
}

fn hex_primary(input: &str) -> IResult<&str, u8> {
  map_res(
    take_while_m_n(2, 2, is_hex_digit),
    from_hex
  )(input)
}

pub fn hex_color(input: &str) -> IResult<&str, Color> {
  let (input, _) = tag("#")(input)?;
  let (input, (red, green, blue)) = tuple((hex_primary, hex_primary, hex_primary))(input)?;

  Ok((input, Color { red, green, blue }))
}

fn is_symbol_char(c: char) -> bool {
  c.is_alphanumeric() || (c == '_')
}

fn symbol_head(input: &str) -> IResult<&str, &str> {
  alt((alpha1, tag("_")))(input)
}

fn symbol_tail(input: &str) -> IResult<&str, &str> {
  take_while(is_symbol_char)(input)
}

pub fn symbol(og_input: &str) -> IResult<&str, Symbol> {
  let (input, (head, tail)) = tuple((symbol_head, symbol_tail))(og_input)?;

  let name = &og_input[0..head.len()+tail.len()];

  Ok((input, Symbol { name }))
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::assertions::assert_is_err;

    #[test]
    fn parse_color() {
      assert_eq!(hex_color("#2F14DF"), Ok(("", Color {
        red: 47,
        green: 20,
        blue: 223,
      })));
    }

    #[test]
    fn parse_non_color() {
      let err = assert_is_err(hex_color("#lol"));
      assert_eq!(format!("{}", err), "Parsing Error: Error { input: \"lol\", code: TakeWhileMN }");
    }

    #[test]
    fn parse_symbol() {
      assert_eq!(symbol("hello"), Ok(("", Symbol { name: "hello" })));
    }

    #[test]
    fn parse_symbol_with_underscores() {
      assert_eq!(symbol("he_llo"), Ok(("", Symbol { name: "he_llo" })));
      assert_eq!(symbol("_e_llo"), Ok(("", Symbol { name: "_e_llo" })));
    }

    fn assert_err_is<T: std::fmt::Debug, E: std::fmt::Display>(value: Result<T, E>, err_msg: &str) {
      let err = assert_is_err(value);
      assert_eq!(format!("{}", err), err_msg);
    }

    #[test]
    fn parse_non_symbol() {
      assert_err_is(symbol("#lol"), "Parsing Error: Error { input: \"#lol\", code: Tag }");
      assert_err_is(symbol("123"), "Parsing Error: Error { input: \"123\", code: Tag }");
    }
}
