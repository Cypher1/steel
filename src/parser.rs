use nom::{
    branch::alt,
    bytes::complete::{tag as raw_tag, take_while, take_while1, take_while_m_n},
    character::complete::{alpha1, multispace0},
    combinator::map_res,
    multi::separated_list0,
    sequence::tuple,
};

type SResult<'a, T> = std::result::Result<(&'a str, T), nom::Err<SteelErr>>;

fn tag(raw: &str) -> impl Fn(&str) -> SResult<&str> + '_ {
    // TODO: Consider only ignoring some whitespace...
    move |input: &str| {
        let (input, _) = multispace0::<&str, SteelErr>(input)?;
        raw_tag::<&str, &str, SteelErr>(raw)(input)
    }
}

use crate::error::SteelErr;
use crate::nodes::{Call, Symbol};

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
            return s.name.to_string();
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
    let (input, _) = multispace0(input)?;
    let (input, symbol) = symbol_raw(input)?;
    let id = context.add(symbol);
    Ok((input, id))
}

fn is_operator_char(c: char) -> bool {
    "~!@$%^&*-+=|?/\\:".contains(c)
}

type Precedence = i32;
const MIN_PRECENDENCE: Precedence = Precedence::MIN;
const MAX_PRECENDENCE: Precedence = Precedence::MAX;
const INIT_PRECENDENCE: Precedence = MIN_PRECENDENCE;
const MUL_PRECENDENCE: Precedence = 2;
const PLUS_PRECENDENCE: Precedence = 1;

pub fn operator_raw<'source>(
    input: &'source str,
    min_prec: &mut Precedence, // TODO: reject operators of the wrong prec...
) -> SResult<'source, Symbol<'source>> {
    let (input, name) = take_while_m_n(1, 3, is_operator_char)(input)?;
    let precendence = match name {
        "+" => PLUS_PRECENDENCE,
        "*" => MUL_PRECENDENCE,
        _ => MAX_PRECENDENCE,
    };
    if precendence < *min_prec {
        return Err(nom::Err::Error(SteelErr::PrecedenceError { precendence }));
    }
    eprintln!("RAISE PREC FROM {:?} TO {:?} ", min_prec, precendence);
    *min_prec = precendence; // otherwise raise the precendence.
    Ok((input, Symbol::operator(name)))
}

pub fn operator<
    'source,
    ID,
    E: Into<SteelErr>,
    C: ParserStorage<'source, ID, Symbol<'source>, E>,
>(
    context: &mut C,
    input: &'source str,
    min_prec: &mut Precedence,
) -> SResult<'source, ID> {
    let (input, name) = operator_raw(input, min_prec)?;
    let id = context.add(name);
    Ok((input, id))
}

fn args<'source, C: ParserContext<'source>>(
    context: &mut C,
    input: &'source str,
) -> SResult<'source, Vec<C::ID>>
where
    <C as ParserContext<'source>>::E: Into<SteelErr>,
{
    let (input, _) = tag("(")(input)?;
    let (input, args) = separated_list0(tag(","), |input| {
        let mut ignore_prec = INIT_PRECENDENCE;
        expr(context, input, &mut ignore_prec)
    })(input)?;
    let (input, _) = tag(")")(input)?;
    Ok((input, args))
}

fn led<'source, C: ParserContext<'source>>(
    context: &mut C,
    left: C::ID,
    input: &'source str,
    min_prec: &mut Precedence,
) -> SResult<'source, C::ID>
where
    <C as ParserContext<'source>>::E: Into<SteelErr>,
{
    let (input, op) = operator(context, input, min_prec)?;
    let (input, right) = expr(context, input, min_prec)?;
    let call = context.add(Call::new(op, vec![left, right]));
    Ok((input, call))
}

