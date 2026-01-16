use std::path::PathBuf;

use ahash::HashMap;
use winnow::ascii::{dec_uint, float, space1, till_line_ending};
use winnow::combinator::{
    alt, delimited, dispatch, fail, opt, preceded, repeat, separated_pair, terminated,
};
use winnow::error::{ContextError, FromExternalError};
use winnow::{BStr, Result, prelude::*};

use super::{Channel, ColorValue, MapOption, Material, Refl, TextureMap};
use crate::util::{expected, ignoreable, label, parse_path, to_next_line, word};

pub(crate) fn parse_mtl(input: &mut &BStr) -> Result<HashMap<String, Material>> {
    let mut materials = HashMap::default();

    while let Ok(name) = parse_name(input) {
        let material = parse_material(input)?;
        materials.insert(name, material);
    }

    Ok(materials)
}

fn parse_material(input: &mut &BStr) -> Result<Material> {
    let mut material = Material::default();

    while let Ok(key) = keyword(input) {
        match key.to_ascii_lowercase().as_slice() {
            b"ka" => {
                material.ambient = Some(
                    parse_color_value
                        .context(label("ambient (Ka)"))
                        .parse_next(input)?,
                )
            }
            b"kd" => {
                material.diffuse = Some(
                    parse_color_value
                        .context(label("diffuse (Kd)"))
                        .parse_next(input)?,
                )
            }
            b"ks" => {
                material.specular = Some(
                    parse_color_value
                        .context(label("specular (Ks)"))
                        .parse_next(input)?,
                )
            }
            b"tf" => {
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
            b"tr" => {
                material.dissolve = Some(
                    1.0 - float::<_, f32, _>
                        .context(label("dissolve (Tr)"))
                        .parse_next(input)?,
                )
            }
            b"ns" => {
                material.exponent = Some(
                    float
                        .context(label("specular exponent (Ns)"))
                        .parse_next(input)?,
                )
            }
            b"ni" => {
                material.density = Some(
                    float
                        .context(label("optical density (Ni)"))
                        .parse_next(input)?,
                )
            }
            b"map_ka" => {
                material.ambient_map = Some(
                    parse_map
                        .context(label("ambient texture (map_Ka)"))
                        .parse_next(input)?,
                )
            }
            b"map_kd" => {
                material.diffuse_map = Some(
                    parse_map
                        .context(label("diffuse texture (map_Kd)"))
                        .parse_next(input)?,
                )
            }
            b"map_ks" => {
                material.specular_map = Some(
                    parse_map
                        .context(label("specular texture (map_Ks)"))
                        .parse_next(input)?,
                )
            }
            b"map_ns" => {
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
            b"refl" => {
                let (shape, map) = parse_relf
                    .context(label("reflection map (refl)"))
                    .parse_next(input)?;

                match shape {
                    b"sphere" => {
                        material.reflection = Some(Refl::Sphere(map));
                    }
                    cube_side => {
                        let side = String::from_utf8(cube_side.to_vec())
                            .map_err(|e| ContextError::from_external_error(input, e))?;

                        if let Some(Refl::Cube(sides)) = &mut material.reflection {
                            sides.insert(side, map);
                        } else {
                            let mut hashmap = HashMap::default();
                            hashmap.insert(side, map);
                            material.reflection = Some(Refl::Cube(hashmap))
                        }
                    }
                }
            }

            b"pr" => {
                material.roughness = Some(
                    float
                        .context(label("PBR roughness (Pr)"))
                        .parse_next(input)?,
                )
            }
            b"pm" => {
                material.metallic = Some(
                    float
                        .context(label("PBR metallic (Pm)"))
                        .parse_next(input)?,
                )
            }
            b"ps" => {
                material.sheen = Some(
                    float
                        .context(label("PBR sheen value (Ps)"))
                        .parse_next(input)?,
                )
            }
            b"pc" => {
                material.cc_thickness = Some(
                    float
                        .context(label("PBR clearcoat thickness (Pc)"))
                        .parse_next(input)?,
                )
            }
            b"pcr" => {
                material.cc_roughness = Some(
                    float
                        .context(label("PBR clearcoat roughness (Pcr)"))
                        .parse_next(input)?,
                )
            }
            b"ke" => {
                material.emissive = Some(
                    parse_color_value
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
            b"map_pr" => {
                material.roughness_map = Some(
                    parse_map
                        .context(label("roughness texture (map_Pr)"))
                        .parse_next(input)?,
                )
            }
            b"map_pm" => {
                material.metallic_map = Some(
                    parse_map
                        .context(label("metallic texture (map_Pm)"))
                        .parse_next(input)?,
                )
            }
            b"map_ps" => {
                material.sheen_map = Some(
                    parse_map
                        .context(label("sheen texture (map_Ps)"))
                        .parse_next(input)?,
                )
            }
            b"map_ke" => {
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
        .context(label("Material name statement"))
        .context(expected("newmtl <name>"))
        .parse_next(input)
}

fn keyword<'a>(input: &mut &'a BStr) -> Result<&'a [u8]> {
    delimited(ignoreable, word, space1)
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
    (
        float,
        opt((preceded(space1, float), preceded(space1, float))),
    )
        .map(|(a, o)| o.map(|(b, c)| (a, b, c)).unwrap_or((a, a, a)))
        .parse_next(input)
}

fn parse_spectral(input: &mut &BStr) -> Result<ColorValue> {
    let (file, factor) = alt((
        // With factor
        separated_pair(word, space1, float),
        // Without factor
        till_line_ending.map(|file| (file, 1.0)),
    ))
    // Convert file str to path
    .try_map(|(file, factor)| str::from_utf8(file).map(|s| (Box::new(PathBuf::from(s)), factor)))
    .parse_next(input)?;

    Ok(ColorValue::Spectral { file, factor })
}

fn parse_map(input: &mut &BStr) -> Result<TextureMap> {
    let options = repeat(0.., terminated(parse_map_option, space1)).parse_next(input)?;
    let path = parse_path.parse_next(input)?;
    Ok(TextureMap::new(path, options))
}

fn parse_map_option(input: &mut &BStr) -> Result<MapOption> {
    dispatch! { delimited('-', word, space1);
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

fn parse_float3oo(input: &mut &BStr) -> Result<(f32, Option<f32>, Option<f32>)> {
    (
        float,
        opt(preceded(space1, float)),
        opt(preceded(space1, float)),
    )
        .parse_next(input)
}

fn parse_uv_offset(input: &mut &BStr) -> Result<MapOption> {
    parse_float3oo
        .map(|(u, v, w)| MapOption::Offset(u, v.unwrap_or(0.0), w.unwrap_or(0.0)))
        .parse_next(input)
}

fn parse_uv_scale(input: &mut &BStr) -> Result<MapOption> {
    parse_float3oo
        .map(|(u, v, w)| MapOption::Scale(u, v.unwrap_or(1.0), w.unwrap_or(1.0)))
        .parse_next(input)
}

fn parse_uv_turbulance(input: &mut &BStr) -> Result<MapOption> {
    parse_float3oo
        .map(|(u, v, w)| MapOption::Turbulence(u, v.unwrap_or(0.0), w.unwrap_or(0.0)))
        .parse_next(input)
}

fn parse_relf<'a>(input: &mut &'a BStr) -> Result<(&'a [u8], TextureMap)> {
    let shape = alt((
        delimited("-type ", "sphere", space1),
        delimited("-type cube_", word, space1),
    ))
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
