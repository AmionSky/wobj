use std::path::PathBuf;

use winnow::ascii::{float, line_ending, till_line_ending};
use winnow::combinator::{alt, delimited, opt, preceded, repeat, terminated};
use winnow::error::{StrContext, StrContextValue};
use winnow::stream::AsChar;
use winnow::token::{take_till, take_while};
use winnow::{BStr, Result, prelude::*};

pub struct Material {
    pub name: String,
    /// (Ka) ambient reflectivity
    pub ambient: Option<ColorValue>,
    /// (Kd) diffuse reflectivity
    pub diffuse: Option<ColorValue>,
    /// (Ks) specular reflectivity
    pub specular: Option<ColorValue>,
    /// (Tf) transmission filter
    pub filter: Option<ColorValue>,
}

impl Material {
    pub fn new(name: String) -> Self {
        Self {
            name,
            ambient: None,
            diffuse: None,
            specular: None,
            filter: None,
        }
    }
}

#[allow(clippy::upper_case_acronyms)]
pub enum ColorValue {
    RGB(f32, f32, f32),
    XYZ(f32, f32, f32),
    Spectral { file: PathBuf, factor: f32 },
}

pub(crate) fn parse_mtl(input: &mut &BStr) -> Result<Vec<Material>> {
    repeat(0.., parse_material).parse_next(input)
}

fn parse_material(input: &mut &BStr) -> Result<Material> {
    let mut material = Material::new(parse_name.parse_next(input)?);

    while let Ok(key) = keyword.parse_next(input) {
        match key {
            b"Ka" => {
                material.ambient = Some(
                    parse_color_value
                        .context(label("ambient"))
                        .parse_next(input)?,
                )
            }
            b"Kd" => {
                material.diffuse = Some(
                    parse_color_value
                        .context(label("diffuse"))
                        .parse_next(input)?,
                )
            }
            b"Ks" => {
                material.specular = Some(
                    parse_color_value
                        .context(label("specular"))
                        .parse_next(input)?,
                )
            }
            b"Tf" => {
                material.filter = Some(
                    parse_color_value
                        .context(label("transmission filter"))
                        .parse_next(input)?,
                )
            }
            _ => (),
        }

        // Go to next line
        to_next_line.parse_next(input)?;
    }

    Ok(material)
}

fn to_next_line(input: &mut &BStr) -> Result<()> {
    (till_line_ending, opt(line_ending))
        .void()
        .parse_next(input)
}

fn parse_name(input: &mut &BStr) -> Result<String> {
    delimited(
        "newmtl ",
        take_till(1.., (' ', '\t', '\r', '\n')),
        (till_line_ending, line_ending),
    )
    .try_map(|s: &[u8]| String::from_utf8(s.to_vec()))
    .context(label("newmtl"))
    .parse_next(input)
}

fn keyword<'a>(input: &mut &'a BStr) -> Result<&'a [u8]> {
    terminated(take_while(1.., AsChar::is_alphanum), ' ')
        .verify(|k: &[_]| k != b"newmtl")
        .context(label("keyword"))
        .parse_next(input)
}

fn parse_color_value(input: &mut &BStr) -> Result<ColorValue> {
    alt((
        preceded("spectral ", parse_spectral),
        preceded("xyz ", parse_xyz),
        parse_rgb,
    ))
    .context(expected("r g b"))
    .context(expected("spectral file.rfl factor"))
    .context(expected("xyz x y z"))
    .parse_next(input)
}

fn parse_trifloat(input: &mut &BStr) -> Result<(f32, f32, f32)> {
    (float, opt((' ', float, ' ', float)))
        .map(|(a, o)| o.map(|(_, b, _, c)| (a, b, c)).unwrap_or((a, a, a)))
        .parse_next(input)
}

fn parse_rgb(input: &mut &BStr) -> Result<ColorValue> {
    parse_trifloat
        .map(|(r, g, b)| ColorValue::RGB(r, g, b))
        .parse_next(input)
}

fn parse_xyz(input: &mut &BStr) -> Result<ColorValue> {
    parse_trifloat
        .map(|(x, y, z)| ColorValue::XYZ(x, y, z))
        .parse_next(input)
}

fn parse_spectral(input: &mut &BStr) -> Result<ColorValue> {
    let (path, factor) = alt((
        // With factor
        (
            take_till(1.., (' ', '\t', '\r', '\n')),
            ' ',
            float,
            take_till(0.., ('\r', '\n')),
        )
            .map(|(path, _, factor, _)| (path, factor)),
        // Without factor
        (till_line_ending).map(|f| (f, 1.0)),
    ))
    // Convert path bytes to str
    .verify_map(|(path, factor)| str::from_utf8(path).map(|s| (s, factor)).ok())
    .parse_next(input)?;

    Ok(ColorValue::Spectral {
        file: PathBuf::from(path),
        factor,
    })
}

fn label(text: &'static str) -> StrContext {
    StrContext::Label(text)
}

fn expected(text: &'static str) -> StrContext {
    StrContext::Expected(StrContextValue::StringLiteral(text))
}
