use std::num::NonZero;
use std::path::PathBuf;

use smallvec::SmallVec;
use winnow::ascii::{dec_int, dec_uint, float, line_ending, till_line_ending};
use winnow::combinator::{alt, delimited, opt, preceded, repeat, separated, seq};
use winnow::error::{StrContext, StrContextValue};
use winnow::stream::AsChar;
use winnow::token::{take_till, take_while};
use winnow::{BStr, Result, prelude::*};

use crate::{Face, FacePoint, Obj, Object};

pub(crate) fn parse_obj(input: &mut &BStr) -> Result<Obj> {
    let mut obj = Obj::default();
    let mut current = Object::default();

    fn check_finalize(current: &mut Object, obj: &mut Obj) {
        if !current.faces.is_empty() {
            obj.objects.push(current.clone());
            current.faces.clear();
        }
    }

    while let Ok(key) = keyword.parse_next(input) {
        match key {
            b"v" => obj.vertex.push(
                parse_float3
                    .context(StrContext::Label("vertex geometry"))
                    .parse_next(input)?,
            ),
            b"vn" => obj.normal.push(
                parse_float3
                    .context(StrContext::Label("vertex normal"))
                    .parse_next(input)?,
            ),
            b"vt" => obj.texture.push(
                parse_vt
                    .context(StrContext::Label("vertex texture"))
                    .parse_next(input)?,
            ),
            b"f" => current.faces.push(parse_face(input, &obj)?),
            b"g" => {
                check_finalize(&mut current, &mut obj);
                current.groups = parse_groups
                    .context(StrContext::Label("attribute group"))
                    .parse_next(input)?;
            }
            b"s" => {
                check_finalize(&mut current, &mut obj);
                current.smoothing = parse_smoothing
                    .context(StrContext::Label("attribute smoothing group"))
                    .parse_next(input)?;
            }
            b"o" => {
                check_finalize(&mut current, &mut obj);
                current.name = Some(
                    parse_string
                        .context(StrContext::Label("attribute object name"))
                        .parse_next(input)?,
                );
            }
            b"mtllib" => {
                check_finalize(&mut current, &mut obj);
                current.mtllib = Some(
                    parse_path
                        .context(StrContext::Label("attribute mtllib"))
                        .parse_next(input)?,
                );
            }
            b"usemtl" => {
                check_finalize(&mut current, &mut obj);
                current.material = Some(
                    parse_string
                        .context(StrContext::Label("attribute material"))
                        .parse_next(input)?,
                );
            }
            _ => (), // Skip unknown keywords
        }

        // Go to next line
        (till_line_ending, opt(line_ending))
            .void()
            .parse_next(input)?;
    }

    if !current.faces.is_empty() {
        obj.objects.push(current);
    }

    Ok(obj)
}

fn comment(input: &mut &BStr) -> Result<()> {
    repeat(0.., ('#', till_line_ending, line_ending).void()).parse_next(input)
}

fn keyword<'a>(input: &mut &'a BStr) -> Result<&'a [u8]> {
    delimited(comment, take_while(1.., AsChar::is_alphanum), ' ')
        .context(StrContext::Label("keyword"))
        .parse_next(input)
}

fn parse_float3(input: &mut &BStr) -> Result<[f32; 3]> {
    let (x, _, y, _, z) = seq!(float, ' ', float, ' ', float).parse_next(input)?;
    Ok([x, y, z])
}

fn parse_vt(input: &mut &BStr) -> Result<[f32; 2]> {
    let (u, v) = (float, opt(preceded(' ', float))).parse_next(input)?;
    Ok([u, v.unwrap_or(0.0)])
}

fn parse_face(input: &mut &BStr, obj: &Obj) -> Result<Face> {
    let points: Vec<_> = separated(3.., parse_face_point, ' ')
        .context(StrContext::Label("element face"))
        .parse_next(input)?;

    fn calc_index(i: NonZero<isize>, len: usize) -> usize {
        match i.is_positive() {
            // Get the zeroed index
            true => (i.get() - 1) as usize,
            // Calculate from relative index
            false => len.saturating_add_signed(i.get()),
        }
    }

    let face: SmallVec<[_; 4]> = points
        .into_iter()
        .map(|FacePoint { v, t, n }| {
            let v = calc_index(v, obj.vertex.len());
            let t = t.map(|i| calc_index(i, obj.texture.len()));
            let n = n.map(|i| calc_index(i, obj.normal.len()));
            FacePoint { v, t, n }
        })
        .collect();

    Ok(Face(face))
}

fn parse_index(input: &mut &BStr) -> Result<NonZero<isize>> {
    dec_int.verify_map(NonZero::new).parse_next(input)
}

fn parse_face_point(input: &mut &BStr) -> Result<FacePoint<NonZero<isize>>> {
    let (v, t, n) = (
        parse_index,
        opt(preceded('/', parse_index)),
        opt(preceded(alt(("//", "/")), parse_index)),
    )
        .parse_next(input)?;

    Ok(FacePoint { v, t, n })
}

fn parse_groups(input: &mut &BStr) -> Result<Vec<String>> {
    separated(
        1..,
        take_till(1.., (' ', '\r', '\n')).map(|g| String::from_utf8_lossy(g).to_string()),
        ' ',
    )
    .parse_next(input)
}

fn parse_smoothing(input: &mut &BStr) -> Result<u32> {
    alt((dec_uint, "off".value(0))).parse_next(input)
}

