use std::num::NonZero;

use winnow::ascii::{dec_int, dec_uint, float, space1};
use winnow::combinator::{alt, delimited, opt, preceded, separated, separated_pair, seq};
use winnow::error::ContextError;
use winnow::{BStr, Result, prelude::*};

use super::{Faces, MeshData, Obj, VertexData};
use crate::util::{
    description, expected, ignoreable, label, parse_path, parse_string, to_next_line, word,
};

pub(crate) fn parse_obj(input: &mut &BStr) -> Result<Obj> {
    let mut data = VertexData::default();
    let mut meshes = Vec::new();
    let mut current = MeshData::default();

    // Check if the current mesh needs to be added to meshes
    let mut check = |current: &mut MeshData| {
        if current.faces.is_some() {
            meshes.push(current.clone());
            current.faces = None;
        }
    };

    while let Ok(key) = keyword(input) {
        match key {
            b"v" => data.vertex.push(
                parse_float3
                    .context(label("vertex geometry"))
                    .parse_next(input)?,
            ),
            b"vn" => data.normal.push(
                parse_float3
                    .context(label("vertex normal"))
                    .parse_next(input)?,
            ),
            b"vt" => data.texture.push(
                parse_vt
                    .context(label("vertex texture"))
                    .parse_next(input)?,
            ),
            b"f" => match &mut current.faces {
                Some(faces) => match faces {
                    Faces::V(list) => list.push(parse_face_v(&data).parse_next(input)?),
                    Faces::VT(list) => list.push(parse_face_vt(&data).parse_next(input)?),
                    Faces::VN(list) => list.push(parse_face_vn(&data).parse_next(input)?),
                    Faces::VTN(list) => list.push(parse_face_vtn(&data).parse_next(input)?),
                },
                None => current.faces = Some(parse_face_start(input, &data)?),
            },
            b"g" => {
                check(&mut current);
                current.groups = parse_groups
                    .context(label("attribute group"))
                    .parse_next(input)?;
            }
            b"s" => {
                check(&mut current);
                current.smoothing = parse_smoothing
                    .context(label("attribute smoothing group"))
                    .parse_next(input)?;
            }
            b"o" => {
                check(&mut current);
                current.name = Some(
                    parse_string
                        .context(label("attribute object name"))
                        .parse_next(input)?,
                );
            }
            b"mtllib" => {
                check(&mut current);
                current.mtllib = Some(
                    parse_path
                        .context(label("attribute mtllib"))
                        .parse_next(input)?,
                );
            }
            b"usemtl" => {
                check(&mut current);
                current.material = Some(
                    parse_string
                        .context(label("attribute material"))
                        .parse_next(input)?,
                );
            }
            _ => (), // Skip unknown keywords
        }

        to_next_line(input)?;
    }

    if current.faces.is_some() {
        meshes.push(current);
    }

    Ok(Obj { data, meshes })
}

fn keyword<'a>(input: &mut &'a BStr) -> Result<&'a [u8]> {
    delimited(ignoreable, word, space1)
        .context(label("keyword"))
        .parse_next(input)
}

fn parse_float3(input: &mut &BStr) -> Result<[f32; 3]> {
    (float, space1, float, space1, float)
        .map(|(x, _, y, _, z)| [x, y, z])
        .context(expected("x y z"))
        .context(description("3 coordinates"))
        .parse_next(input)
}

fn parse_vt(input: &mut &BStr) -> Result<[f32; 2]> {
    (float, opt(preceded(space1, float)))
        .map(|(u, v)| [u, v.unwrap_or(0.0)])
        .context(expected("u v"))
        .context(description("texture coordinates"))
        .parse_next(input)
}

fn parse_face_start(input: &mut &BStr, data: &VertexData) -> Result<Faces> {
    alt((
        parse_face_vtn(data).map(|v: Vec<_>| Faces::VTN(vec![v])),
        parse_face_vn(data).map(|v: Vec<_>| Faces::VN(vec![v])),
        parse_face_vt(data).map(|v: Vec<_>| Faces::VT(vec![v])),
        parse_face_v(data).map(|v: Vec<_>| Faces::V(vec![v])),
    ))
    .parse_next(input)
}

fn calc_index(i: NonZero<isize>, len: usize) -> usize {
    match i.is_positive() {
        // Get the zeroed index
        true => (i.get() - 1) as usize,
        // Calculate from relative index
        false => len.saturating_add_signed(i.get()),
    }
}

fn parse_index<'a>(len: usize) -> impl Parser<&'a BStr, usize, ContextError> {
    dec_int
        .verify_map(NonZero::new)
        .map(move |i| calc_index(i, len))
}

