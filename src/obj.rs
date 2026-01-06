use winnow::ascii::{float, line_ending, space0, till_line_ending};
use winnow::combinator::{opt, preceded, repeat};
use winnow::error::StrContext;
use winnow::stream::AsChar;
use winnow::token::take_while;
use winnow::{BStr, Result, prelude::*};

const KEYWORD_VERTEX_GEOMETRY: &[u8] = "v".as_bytes();
const KEYWORD_VERTEX_TEXTURE: &[u8] = "vt".as_bytes();
const KEYWORD_VERTEX_NORMAL: &[u8] = "vn".as_bytes();

use crate::{Face, Obj, Object};

pub(crate) fn parse_obj(input: &mut &BStr) -> Result<Obj> {
    let mut obj = Obj::default();

    while let Ok(key) = keyword.parse_next(input) {
        match key {
            KEYWORD_VERTEX_GEOMETRY => obj.vertex.push(
                parse_float3
                    .context(StrContext::Label("vertex geometry"))
                    .parse_next(input)?,
            ),
            KEYWORD_VERTEX_NORMAL => obj.normal.push(
                parse_float3
                    .context(StrContext::Label("vertex normal"))
                    .parse_next(input)?,
            ),
            KEYWORD_VERTEX_TEXTURE => obj.texture.push(
                parse_vt
                    .context(StrContext::Label("vertex texture"))
                    .parse_next(input)?,
            ),
            _ => {
                // Ignoring unknown keywords
                till_line_ending.parse_next(input)?;
            }
        }
        (till_line_ending, line_ending).void().parse_next(input)?;
    }

    Ok(obj)
}

fn comment(input: &mut &BStr) -> Result<()> {
    repeat(0.., ('#', till_line_ending, line_ending).void()).parse_next(input)
}

fn keyword<'a>(input: &mut &'a BStr) -> Result<&'a [u8]> {
    preceded(comment, take_while(1.., AsChar::is_alphanum))
        .context(StrContext::Label("keyword"))
        .parse_next(input)
}

fn parse_float(input: &mut &BStr) -> Result<f32> {
    preceded(space0, float).parse_next(input)
}

fn parse_float3(input: &mut &BStr) -> Result<[f32; 3]> {
    let (x, y, z) = (parse_float, parse_float, parse_float).parse_next(input)?;
    Ok([x, y, z])
}

fn parse_vt(input: &mut &BStr) -> Result<[f32; 2]> {
    let (u, v) = (parse_float, opt(parse_float)).parse_next(input)?;
    Ok([u, v.unwrap_or(0.0)])
}