fn nud<'source, C: ParserContext<'source>>(
    context: &mut C,
    input: &'source str,
) -> SResult<'source, C::ID>
where
    <C as ParserContext<'source>>::E: Into<SteelErr>,
{
    if let Ok((input, _)) = tag("(")(input) {
        let mut ignore_prec = INIT_PRECENDENCE;
        let (input, wrapped) = expr(context, input, &mut ignore_prec)?;
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
    let mut ignore_prec = INIT_PRECENDENCE;
    if let Ok((input, op)) = operator(context, input, &mut ignore_prec) {
        // Unified calling syntax for a prefix operator
        if let Ok((input, args)) = args(context, input) {
            // Function call
            if let Ok(op) = context.get_symbol_mut(op) {
                // TODO: WHAT!?
                op.is_operator = false;
            }
            let call = context.add(Call::new(op, args));
            return Ok((input, call));
        }
        // Prefix operator
        let mut ignore_prec = INIT_PRECENDENCE;
        let (input, right) = expr(context, input, &mut ignore_prec)?;
        let call = context.add(Call::new(op, vec![right]));
        return Ok((input, call));
    }
    // Otherwise expect a number
    number_i64(context, input)
}

pub fn expr<'context, 'source: 'context, C: ParserContext<'source>>(
    context: &'context mut C,
    input: &'source str,
    min_prec: &mut Precedence,
) -> SResult<'source, C::ID>
where
    <C as ParserContext<'source>>::E: Into<SteelErr>,
{
    let mut state = nud(context, input)?;
    loop {
        let update = led(context, state.1, state.0, min_prec);
        match update {
            Ok(new_state) => {
                state = new_state;
            }
            Err(_) => return Ok(state),
        }
    }
}

pub fn program<'context, 'source: 'context, C: ParserContext<'source>>(
    context: &'context mut C,
    input: &'source str,
) -> SResult<'source, C::ID>
where
    <C as ParserContext<'source>>::E: Into<SteelErr>,
{
    let mut min_prec = INIT_PRECENDENCE;
    let (mut input, mut left) = expr(context, input, &mut min_prec)?;
    loop {
        let (new_input, _) = multispace0(input)?;
        input = new_input;
        if input.is_empty() {
            return Ok((input, left));
        }
        // dbg!(&state);
        match led(context, left, input, &mut min_prec) {
            Ok((new_input, new_left)) => {
                input = new_input;
                left = new_left;
            }
            Err(nom::Err::Error(SteelErr::PrecedenceError { precendence })) => {
                // go back down and try again
                eprintln!("TRY AGAIN {:?}", precendence);
                min_prec = precendence;
            }
            Err(e) => return Err(e),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::assertions::assert_err_is;

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
        let mut prec = INIT_PRECENDENCE;
        assert_eq!(
            operator_raw("||", &mut prec).unwrap(),
            ("", Symbol::operator("||"))
        );
        let mut prec = INIT_PRECENDENCE;
        assert_eq!(
            operator_raw("+", &mut prec).unwrap(),
            ("", Symbol::operator("+"))
        );
        let mut prec = MUL_PRECENDENCE;
        assert_eq!(
            operator_raw("*", &mut prec).unwrap(),
            ("", Symbol::operator("*"))
        );
        let mut prec = PLUS_PRECENDENCE;
        assert_eq!(
            operator_raw("*", &mut prec).unwrap(),
            ("", Symbol::operator("*"))
        );
        let mut prec = INIT_PRECENDENCE;
        assert_eq!(
            operator_raw("*", &mut prec).unwrap(),
            ("", Symbol::operator("*"))
        );
    }

    #[test]
    fn parse_operator_in_wrong_precendence() {
        let mut prec = MUL_PRECENDENCE;
        assert_err_is(
            operator_raw("+", &mut prec),
            &format!(
                "Parsing Error: PrecedenceError {{ precendence: {} }}",
                PLUS_PRECENDENCE
            ),
        );
    }

    #[test]
    fn parse_non_operator() {
        let mut prec = INIT_PRECENDENCE;
        assert_err_is(
            operator_raw("#lol", &mut prec),
            "Parsing Error: Error { input: \"#lol\", code: TakeWhileMN }",
        );
        let mut prec = INIT_PRECENDENCE;
        assert_err_is(
            operator_raw("123", &mut prec),
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