/// Parses a non-empty UTF-8 string
fn parse_string(input: &mut &BStr) -> Result<String> {
    till_line_ending
        .verify(|s: &[_]| !s.is_empty())
        .verify_map(|s: &[_]| String::from_utf8(s.to_vec()).ok())
        .context(StrContext::Expected(StrContextValue::Description(
            "UTF-8 string",
        )))
        .parse_next(input)
}

/// Parses a non-empty filesystem path
fn parse_path(input: &mut &BStr) -> Result<PathBuf> {
    parse_string
        .map(PathBuf::from)
        .context(StrContext::Expected(StrContextValue::Description(
            "filesystem path",
        )))
        .parse_next(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use smallvec::smallvec;

    impl<T> FacePoint<T> {
        fn v(v: T) -> Self {
            Self {
                v,
                t: None,
                n: None,
            }
        }

        fn vt(v: T, t: T) -> Self {
            Self {
                v,
                t: Some(t),
                n: None,
            }
        }

        fn vn(v: T, n: T) -> Self {
            Self {
                v,
                t: None,
                n: Some(n),
            }
        }

        fn vtn(v: T, t: T, n: T) -> Self {
            Self {
                v,
                t: Some(t),
                n: Some(n),
            }
        }
    }

    #[test]
    fn face_parsing() {
        let mut obj = Obj::default();
        obj.vertex.append(&mut [[1.0, 2.0, 3.0]].repeat(3));
        obj.normal.append(&mut [[1.0, 2.0, 3.0]].repeat(3));
        obj.texture.append(&mut [[1.0, 2.0]].repeat(3));

        assert_eq!(
            parse_face(&mut BStr::new("1 2 3"), &obj).unwrap(),
            Face(smallvec!(FacePoint::v(0), FacePoint::v(1), FacePoint::v(2)))
        );
        assert_eq!(
            parse_face(&mut BStr::new("1/3 2/2 3/1"), &obj).unwrap(),
            Face(smallvec!(
                FacePoint::vt(0, 2),
                FacePoint::vt(1, 1),
                FacePoint::vt(2, 0)
            ))
        );
        assert_eq!(
            parse_face(&mut BStr::new("1//3 2//2 3//1"), &obj).unwrap(),
            Face(smallvec!(
                FacePoint::vn(0, 2),
                FacePoint::vn(1, 1),
                FacePoint::vn(2, 0)
            ))
        );
        assert_eq!(
            parse_face(&mut BStr::new("1/2/3 2/3/1 3/1/2"), &obj).unwrap(),
            Face(smallvec!(
                FacePoint::vtn(0, 1, 2),
                FacePoint::vtn(1, 2, 0),
                FacePoint::vtn(2, 0, 1)
            ))
        );
        assert_eq!(
            parse_face(&mut BStr::new("-1 -2 -3"), &obj).unwrap(),
            Face(smallvec!(FacePoint::v(2), FacePoint::v(1), FacePoint::v(0)))
        );

        assert!(parse_face(&mut BStr::new(" "), &obj).is_err());
        assert!(parse_face(&mut BStr::new("1"), &obj).is_err());
        assert!(parse_face(&mut BStr::new("1 2"), &obj).is_err());

        assert_ne!(
            parse_face(&mut BStr::new("1 2 3"), &obj).unwrap(),
            Face(smallvec!(FacePoint::v(2), FacePoint::v(1), FacePoint::v(0)))
        );
    }

    #[test]
    fn face_point_parsing() {
        // Check correct
        assert_eq!(
            parse_face_point.parse(BStr::new("1")).unwrap(),
            FacePoint::v(NonZero::new(1).unwrap())
        );
        assert_eq!(
            parse_face_point.parse(BStr::new("1/2")).unwrap(),
            FacePoint::vt(NonZero::new(1).unwrap(), NonZero::new(2).unwrap())
        );
        assert_eq!(
            parse_face_point.parse(BStr::new("1//3")).unwrap(),
            FacePoint::vn(NonZero::new(1).unwrap(), NonZero::new(3).unwrap())
        );
        assert_eq!(
            parse_face_point.parse(BStr::new("1/2/3")).unwrap(),
            FacePoint::vtn(
                NonZero::new(1).unwrap(),
                NonZero::new(2).unwrap(),
                NonZero::new(3).unwrap()
            )
        );

        // Check incorrect
        assert!(parse_face_point.parse(BStr::new("1/")).is_err());
        assert!(parse_face_point.parse(BStr::new("1//")).is_err());
        assert!(parse_face_point.parse(BStr::new("/2/")).is_err());
        assert!(parse_face_point.parse(BStr::new("//3")).is_err());
        assert!(parse_face_point.parse(BStr::new("/2/3")).is_err());
        assert!(parse_face_point.parse(BStr::new("//")).is_err());
        assert!(parse_face_point.parse(BStr::new("/")).is_err());
        assert!(parse_face_point.parse(BStr::new("")).is_err());
        assert!(parse_face_point.parse(BStr::new("1/e/3")).is_err());
        assert!(parse_face_point.parse(BStr::new("1/2/e")).is_err());
        assert!(parse_face_point.parse(BStr::new("1//e")).is_err());
        assert!(parse_face_point.parse(BStr::new("1/e")).is_err());
        assert!(parse_face_point.parse(BStr::new("1.0")).is_err());
        assert!(parse_face_point.parse(BStr::new("0")).is_err());
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
