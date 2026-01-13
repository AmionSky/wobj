use std::num::NonZero;

use winnow::ascii::{dec_int, dec_uint, float};
use winnow::combinator::{alt, delimited, opt, preceded, separated, separated_pair, seq};
use winnow::error::ContextError;
use winnow::{BStr, Result, prelude::*};

use super::{Faces, Obj, Object};
use crate::util::{ignoreable, label, parse_path, parse_string, to_next_line, word};

pub(crate) fn parse_obj(input: &mut &BStr) -> Result<Obj> {
    let mut obj = Obj::default();
    let mut current = Object::default();

    fn check_finalize(current: &mut Object, obj: &mut Obj) {
        if current.faces.is_some() {
            obj.objects.push(current.clone());
            current.faces = None;
        }
    }

    while let Ok(key) = keyword(input) {
        match key {
            b"v" => obj.vertex.push(
                parse_float3
                    .context(label("vertex geometry"))
                    .parse_next(input)?,
            ),
            b"vn" => obj.normal.push(
                parse_float3
                    .context(label("vertex normal"))
                    .parse_next(input)?,
            ),
            b"vt" => obj.texture.push(
                parse_vt
                    .context(label("vertex texture"))
                    .parse_next(input)?,
            ),
            b"f" => match &mut current.faces {
                Some(faces) => match faces {
                    Faces::V(list) => list.push(parse_face_v(obj.vertex.len()).parse_next(input)?),
                    Faces::VT(list) => list.push(
                        parse_face_vt(obj.vertex.len(), obj.texture.len()).parse_next(input)?,
                    ),
                    Faces::VN(list) => list
                        .push(parse_face_vn(obj.vertex.len(), obj.normal.len()).parse_next(input)?),
                    Faces::VTN(list) => list.push(
                        parse_face_vtn(obj.vertex.len(), obj.texture.len(), obj.normal.len())
                            .parse_next(input)?,
                    ),
                },
                None => current.faces = Some(parse_face_start(input, &obj)?),
            },
            b"g" => {
                check_finalize(&mut current, &mut obj);
                current.groups = parse_groups
                    .context(label("attribute group"))
                    .parse_next(input)?;
            }
            b"s" => {
                check_finalize(&mut current, &mut obj);
                current.smoothing = parse_smoothing
                    .context(label("attribute smoothing group"))
                    .parse_next(input)?;
            }
            b"o" => {
                check_finalize(&mut current, &mut obj);
                current.name = Some(
                    parse_string
                        .context(label("attribute object name"))
                        .parse_next(input)?,
                );
            }
            b"mtllib" => {
                check_finalize(&mut current, &mut obj);
                current.mtllib = Some(
                    parse_path
                        .context(label("attribute mtllib"))
                        .parse_next(input)?,
                );
            }
            b"usemtl" => {
                check_finalize(&mut current, &mut obj);
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
        obj.objects.push(current);
    }

    Ok(obj)
}

fn keyword<'a>(input: &mut &'a BStr) -> Result<&'a [u8]> {
    delimited(ignoreable, word, ' ')
        .context(label("keyword"))
        .parse_next(input)
}

fn parse_float3(input: &mut &BStr) -> Result<[f32; 3]> {
    (float, ' ', float, ' ', float)
        .map(|(x, _, y, _, z)| [x, y, z])
        .parse_next(input)
}

fn parse_vt(input: &mut &BStr) -> Result<[f32; 2]> {
    (float, opt(preceded(' ', float)))
        .map(|(u, v)| [u, v.unwrap_or(0.0)])
        .parse_next(input)
}

