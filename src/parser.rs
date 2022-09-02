use nom::{
    branch::alt,
    bytes::complete::{tag as raw_tag, take_while, take_while1, take_while_m_n},
    character::complete::{space0, alpha1},
    combinator::map_res,
    sequence::tuple,
    multi::separated_list0,
};

type SResult<'a, T> = std::result::Result<(&'a str, T), nom::Err<SteelErr>>;

// Automatically ignore whitespace by default...
// TODO: Consider only ignoring some whitespace...
pub fn tag(
    raw: &str
) -> impl Fn(&str) -> SResult<&str> + '_ {
    move |input: &str| {
        let (input, _) = space0::<&str, SteelErr>(input)?;
        raw_tag::<&str, &str, SteelErr>(raw)(input)
            // TODO: This might be expensive?
            //Err(_) => Err(SteelErr::ParseErrorExpected(raw.to_string(), input.to_string())),
    }
}

use crate::nodes::{Call, Symbol};
use crate::primitives::Color;
use crate::error::SteelErr;

pub trait ParserStorage<'source, ID, T, E> {
    fn add(&mut self, value: T) -> ID;
    fn get(&self, id: ID) -> Result<&T, E>;
    fn get_mut(&mut self, id: ID) -> Result<&mut T, E>;
}

pub trait ParserContext<'source>:
    ParserStorage<'source, Self::ID, Call<Self::ID>, Self::E>
    + ParserStorage<'source, Self::ID, Symbol<'source>, Self::E>
    + ParserStorage<'source, Self::ID, i64, Self::E>
{
    type ID: Copy + std::fmt::Debug;
    type E;

    fn get_symbol(&self, id: Self::ID) -> Result<&Symbol<'source>, Self::E> {
        self.get(id)
    }
    fn get_symbol_mut(&mut self, id: Self::ID) -> Result<&mut Symbol<'source>, Self::E> {
        self.get_mut(id)
    }
    fn get_call(&self, id: Self::ID) -> Result<&Call<Self::ID>, Self::E> {
        self.get(id)
    }
    fn get_call_mut(&mut self, id: Self::ID) -> Result<&mut Call<Self::ID>, Self::E> {
        self.get_mut(id)
    }
    fn get_i64(&self, id: Self::ID) -> Result<&i64, Self::E> {
        self.get(id)
    }
    fn get_i64_mut(&mut self, id: Self::ID) -> Result<&mut i64, Self::E> {
        self.get_mut(id)
    }
    fn active_mem_usage(&self) -> usize;
    fn mem_usage(&self) -> usize;
    fn pretty(&self, id: Self::ID) -> String {
        if let Ok(v) = self.get_i64(id) {
            return format!("{}", v);
        }
        if let Ok(s) = self.get_symbol(id) {
            return format!("{}", s.name);
        }
        if let Ok(c) = self.get_call(id) {
            let callee = self.pretty(c.callee);
            let is_operator_call = if let Ok(sym) = self.get_symbol(c.callee) {
                sym.is_operator
            } else {
                false
            };
            if is_operator_call {
                let args: Vec<String> = c.args.iter().map(|arg| self.pretty(*arg)).collect();
                let args = args.join(&callee);
                return format!("({}{})", if c.args.len() < 2 { &callee } else { "" }, args);
            }
            let args: Vec<String> = c.args.iter().map(|arg| self.pretty(*arg)).collect();
            let args = args.join(", ");
            return format!("{}({})", callee, args);
        }
        format!("{{node? {:?}}}", id)
    }
}

fn from_hex(input: &str) -> Result<u8, std::num::ParseIntError> {
    u8::from_str_radix(input, 16)
}

fn is_hex_digit(c: char) -> bool {
    c.is_ascii_hexdigit()
}

fn hex_primary(input: &str) -> SResult<u8> {
    map_res(take_while_m_n(2, 2, is_hex_digit), from_hex)(input)
}

pub fn hex_color(input: &str) -> SResult<Color> {
    let (input, _) = tag("#")(input)?;
    let (input, (red, green, blue)) = tuple((hex_primary, hex_primary, hex_primary))(input)?;

    Ok((input, Color { red, green, blue }))
}

pub fn number_i64_raw(input: &str) -> SResult<i64> {
    let (input, sign) = alt((tag("+"), tag("-"), tag("")))(input)?;
    let (input, value) = map_res(
        take_while1(&|c: char| c.is_ascii_digit()),
        &|input: &str| input.parse::<i64>(),
    )(input)?;
    Ok((input, if sign == "-" { -value } else { value }))
}

pub fn number_i64<'source, ID, E: Into<SteelErr>, C: ParserStorage<'source, ID, i64, E>>(
    context: &mut C,
    input: &'source str,
) -> SResult<'source, ID> {
    let (input, value) = number_i64_raw(input)?;
    let id = context.add(value);
    Ok((input, id))
}

fn is_identifier_char(c: char) -> bool {
    c.is_alphanumeric() || (c == '_')
}

fn identifier_head(input: &str) -> SResult<&str> {
    alt((alpha1, tag("_")))(input)
}

fn identifier_tail(input: &str) -> SResult<&str> {
    take_while(is_identifier_char)(input)
}

