use std::path::PathBuf;

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

/// Parses a non-empty UTF-8 string
pub fn parse_string(input: &mut &BStr) -> Result<String> {
    till_line_ending
        .verify(|s: &[_]| !s.is_empty())
        .try_map(|s: &[_]| String::from_utf8(s.to_vec()))
        .context(description("UTF-8 string"))
        .parse_next(input)
}

/// Parses a non-empty filesystem path
pub fn parse_path(input: &mut &BStr) -> Result<PathBuf> {
    parse_string
        .map(PathBuf::from)
        .context(description("filesystem path"))
        .parse_next(input)
}
