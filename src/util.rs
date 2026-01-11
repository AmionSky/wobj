use winnow::ascii::{line_ending, till_line_ending};
use winnow::combinator::opt;
use winnow::error::{StrContext, StrContextValue};
use winnow::token::take_till;
use winnow::{BStr, Parser, Result};

/// Go to next line
pub fn to_next_line(input: &mut &BStr) -> Result<()> {
    (till_line_ending, opt(line_ending))
        .void()
        .parse_next(input)
}

pub fn word<'a>(input: &mut &'a BStr) -> Result<&'a [u8]> {
    take_till(1.., (' ', '\t', '\r', '\n')).parse_next(input)
}

pub fn label(text: &'static str) -> StrContext {
    StrContext::Label(text)
}

pub fn expected(text: &'static str) -> StrContext {
    StrContext::Expected(StrContextValue::StringLiteral(text))
}

pub fn description(text: &'static str) -> StrContext {
    StrContext::Expected(StrContextValue::Description(text))
}