fn parse_face_v<'a>(data: &VertexData) -> impl Parser<&'a BStr, Vec<usize>, ContextError> {
    separated(3.., parse_index(data.vertex.len()), space1)
        .context(expected("v1 v2 v3 ..."))
        .context(description("3 or more vertex indicies"))
}

fn parse_face_vt<'a>(
    data: &VertexData,
) -> impl Parser<&'a BStr, Vec<(usize, usize)>, ContextError> {
    separated(
        3..,
        separated_pair(
            parse_index(data.vertex.len()),
            '/',
            parse_index(data.texture.len()),
        ),
        space1,
    )
    .context(expected("v1/t1 v2/t2 v3/t3 ..."))
    .context(description("3 or more vertex and texture indicies"))
}

fn parse_face_vn<'a>(
    data: &VertexData,
) -> impl Parser<&'a BStr, Vec<(usize, usize)>, ContextError> {
    separated(
        3..,
        separated_pair(
            parse_index(data.vertex.len()),
            "//",
            parse_index(data.normal.len()),
        ),
        space1,
    )
    .context(expected("v1//n1 v2//n2 v3//n3 ..."))
    .context(description("3 or more vertex and normal indicies"))
}

fn parse_face_vtn<'a>(
    data: &VertexData,
) -> impl Parser<&'a BStr, Vec<(usize, usize, usize)>, ContextError> {
    separated(
        3..,
        seq!(
            parse_index(data.vertex.len()),
            _: '/',
            parse_index(data.texture.len()),
            _: '/',
            parse_index(data.normal.len()),
        ),
        space1,
    )
    .context(expected("v1/t1/n1 v2/t2/n2 v3/t3/n3 ..."))
    .context(description("3 or more vertex, texture and normal indicies"))
}

fn parse_groups(input: &mut &BStr) -> Result<Vec<String>> {
    separated(
        1..,
        word.try_map(|s: &[_]| String::from_utf8(s.to_vec())),
        space1,
    )
    .context(expected("group1 group2 ..."))
    .context(description("list of group names"))
    .parse_next(input)
}

fn parse_smoothing(input: &mut &BStr) -> Result<u32> {
    alt((dec_uint, "off".value(0)))
        .context(description("smoothing group number or 'off'"))
        .parse_next(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn face_parsing() {
        let mut data = VertexData::default();
        data.vertex.append(&mut [[1.0, 2.0, 3.0]].repeat(3));
        data.normal.append(&mut [[1.0, 2.0, 3.0]].repeat(3));
        data.texture.append(&mut [[1.0, 2.0]].repeat(3));

        assert_eq!(
            parse_face_start(&mut BStr::new("1 2 3"), &data).unwrap(),
            Faces::V(vec!(vec!(0, 1, 2)))
        );
        assert_eq!(
            parse_face_start(&mut BStr::new("1/3 2/2 3/1"), &data).unwrap(),
            Faces::VT(vec!(vec!((0, 2), (1, 1), (2, 0))))
        );
        assert_eq!(
            parse_face_start(&mut BStr::new("1//3 2//2 3//1"), &data).unwrap(),
            Faces::VN(vec!(vec!((0, 2), (1, 1), (2, 0))))
        );
        assert_eq!(
            parse_face_start(&mut BStr::new("1/2/3 2/3/1 3/1/2"), &data).unwrap(),
            Faces::VTN(vec!(vec!((0, 1, 2), (1, 2, 0), (2, 0, 1))))
        );
        assert_eq!(
            parse_face_start(&mut BStr::new("-1 -2 -3"), &data).unwrap(),
            Faces::V(vec!(vec!(2, 1, 0)))
        );

        assert!(parse_face_start(&mut BStr::new(" "), &data).is_err());
        assert!(parse_face_start(&mut BStr::new("1"), &data).is_err());
        assert!(parse_face_start(&mut BStr::new("1 2"), &data).is_err());
        assert!(parse_face_start(&mut BStr::new("1 e 2"), &data).is_err());
        assert!(parse_face_start(&mut BStr::new("1 2 /3"), &data).is_err());
        assert!(parse_face_start(&mut BStr::new("1/2 2 3/2"), &data).is_err());

        assert_ne!(
            parse_face_start(&mut BStr::new("1 2 3"), &data).unwrap(),
            Faces::V(vec!(vec!(2, 1, 0)))
        );
    }

    #[test]
    fn group_parsing() {
        assert_eq!(
            parse_groups.parse(BStr::new("group1")),
            Ok(vec!["group1".to_string()])
        );
        assert_eq!(
            parse_groups.parse(BStr::new("group1 group2 group3")),
            Ok(vec![
                "group1".to_string(),
                "group2".to_string(),
                "group3".to_string()
            ])
        );

        assert!(parse_groups.parse(BStr::new(" ")).is_err());
    }
}
