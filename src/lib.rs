mod obj;

use std::path::{Path, PathBuf};

use obj::parse_obj;
use smallvec::SmallVec;
use winnow::{BStr, Parser};

#[derive(Debug, Default)]
pub struct Obj {
    vertex: Vec<[f32; 3]>,
    normal: Vec<[f32; 3]>,
    texture: Vec<[f32; 2]>,
    objects: Vec<Object>,
}

impl Obj {
    pub fn parse<P: AsRef<Path>>(file: P) -> Result<Self, Box<dyn std::error::Error>> {
        let obj = std::fs::read(file).unwrap();

        match parse_obj.parse(BStr::new(&obj)) {
            Ok(obj) => Ok(obj),
            Err(error) => {
                eprintln!("{error}");
                Err("error".into())
            }
        }
    }

    pub fn objects(&self) -> &[Object] {
        &self.objects
    }

    pub fn vertecies(&self) -> &[[f32; 3]] {
        &self.vertex
    }
}

#[derive(Debug, Default, Clone)]
pub struct Object {
    name: Option<String>,
    material: Option<String>,
    mtllib: Option<PathBuf>,
    groups: Vec<String>,
    smoothing: u32,
    faces: Vec<Face>,
}

impl Object {
    pub fn name(&self) -> Option<&String> {
        self.name.as_ref()
    }

    // pub fn faces(&self) -> Vec<[usize; 3]> {
    //     self.faces.iter().map(|f| f.vertex).collect()
    // }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct FacePoint<T> {
    v: T,
    t: Option<T>,
    n: Option<T>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Face(SmallVec<[FacePoint<usize>; 4]>);

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct TriFace {
    vertex: [usize; 3],
    normal: Option<[usize; 3]>,
    texture: Option<[usize; 3]>,
}

// Returns zeroed index
// fn parse_index(input: &mut &BStr) -> Result<usize> {
//     dec_uint
//         .verify_map(|v: usize| v.checked_add_signed(-1))
//         .parse_next(input)
// }

// fn opt_index(a: Option<usize>, b: Option<usize>, c: Option<usize>) -> Option<[usize; 3]> {
//         match (a, b, c) {
//             (Some(t1), Some(t2), Some(t3)) => Some([t1, t2, t3]),
//             _ => None,
//         }
//     }

//     // Triangulate faces
//     for i in 2..f.len() {
//         let (a, b, c) = (0, i - 1, i);

//         let v = [f[a].0, f[b].0, f[c].0];
//         let t = opt_index(f[a].1, f[b].1, f[c].1);
//         let n = opt_index(f[a].2, f[b].2, f[c].2);

//         faces.push(Face {
//             vertex: v,
//             normal: n,
//             texture: t,
//         });
//     }
