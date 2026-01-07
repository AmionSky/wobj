use std::path::PathBuf;
use std::str::FromStr;

use winnow::ascii::{dec_uint, float, line_ending, till_line_ending};
use winnow::combinator::{alt, delimited, opt, preceded, repeat, separated, seq};
use winnow::error::StrContext;
use winnow::stream::AsChar;
use winnow::token::{take_till, take_while};
use winnow::{BStr, Result, prelude::*};

const KEYWORD_VERTEX_GEOMETRY: &[u8] = "v".as_bytes();
const KEYWORD_VERTEX_TEXTURE: &[u8] = "vt".as_bytes();
const KEYWORD_VERTEX_NORMAL: &[u8] = "vn".as_bytes();
const KEYWORD_FACE: &[u8] = "f".as_bytes();
const KEYWORD_GROUP: &[u8] = "g".as_bytes();
const KEYWORD_SMOOTHING: &[u8] = "s".as_bytes();
const KEYWORD_OBJECT: &[u8] = "o".as_bytes();
const KEYWORD_MTLLIB: &[u8] = "mtllib".as_bytes();
const KEYWORD_MATERIAL: &[u8] = "usemtl".as_bytes();

use crate::{Face, Obj, Object};

pub(crate) fn parse_obj(input: &mut &BStr) -> Result<Obj> {
    let mut obj = Obj::default();
    let mut current = Object::default();

    while let Ok(key) = keyword.parse_next(input) {
        match key {
            // TODO: Caseless?
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
            KEYWORD_FACE => current.faces.extend(
                parse_face
                    .context(StrContext::Label("element face"))
                    .parse_next(input)?,
            ),
            KEYWORD_GROUP => {
                current.groups = parse_groups.parse_next(input)?;
                check_finalize(&mut current, &mut obj);
            }
            KEYWORD_SMOOTHING => {
                current.smoothing = parse_smoothing.parse_next(input)?;
                check_finalize(&mut current, &mut obj);
            }
            KEYWORD_OBJECT => {
                let name = parse_string.parse_next(input)?;
                match name.is_empty() {
                    true => current.name = None,
                    false => current.name = Some(name),
                }
                check_finalize(&mut current, &mut obj);
            }
            KEYWORD_MTLLIB => {
                current.mtllib = Some(parse_mtllib.parse_next(input)?);
                check_finalize(&mut current, &mut obj);
            }
            KEYWORD_MATERIAL => {
                let material = parse_string.parse_next(input)?;
                match material.is_empty() {
                    true => current.material = None,
                    false => current.material = Some(material),
                }
                check_finalize(&mut current, &mut obj);
            }
            _ => {
                // Ignoring unknown keywords
                till_line_ending.parse_next(input)?;
            }
        }
        (till_line_ending, line_ending).void().parse_next(input)?;
    }

    if !current.faces.is_empty() {
        obj.objects.push(current);
    }

    Ok(obj)
}