fn parse_face_start(input: &mut &BStr, obj: &Obj) -> Result<Faces> {
    let vlen = obj.vertex.len();
    let tlen = obj.texture.len();
    let nlen = obj.normal.len();

    alt((
        parse_face_vtn(vlen, tlen, nlen).map(|v: Vec<_>| Faces::VTN(vec![v])),
        parse_face_vn(vlen, nlen).map(|v: Vec<_>| Faces::VN(vec![v])),
        parse_face_vt(vlen, tlen).map(|v: Vec<_>| Faces::VT(vec![v])),
        parse_face_v(vlen).map(|v: Vec<_>| Faces::V(vec![v])),
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

fn parse_face_v<'a>(vlen: usize) -> impl Parser<&'a BStr, Vec<usize>, ContextError> {
    separated(3.., parse_index(vlen), ' ')
}

fn parse_face_vt<'a>(
    vlen: usize,
    tlen: usize,
) -> impl Parser<&'a BStr, Vec<(usize, usize)>, ContextError> {
    separated(
        3..,
        separated_pair(parse_index(vlen), '/', parse_index(tlen)),
        ' ',
    )
}

fn parse_face_vn<'a>(
    vlen: usize,
    nlen: usize,
) -> impl Parser<&'a BStr, Vec<(usize, usize)>, ContextError> {
    separated(
        3..,
        separated_pair(parse_index(vlen), "//", parse_index(nlen)),
        ' ',
    )
}

fn parse_face_vtn<'a>(
    vlen: usize,
    tlen: usize,
    nlen: usize,
) -> impl Parser<&'a BStr, Vec<(usize, usize, usize)>, ContextError> {
    separated(
        3..,
        seq!(
            parse_index(vlen),
            _: '/',
            parse_index(tlen),
            _: '/',
            parse_index(nlen),
        ),
        ' ',
    )
}

fn parse_groups(input: &mut &BStr) -> Result<Vec<String>> {
    separated(
        1..,
        word.try_map(|s: &[_]| String::from_utf8(s.to_vec())),
        ' ',
    )
    .parse_next(input)
}

fn parse_smoothing(input: &mut &BStr) -> Result<u32> {
    alt((dec_uint, "off".value(0))).parse_next(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn face_parsing() {
        let mut obj = Obj::default();
        obj.vertex.append(&mut [[1.0, 2.0, 3.0]].repeat(3));
        obj.normal.append(&mut [[1.0, 2.0, 3.0]].repeat(3));
        obj.texture.append(&mut [[1.0, 2.0]].repeat(3));

        assert_eq!(
            parse_face_start(&mut BStr::new("1 2 3"), &obj).unwrap(),
            Faces::V(vec!(vec!(0, 1, 2)))
        );
        assert_eq!(
            parse_face_start(&mut BStr::new("1/3 2/2 3/1"), &obj).unwrap(),
            Faces::VT(vec!(vec!((0, 2), (1, 1), (2, 0))))
        );
        assert_eq!(
            parse_face_start(&mut BStr::new("1//3 2//2 3//1"), &obj).unwrap(),
            Faces::VN(vec!(vec!((0, 2), (1, 1), (2, 0))))
        );
        assert_eq!(
            parse_face_start(&mut BStr::new("1/2/3 2/3/1 3/1/2"), &obj).unwrap(),
            Faces::VTN(vec!(vec!((0, 1, 2), (1, 2, 0), (2, 0, 1))))
        );
        assert_eq!(
            parse_face_start(&mut BStr::new("-1 -2 -3"), &obj).unwrap(),
            Faces::V(vec!(vec!(2, 1, 0)))
        );

        assert!(parse_face_start(&mut BStr::new(" "), &obj).is_err());
        assert!(parse_face_start(&mut BStr::new("1"), &obj).is_err());
        assert!(parse_face_start(&mut BStr::new("1 2"), &obj).is_err());
        assert!(parse_face_start(&mut BStr::new("1 e 2"), &obj).is_err());
        assert!(parse_face_start(&mut BStr::new("1 2 /3"), &obj).is_err());
        assert!(parse_face_start(&mut BStr::new("1/2 2 3/2"), &obj).is_err());

        assert_ne!(
            parse_face_start(&mut BStr::new("1 2 3"), &obj).unwrap(),
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
