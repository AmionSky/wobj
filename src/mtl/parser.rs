use std::path::PathBuf;

use winnow::ascii::{float, till_line_ending};
use winnow::combinator::{alt, delimited, opt, preceded, repeat, separated_pair, terminated};
use winnow::{BStr, Result, prelude::*};

use super::{ColorValue, Material};
use crate::util::{expected, label, to_next_line, word};

pub(crate) fn parse_mtl(input: &mut &BStr) -> Result<Vec<Material>> {
    repeat(0.., parse_material).parse_next(input)
}

fn parse_material(input: &mut &BStr) -> Result<Material> {
    let mut material = Material::new(parse_name.parse_next(input)?);

    while let Ok(key) = keyword(input) {
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

        to_next_line(input)?;
    }

    Ok(material)
}

fn parse_name(input: &mut &BStr) -> Result<String> {
    delimited("newmtl ", word, to_next_line)
        .try_map(|s| String::from_utf8(s.to_vec()))
        .context(label("newmtl"))
        .parse_next(input)
}

fn keyword<'a>(input: &mut &'a BStr) -> Result<&'a [u8]> {
    terminated(word, ' ')
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
    let (file, factor) = alt((
        // With factor
        separated_pair(word, ' ', float),
        // Without factor
        till_line_ending.map(|file| (file, 1.0)),
    ))
    // Convert file str to path
    .try_map(|(file, factor)| {
        str::from_utf8(file)
            .map(|s| (PathBuf::from(s), factor))
    })
    .parse_next(input)?;

    Ok(ColorValue::Spectral { file, factor })
}
