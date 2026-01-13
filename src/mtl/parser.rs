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
                        .context(label("ambient (Ka)"))
                        .parse_next(input)?,
                )
            }
            b"Kd" => {
                material.diffuse = Some(
                    parse_color_value
                        .context(label("diffuse (Kd)"))
                        .parse_next(input)?,
                )
            }
            b"Ks" => {
                material.specular = Some(
                    parse_color_value
                        .context(label("specular (Ks)"))
                        .parse_next(input)?,
                )
            }
            b"Tf" => {
                material.filter = Some(
                    parse_color_value
                        .context(label("transmission filter (Tf)"))
                        .parse_next(input)?,
                )
            }
            b"illum" => {
                material.illum = Some(
                    dec_uint
                        .context(label("illumination model (illum)"))
                        .parse_next(input)?,
                )
            }
            b"d" => {
                material.halo = opt("-halo ").parse_next(input)?.is_some();
                material.dissolve = Some(float.context(label("dissolve (d)")).parse_next(input)?);
            }
            b"Tr" => {
                material.dissolve = Some(
                    1.0 - float::<_, f32, _>
                        .context(label("dissolve (Tr)"))
                        .parse_next(input)?,
                )
            }
            b"Ns" => {
                material.exponent = Some(
                    float
                        .context(label("specular exponent (Ns)"))
                        .parse_next(input)?,
                )
            }
            b"Ni" => {
                material.density = Some(
                    float
                        .context(label("optical density (Ni)"))
                        .parse_next(input)?,
                )
            }
            b"map_Ka" => {
                material.ambient_map = Some(
                    parse_map
                        .context(label("ambient texture (map_Ka)"))
                        .parse_next(input)?,
                )
            }
            b"map_Kd" => {
                material.diffuse_map = Some(
                    parse_map
                        .context(label("diffuse texture (map_Kd)"))
                        .parse_next(input)?,
                )
            }
            b"map_Ks" => {
                material.specular_map = Some(
                    parse_map
                        .context(label("specular texture (map_Ks)"))
                        .parse_next(input)?,
                )
            }
            b"map_Ns" => {
                material.exponent_map = Some(
                    parse_map
                        .context(label("specular exponent texture (map_Ns)"))
                        .parse_next(input)?,
                )
            }
            b"map_d" => {
                material.dissolve_map = Some(
                    parse_map
                        .context(label("dissolve texture (map_d)"))
                        .parse_next(input)?,
                )
            }
            b"decal" => {
                material.decal_map = Some(
                    parse_map
                        .context(label("decal texture (decal)"))
                        .parse_next(input)?,
                )
            }
            b"disp" => {
                material.disp_map = Some(
                    parse_map
                        .context(label("displacement texture (disp)"))
                        .parse_next(input)?,
                )
            }
            b"bump" | b"map_bump" => {
                material.bump_map = Some(
                    parse_map
                        .context(label("bump texture (bump/map_bump)"))
                        .parse_next(input)?,
                )
            }
            b"map_aat" => {
                material.anti_aliasing = parse_on_off
                    .context(label("anti-aliasing (map_aat)"))
                    .parse_next(input)?
            }
            b"relf" => material.relf.push(
                parse_relf
                    .context(label("reflection map (relf)"))
                    .parse_next(input)?,
            ),
            b"Pr" => {
                material.roughness = Some(
                    float
                        .context(label("PBR roughness (Pr)"))
                        .parse_next(input)?,
                )
            }
            b"Pm" => {
                material.metallic = Some(
                    float
                        .context(label("PBR metallic (Pm)"))
                        .parse_next(input)?,
                )
            }
            b"Ps" => {
                material.sheen = Some(
                    float
                        .context(label("PBR sheen value (Ps)"))
                        .parse_next(input)?,
                )
            }
            b"Pc" => {
                material.cc_thickness = Some(
                    float
                        .context(label("PBR clearcoat thickness (Pc)"))
                        .parse_next(input)?,
                )
            }
            b"Pcr" => {
                material.cc_roughness = Some(
                    float
                        .context(label("PBR clearcoat roughness (Pcr)"))
                        .parse_next(input)?,
                )
            }
            b"Ke" => {
                material.emissive = Some(
                    float
                        .context(label("PBR emissive (Ke)"))
                        .parse_next(input)?,
                )
            }
            b"aniso" => {
                material.anisotropy = Some(
                    float
                        .context(label("PBR anisotropy (aniso)"))
                        .parse_next(input)?,
                )
            }
            b"anisor" => {
                material.anisotropy_rotation = Some(
                    float
                        .context(label("PBR anisotropy rotation (anisor)"))
                        .parse_next(input)?,
                )
            }
            b"map_Pr" => {
                material.roughness_map = Some(
                    parse_map
                        .context(label("roughness texture (map_Pr)"))
                        .parse_next(input)?,
                )
            }
            b"map_Pm" => {
                material.metallic_map = Some(
                    parse_map
                        .context(label("metallic texture (map_Pm)"))
                        .parse_next(input)?,
                )
            }
            b"map_Ps" => {
                material.sheen_map = Some(
                    parse_map
                        .context(label("sheen texture (map_Ps)"))
                        .parse_next(input)?,
                )
            }
            b"map_Ke" => {
                material.emissive_map = Some(
                    parse_map
                        .context(label("emissive texture (map_Ke)"))
                        .parse_next(input)?,
                )
            }
            b"norm" => {
                material.normal_map = Some(
                    parse_map
                        .context(label("normal texture (norm)"))
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
        parse_float3o.map(ColorValue::rgb),
        preceded("spectral ", parse_spectral),
        preceded("xyz ", parse_float3o.map(ColorValue::xyz)),
    ))
    .context(expected("r g b"))
    .context(expected("spectral file.rfl factor"))
    .context(expected("xyz x y z"))
    .parse_next(input)
}

fn parse_float3o(input: &mut &BStr) -> Result<(f32, f32, f32)> {
    (float, opt((preceded(' ', float), preceded(' ', float))))
        .map(|(a, o)| o.map(|(b, c)| (a, b, c)).unwrap_or((a, a, a)))
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
    .try_map(|(file, factor)| str::from_utf8(file).map(|s| (Box::new(PathBuf::from(s)), factor)))
    .parse_next(input)?;

    Ok(ColorValue::Spectral { file, factor })
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