fn check_finalize(current: &mut Object, obj: &mut Obj) {
    if !current.faces.is_empty() {
        obj.objects.push(current.clone());
        current.faces.clear();
    }
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

fn opt_index(a: Option<usize>, b: Option<usize>, c: Option<usize>) -> Option<[usize; 3]> {
    match (a, b, c) {
        (Some(t1), Some(t2), Some(t3)) => Some([t1, t2, t3]),
        _ => None,
    }
}

fn parse_face(input: &mut &BStr) -> Result<Vec<Face>> {
    // Note: negative indexes are not supported yet
    let f: Vec<_> = separated(3.., parse_face_vertex, ' ').parse_next(input)?;
    let mut faces: Vec<Face> = Vec::with_capacity(f.len() - 2);

    // Triangulate faces
    for i in 2..f.len() {
        let (a, b, c) = (0, i - 1, i);

        let v = [f[a].0, f[b].0, f[c].0];
        let t = opt_index(f[a].1, f[b].1, f[c].1);
        let n = opt_index(f[a].2, f[b].2, f[c].2);

        faces.push(Face {
            vertex: v,
            normal: n,
            texture: t,
        });
    }

    Ok(faces)
}

/// Returns zeroed index
fn parse_index(input: &mut &BStr) -> Result<usize> {
    dec_uint
        .verify_map(|v: usize| v.checked_add_signed(-1))
        .parse_next(input)
}

/// Returns: (vertex, texture, normal) with 0 index
fn parse_face_vertex(input: &mut &BStr) -> Result<(usize, Option<usize>, Option<usize>)> {
    (
        parse_index,
        opt(preceded('/', parse_index)),
        opt(preceded(alt(("//", "/")), parse_index)),
    )
        .parse_next(input)
}

fn parse_groups(input: &mut &BStr) -> Result<Vec<String>> {
    separated(
        1..,
        take_till(1.., AsChar::is_space).map(|g| String::from_utf8_lossy(g).to_string()),
        ' ',
    )
    .parse_next(input)
}

fn parse_smoothing(input: &mut &BStr) -> Result<u32> {
    alt((dec_uint, "off".value(0))).parse_next(input)
}

fn parse_string(input: &mut &BStr) -> Result<String> {
    till_line_ending
        .map(|g| String::from_utf8_lossy(g).to_string())
        .parse_next(input)
}

fn parse_mtllib(input: &mut &BStr) -> Result<PathBuf> {
    till_line_ending
        .verify_map(|g| {
            str::from_utf8(g)
                .ok()
                .and_then(|s| PathBuf::from_str(s).ok())
        })
        .parse_next(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn face_parsing() {
        let faces = parse_face.parse(BStr::new("1 2 3")).unwrap();
        assert_eq!(
            faces,
            vec![Face {
                vertex: [0, 1, 2],
                normal: None,
                texture: None
            }]
        );

        let faces = parse_face.parse(BStr::new("1/3 2/2 3/1")).unwrap();
        assert_eq!(
            faces,
            vec![Face {
                vertex: [0, 1, 2],
                normal: None,
                texture: Some([2, 1, 0])
            }]
        );

        let faces = parse_face.parse(BStr::new("1/3/6 2/2/5 3/1/4")).unwrap();
        assert_eq!(
            faces,
            vec![Face {
                vertex: [0, 1, 2],
                normal: Some([5, 4, 3]),
                texture: Some([2, 1, 0])
            }]
        );

        let faces = parse_face.parse(BStr::new("1 2 3 4")).unwrap();
        assert_eq!(
            faces,
            vec![
                Face {
                    vertex: [0, 1, 2],
                    normal: None,
                    texture: None
                },
                Face {
                    vertex: [0, 2, 3],
                    normal: None,
                    texture: None
                }
            ]
        );

        let faces = parse_face
            .parse(BStr::new("3/6/9 4/7/8 5/8/7 6/9/6"))
            .unwrap();
        assert_eq!(
            faces,
            vec![
                Face {
                    vertex: [2, 3, 4],
                    normal: Some([8, 7, 6]),
                    texture: Some([5, 6, 7])
                },
                Face {
                    vertex: [2, 4, 5],
                    normal: Some([8, 6, 5]),
                    texture: Some([5, 7, 8])
                }
            ]
        );

        assert!(parse_face.parse(BStr::new(" ")).is_err());
        assert!(parse_face.parse(BStr::new("1")).is_err());
        assert!(parse_face.parse(BStr::new("1 2")).is_err());
    }

    #[test]
    fn face_vertex_parsing() {
        // Check correct
        assert_eq!(
            parse_face_vertex.parse(BStr::new("1")), //
            Ok((0, None, None))
        );
        assert_eq!(
            parse_face_vertex.parse(BStr::new("1/2")),
            Ok((0, Some(1), None))
        );
        assert_eq!(
            parse_face_vertex.parse(BStr::new("1/2/3")),
            Ok((0, Some(1), Some(2)))
        );
        assert_eq!(
            parse_face_vertex.parse(BStr::new("1//3")),
            Ok((0, None, Some(2)))
        );

        // Check incorrect
        assert!(parse_face_vertex.parse(BStr::new("1/")).is_err());
        assert!(parse_face_vertex.parse(BStr::new("1//")).is_err());
        assert!(parse_face_vertex.parse(BStr::new("/2/")).is_err());
        assert!(parse_face_vertex.parse(BStr::new("//3")).is_err());
        assert!(parse_face_vertex.parse(BStr::new("/2/3")).is_err());
        assert!(parse_face_vertex.parse(BStr::new("//")).is_err());
        assert!(parse_face_vertex.parse(BStr::new("/")).is_err());
        assert!(parse_face_vertex.parse(BStr::new("")).is_err());
        assert!(parse_face_vertex.parse(BStr::new("1/e/3")).is_err());
        assert!(parse_face_vertex.parse(BStr::new("1/2/e")).is_err());
        assert!(parse_face_vertex.parse(BStr::new("1//e")).is_err());
        assert!(parse_face_vertex.parse(BStr::new("1/e")).is_err());
        assert!(parse_face_vertex.parse(BStr::new("1.0")).is_err());
        assert!(parse_face_vertex.parse(BStr::new("0")).is_err());
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
