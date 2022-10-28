use crate::compiler_context::{CompilerContext, NodeStore};
use crate::error::SteelErr;
use crate::nodes::{Call, Operator, Symbol};
use nom::{
    branch::alt,
    bytes::complete::{tag as raw_tag, take_while, take_while1},
    character::complete::{alpha1, multispace0},
    combinator::map_res,
    multi::separated_list0,
    sequence::tuple,
};

type SResult<'a, T> = std::result::Result<(&'a str, T), nom::Err<SteelErr>>;

fn tag(raw: &str) -> impl Fn(&str) -> SResult<&str> + '_ {
    |input: &str| {
        let input = multispace0::<&str, SteelErr>(input)?.0;
        raw_tag::<&str, &str, SteelErr>(raw)(input)
            .map_err(|e| SteelErr::ErrorExpected(Box::new(e.into()), raw.to_string()).into())
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

pub fn number_i64<'source, ID, E: Into<SteelErr>, C: NodeStore<ID, i64, E>>(
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
pub fn binding<'source, ID, E: Into<SteelErr>, C: NodeStore<ID, Symbol, E>>(
    _context: &mut C,
    input: &'source str,
) -> SResult<'source, String> {
    let (og_input, _) = multispace0(input)?;
    let (input, (head, tail)) = tuple((identifier_head, identifier_tail))(og_input)?;
    let name = &og_input[0..head.len() + tail.len()];
    let (input, _) = multispace0(input)?;
    let (input, _) = tag("=")(input)?;
    Ok((input, name.to_string()))
}

pub fn symbol<'source, ID, E: Into<SteelErr>, C: NodeStore<ID, Symbol, E>>(
    context: &mut C,
    input: &'source str,
) -> SResult<'source, ID> {
    let (input, _) = multispace0(input)?;
    let (input, symbol) = symbol_raw(input)?;
    let id = context.add(symbol);
    Ok((input, id))
}

type Precedence = i32;
const MIN_PRECENDENCE: Precedence = Precedence::MIN;
const INIT_PRECENDENCE: Precedence = MIN_PRECENDENCE;
const MUL_PRECENDENCE: Precedence = 2;
const PLUS_PRECENDENCE: Precedence = 1;

pub fn operator_raw<'source>(
    input: &'source str,
    min_prec: &mut Precedence, // TODO: reject operators of the wrong prec...
) -> SResult<'source, Operator> {
    let ch = input.chars().next();
    use Operator::*;
    let (precendence, op) = match ch {
        Some('+') => (PLUS_PRECENDENCE, Add),
        Some('-') => (PLUS_PRECENDENCE, Sub),
        Some('*') => (MUL_PRECENDENCE, Mul),
        Some('/') => (MUL_PRECENDENCE, Div),
        _ => {
            return Err(nom::Err::Error(SteelErr::MalformedExpression(
                input.to_string(),
                "operator".to_string(),
            )))
        }
    };
    if precendence < *min_prec {
        return Err(nom::Err::Error(SteelErr::PrecedenceError { precendence }));
    }
    *min_prec = precendence; // otherwise raise the precendence.
    Ok((&input[1..], op))
}

pub fn operator<'source, ID, E: Into<SteelErr>, C: NodeStore<ID, Operator, E>>(
    context: &mut C,
    input: &'source str,
    min_prec: &mut Precedence,
) -> SResult<'source, ID> {
    let (input, operator) = operator_raw(input, min_prec)?;
    let id = context.add(operator);
    Ok((input, id))
}

type ArgBindings<ID> = Vec<(String, ID)>;

fn args<'source, C: CompilerContext>(
    context: &mut C,
    input: &'source str,
) -> SResult<'source, ArgBindings<C::ID>>
where
    <C as CompilerContext>::E: Into<SteelErr>,
{
    let (input, _) = tag("(")(input)?;
    let mut arg_num = 0;
    let (input, args) = separated_list0(tag(","), |input| {
        let (input, name) = if let Ok((input, sym)) = binding(context, input) {
            (input, sym)
        } else {
            let res = (input, format!("arg_{}", arg_num));
            arg_num += 1;
            res
        };
        let mut ignore_prec = INIT_PRECENDENCE;
        let (input, value) = expr(context, input, &mut ignore_prec)?;
        Ok((input, (name, value)))
    })(input)?;
    let (input, _) = tag(")")(input)?;
    Ok((input, args))
}

fn led<'source, C: CompilerContext>(
    context: &mut C,
    left: C::ID,
    input: &'source str,
    min_prec: &mut Precedence,
) -> SResult<'source, C::ID>
where
    <C as CompilerContext>::E: Into<SteelErr>,
{
    // Unified calling syntax e.g. <expr>(...args...).
    if let Ok((input, args)) = args(context, input) {
        let call = context.add(Call::new(left, args));
        return Ok((input, call));
    }
    let (input, op) = operator(context, input, min_prec)?;
    let (input, right) = expr(context, input, min_prec)?;
    let call = context.add(Call::new(
        op,
        vec![("arg_0".to_string(), left), ("arg_1".to_string(), right)],
    ));
    Ok((input, call))
}