pub fn symbol_raw(og_input: &str) -> SResult<Symbol> {
    let (input, (head, tail)) = tuple((identifier_head, identifier_tail))(og_input)?;

    let name = &og_input[0..head.len() + tail.len()];

    Ok((input, Symbol::new(name)))
}
pub fn symbol<'source, ID, E: Into<SteelErr>, C: ParserStorage<'source, ID, Symbol<'source>, E>>(
    context: &mut C,
    input: &'source str,
) -> SResult<'source, ID> {
    let (input, _) = space0(input)?;
    let (input, symbol) = symbol_raw(input)?;
    let id = context.add(symbol);
    Ok((input, id))
}

fn is_operator_char(c: char) -> bool {
    "~!@$%^&*-+=|?/\\:".contains(c)
}

pub fn operator_raw(input: &str) -> SResult<Symbol> {
    let (input, name) = take_while_m_n(1, 3, is_operator_char)(input)?;
    Ok((input, Symbol::operator(name)))
}
pub fn operator<'source, ID, E: Into<SteelErr>, C: ParserStorage<'source, ID, Symbol<'source>, E>>(
    context: &mut C,
    input: &'source str,
) -> SResult<'source, ID> {
    let (input, _) = space0(input)?;
    let (input, symbol) = operator_raw(input)?;
    let id = context.add(symbol);
    Ok((input, id))
}

pub fn args<'source, C: ParserContext<'source>>(
    context: &mut C,
    input: &'source str,
) -> SResult<'source, Vec<C::ID>> where <C as ParserContext<'source>>::E: Into<SteelErr> {
    let (input, _) = tag("(")(input)?;
    let (input, args) = separated_list0(tag(","), |input|expr(context, input))(input)?;
    let (input, _) = tag(")")(input)?;
    return Ok((input, args));
}

pub fn led<'source, C: ParserContext<'source>>(
    context: &mut C,
    left: C::ID,
    input: &'source str,
) -> SResult<'source, C::ID> where <C as ParserContext<'source>>::E: Into<SteelErr> {
    // try precedence(less) parsing...
    // TODO: add precedence...
    let (input, op) = operator(context, input)?;
    let (input, right) = expr(context, input)?;
    let call = context.add(Call::new(op, vec![left, right]));
    return Ok((input, call));
}

pub fn nud<'source, C: ParserContext<'source>>(
    context: &mut C,
    input: &'source str,
) -> SResult<'source, C::ID> where <C as ParserContext<'source>>::E: Into<SteelErr> {
    if let Ok((input, _)) = tag("(")(input) {
        let (input, wrapped) = expr(context, input)?;
        let (input, _) = tag(")")(input)?;
        return Ok((input, wrapped));
    }
    if let Ok((input, sym)) = symbol(context, input) {
        if let Ok((input, args)) = args(context, input) {
            // Function call
            let call = context.add(Call::new(sym, args));
            return Ok((input, call));
        }
        return Ok((input, sym));
    }
    if let Ok((input, op)) = operator(context, input) {
        // Unified calling syntax for a prefix operator
        if let Ok((input, args)) = args(context, input) {
            // Function call
            if let Ok(op) = context.get_symbol_mut(op) { // TODO: WHAT!?
                op.is_operator = false;
            }
            let call = context.add(Call::new(op, args));
            return Ok((input, call));
        }
        // Prefix operator
        let (input, right) = expr(context, input)?;
        let call = context.add(Call::new(op, vec![right]));
        return Ok((input, call));
    }
    // Otherwise expect a number
    number_i64(context, input)
}

pub fn expr<'context, 'source: 'context, C: ParserContext<'source>>(
    context: &'context mut C,
    input: &'source str,
) -> SResult<'source, C::ID> where <C as ParserContext<'source>>::E: Into<SteelErr> {
    let mut state = nud(context, input)?;
    loop {
        let update = led(context, state.1, state.0);
        match update {
            Ok(new_state) => {
                state = new_state;
            }
            Err(_) => return Ok(state),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::assertions::{assert_err_is, assert_is_err};

    #[test]
    fn parse_color() {
        assert_eq!(
            hex_color("#2F14DF").expect("Should parse"),
            (
                "",
                Color {
                    red: 47,
                    green: 20,
                    blue: 223,
                }
            )
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
        assert_eq!(symbol_raw("hello").unwrap(), ("", Symbol::new("hello")));
    }

    #[test]
    fn parse_symbol_with_underscores() {
        assert_eq!(symbol_raw("he_llo").unwrap(), ("", Symbol::new("he_llo")));
        assert_eq!(symbol_raw("_e_llo").unwrap(), ("", Symbol::new("_e_llo")));
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
        assert_eq!(operator_raw("||").unwrap(), ("", Symbol::operator("||")));
        assert_eq!(operator_raw("+").unwrap(), ("", Symbol::operator("+")));
        assert_eq!(operator_raw("*").unwrap(), ("", Symbol::operator("*")));
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
        assert_eq!(number_i64_raw("123").unwrap(), ("", 123i64));
        assert_eq!(number_i64_raw("-0").unwrap(), ("", 0i64));
        assert_eq!(number_i64_raw("-1").unwrap(), ("", -1i64));
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
