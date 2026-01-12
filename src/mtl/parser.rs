use std::collections::HashMap;
use std::path::PathBuf;

use winnow::ascii::{dec_uint, float, till_line_ending};
use winnow::combinator::{
    alt, delimited, dispatch, fail, opt, preceded, repeat, separated_pair, terminated,
};
use winnow::{BStr, Result, prelude::*};

use super::{Channel, ColorValue, MapOption, Material, TextureMap};
use crate::util::{expected, ignoreable, label, parse_path, to_next_line, word};

pub(crate) fn parse_mtl(input: &mut &BStr) -> Result<HashMap<String, Material>> {
    let mut materials = HashMap::new();

    while let Ok(name) = parse_name(input) {
        let material = parse_material(input)?;
        materials.insert(name, material);
    }

    Ok(materials)
}

fn parse_material(input: &mut &BStr) -> Result<Material> {
    let mut material = Material::default();

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
            b"illum" => {
                material.illum = Some(
                    parse_illum
                        .context(label("illumination model"))
                        .parse_next(input)?,
                )
            }
            b"d" => {
                material.dissolve = Some(
                    parse_dissolve
                        .context(label("dissolve"))
                        .parse_next(input)?,
                )
            }
            b"Ns" => {
                material.exponent = Some(
                    float
                        .context(label("specular exponent"))
                        .parse_next(input)?,
                )
            }
            b"Ni" => {
                material.density = Some(
                    float
                        .context(label("index of refraction"))
                        .parse_next(input)?,
                )
            }
            b"map_Ka" => {
                material.ambient_map = Some(
                    parse_map
                        .context(label("ambient texture"))
                        .parse_next(input)?,
                )
            }
            b"map_Kd" => {
                material.diffuse_map = Some(
                    parse_map
                        .context(label("diffuse texture"))
                        .parse_next(input)?,
                )
            }
            b"map_Ks" => {
                material.specular_map = Some(
                    parse_map
                        .context(label("specular texture"))
                        .parse_next(input)?,
                )
            }
            b"map_Ns" => {
                material.exponent_map = Some(
                    parse_map
                        .context(label("specular exponent texture"))
                        .parse_next(input)?,
                )
            }
            b"map_d" => {
                material.dissolve_map = Some(
                    parse_map
                        .context(label("dissolve texture"))
                        .parse_next(input)?,
                )
            }
            b"decal" => {
                material.decal_map = Some(
                    parse_map
                        .context(label("decal texture"))
                        .parse_next(input)?,
                )
            }
            b"disp" => {
                material.disp_map = Some(
                    parse_map
                        .context(label("displacement texture"))
                        .parse_next(input)?,
                )
            }
            b"bump" => {
                material.bump_map = Some(
                    parse_map
                        .context(label("bump-map texture"))
                        .parse_next(input)?,
                )
            }
            b"map_aat" => {
                material.aa_map = Some(
                    parse_on_off
                        .context(label("texture anti-aliasing"))
                        .parse_next(input)?,
                )
            }
            b"relf" => material.relf.push(
                parse_relf
                    .context(label("reflection map"))
                    .parse_next(input)?,
            ),
            _ => (),
        }

        to_next_line(input)?;
    }

    Ok(material)
}

fn parse_name(input: &mut &BStr) -> Result<String> {
    delimited(ignoreable, preceded("newmtl ", word), to_next_line)
        .try_map(|s| String::from_utf8(s.to_vec()))
        .context(label("newmtl"))
        .parse_next(input)
}

fn keyword<'a>(input: &mut &'a BStr) -> Result<&'a [u8]> {
    delimited(ignoreable, word, ' ')
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
    .try_map(|(file, factor)| str::from_utf8(file).map(|s| (PathBuf::from(s), factor)))
    .parse_next(input)?;

    Ok(ColorValue::Spectral { file, factor })
}

fn parse_illum(input: &mut &BStr) -> Result<u8> {
    alt((dec_uint, preceded("illum_", dec_uint))).parse_next(input)
}

fn parse_dissolve(input: &mut &BStr) -> Result<(f32, bool)> {
    alt((
        preceded("-halo ", float).map(|f| (f, true)),
        float.map(|f| (f, false)),
    ))
    .parse_next(input)
}

fn parse_map(input: &mut &BStr) -> Result<TextureMap> {
    let options = repeat(0.., terminated(parse_map_option, ' ')).parse_next(input)?;
    let path = parse_path.parse_next(input)?;
    Ok(TextureMap { path, options })
}

fn parse_map_option(input: &mut &BStr) -> Result<MapOption> {
    dispatch! { delimited('-', word, ' ');
        b"blendu" => parse_on_off.map(MapOption::BlendU),
        b"blendv" => parse_on_off.map(MapOption::BlendV),
        b"bm" => float.map(MapOption::BumpMultiplier),
        b"boost" => float.map(MapOption::Boost),
        b"cc" => parse_on_off.map(MapOption::ColorCorrection),
        b"clamp" => parse_on_off.map(MapOption::Clamp),
        b"imfchan" => parse_channel.map(MapOption::Channel),
        b"mm" => (float, float).map(|(b, g)| MapOption::MM(b, g)),
        b"o" => parse_uv_offset,
        b"s" => parse_uv_scale,
        b"t" => parse_uv_turbulance,
        b"texres" => dec_uint.map(MapOption::Resolution),
        _ => fail,
    }
    .parse_next(input)
}

fn parse_on_off(input: &mut &BStr) -> Result<bool> {
    alt(("on".value(true), "off".value(false))).parse_next(input)
}

fn parse_channel(input: &mut &BStr) -> Result<Channel> {
    alt((
        'r'.value(Channel::Red),
        'g'.value(Channel::Green),
        'b'.value(Channel::Blue),
        'm'.value(Channel::Matte),
        'l'.value(Channel::Luminance),
        'z'.value(Channel::ZDepth),
    ))
    .parse_next(input)
}

fn parse_uv_offset(input: &mut &BStr) -> Result<MapOption> {
    (float, opt(preceded(' ', float)), opt(preceded(' ', float)))
        .map(|(u, v, w)| MapOption::Offset(u, v.unwrap_or(0.0), w.unwrap_or(0.0)))
        .parse_next(input)
}

fn parse_uv_scale(input: &mut &BStr) -> Result<MapOption> {
    (float, opt(preceded(' ', float)), opt(preceded(' ', float)))
        .map(|(u, v, w)| MapOption::Scale(u, v.unwrap_or(1.0), w.unwrap_or(1.0)))
        .parse_next(input)
}

fn parse_uv_turbulance(input: &mut &BStr) -> Result<MapOption> {
    (float, opt(preceded(' ', float)), opt(preceded(' ', float)))
        .map(|(u, v, w)| MapOption::Turbulence(u, v.unwrap_or(0.0), w.unwrap_or(0.0)))
        .parse_next(input)
}

fn parse_relf(input: &mut &BStr) -> Result<(String, TextureMap)> {
    let shape = delimited("-type ", word, ' ')
        .try_map(|s| String::from_utf8(s.to_vec()))
        .parse_next(input)?;
    let map = parse_map.parse_next(input)?;
    Ok((shape, map))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_parsing() {
        assert_eq!(parse_name(&mut BStr::new("newmtl Mat")).unwrap(), "Mat");
        assert_eq!(parse_name(&mut BStr::new("\nnewmtl Mat")).unwrap(), "Mat");
        assert_eq!(parse_name(&mut BStr::new("#C\nnewmtl Mat")).unwrap(), "Mat");
        assert!(parse_name(&mut BStr::new("invalid newmtl")).is_err())
    }
}