fn nud<'source, C: CompilerContext>(context: &mut C, input: &'source str) -> SResult<'source, C::ID>
where
    <C as CompilerContext>::E: Into<SteelErr>,
{
    if let Ok((input, _)) = tag("(")(input) {
        let mut ignore_prec = INIT_PRECENDENCE;
        let (input, wrapped) = expr(context, input, &mut ignore_prec)?;
        // TODO: handle larger expressions (before ')' )
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
        // Prefix operator e.g. -3.
        let mut ignore_prec = INIT_PRECENDENCE;
        if let Ok((input, right)) = expr(context, input, &mut ignore_prec) {
            let z = context.add(0);
            let call = context.add(Call::new(
                op,
                vec![("arg_0".to_string(), z), ("arg_1".to_string(), right)],
            ));
            return Ok((input, call));
        }
        // Operator expression e.g. f=+.
        return Ok((input, op));
    }
    // Otherwise expect a number
    if let Ok(res) = number_i64(context, input) {
        return Ok(res);
    }
    if input.is_empty() {
        Err(nom::Err::Error(SteelErr::UnexpectedEndOfInput))
    } else {
        let mut report = input;
        let mut lines = input.split('\n');
        if let Some(first_line) = lines.next() {
            report = first_line;
        }
        Err(nom::Err::Error(SteelErr::MalformedExpression(
            report.to_string(),
            "the end of the input".to_string(),
        )))
    }
}

pub fn expr<'context, 'source: 'context, C: CompilerContext>(
    context: &'context mut C,
    input: &'source str,
    min_prec: &mut Precedence,
) -> SResult<'source, C::ID>
where
    <C as CompilerContext>::E: Into<SteelErr>,
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

pub fn program<'context, 'source: 'context, C: CompilerContext>(
    context: &'context mut C,
    input: &'source str,
) -> SResult<'source, C::ID>
where
    <C as CompilerContext>::E: Into<SteelErr>,
{
    let mut min_prec = INIT_PRECENDENCE;
    let (mut input, mut left) = expr(context, input, &mut min_prec)?;
    loop {
        let (new_input, _) = multispace0(input)?;
        input = new_input;
        if input.is_empty() {
            return Ok((input, left));
        }
        match led(context, left, input, &mut min_prec) {
            Ok((new_input, new_left)) => {
                input = new_input;
                left = new_left;
            }
            Err(nom::Err::Error(SteelErr::PrecedenceError { precendence })) => {
                // go back down and try again
                min_prec = precendence;
            }
            Err(e) => return Err(e),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::assert_err_is;

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
        assert_err_is!(
            symbol_raw("#lol"),
            "Parsing Error: Failed in Alt while parsing #lol\nand Failed in Tag while parsing #lol expected _"
        );
        assert_err_is!(
            symbol_raw("123"),
            "Parsing Error: Failed in Alt while parsing 123\nand Failed in Tag while parsing 123 expected _"
        );
    }

    #[test]
    fn parse_operator() {
        let mut prec = INIT_PRECENDENCE;
        assert_eq!(operator_raw("+", &mut prec).unwrap(), ("", Operator::Add));
        let mut prec = MUL_PRECENDENCE;
        assert_eq!(operator_raw("*", &mut prec).unwrap(), ("", Operator::Mul));
        let mut prec = PLUS_PRECENDENCE;
        assert_eq!(operator_raw("*", &mut prec).unwrap(), ("", Operator::Mul));
        let mut prec = INIT_PRECENDENCE;
        assert_eq!(operator_raw("*", &mut prec).unwrap(), ("", Operator::Mul));
    }

    #[test]
    fn parse_operator_in_wrong_precendence() {
        let mut prec = MUL_PRECENDENCE;
        assert_err_is!(
            operator_raw("+", &mut prec),
            format!(
                "Parsing Error: Unexpected operator due to max precendence setting ({})",
                PLUS_PRECENDENCE
            ),
        );
    }

    #[test]
    fn parse_non_operator() {
        let mut prec = INIT_PRECENDENCE;
        assert_err_is!(
            operator_raw("#lol", &mut prec),
            "Parsing Error: Expected operator, found \"#lol\""
        );
        let mut prec = INIT_PRECENDENCE;
        assert_err_is!(
            operator_raw("123", &mut prec),
            "Parsing Error: Expected operator, found \"123\""
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
        assert_err_is!(
            number_i64_raw("#lol"),
            "Parsing Error: Failed in TakeWhile1 while parsing #lol"
        );
        assert_err_is!(
            number_i64_raw("||"),
            "Parsing Error: Failed in TakeWhile1 while parsing ||"
        );
        assert_err_is!(
            number_i64_raw("e2"),
            "Parsing Error: Failed in TakeWhile1 while parsing e2"
        );
    }
}
