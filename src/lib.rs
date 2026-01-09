mod obj;

use std::path::{Path, PathBuf};

use ahash::RandomState;
use indexmap::IndexSet;
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

    pub fn mesh(&self, faces: &[Face]) -> (Indicies, Vertices) {
        let mut indices = Vec::with_capacity(faces.len() * 3);
        let mut points = IndexSet::with_capacity_and_hasher(faces.len() * 3, RandomState::new());

        // de-duplicate the points
        let mut insert = |point: FacePoint<usize>| {
            let (index, _) = points.insert_full(point);
            indices.push(index);
        };

        // Triangulate faces
        for Face(face) in faces {
            // the parser guarantees that there are at least 3 points
            for i in 2..face.len() {
                let (a, b, c) = (0, i - 1, i);
                insert(face[a].clone());
                insert(face[b].clone());
                insert(face[c].clone());
            }
        }

        let count = points.len();
        let has_normals = points.first().and_then(|p| p.n).is_some();
        let has_texture = points.first().and_then(|p| p.t).is_some();

        let mut v = Vec::with_capacity(points.len());
        let mut n = Vec::with_capacity(if has_normals { count } else { 0 });
        let mut t = Vec::with_capacity(if has_texture { count } else { 0 });

        for point in points.into_iter() {
            v.push(self.vertex[point.v]);

            if has_normals && let Some(index) = point.n {
                n.push(self.normal[index]);
            }

            if has_texture && let Some(index) = point.t {
                t.push(self.texture[index]);
            }
        }

        (
            Indicies(indices),
            Vertices {
                positions: v,
                normals: if n.len() == count { Some(n) } else { None },
                uvs: if t.len() == count { Some(t) } else { None },
            },
        )
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Indicies(pub Vec<usize>);

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Vertices {
    pub positions: Vec<[f32; 3]>,
    pub normals: Option<Vec<[f32; 3]>>,
    pub uvs: Option<Vec<[f32; 2]>>,
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

    pub fn faces(&self) -> &[Face] {
        &self.faces
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
struct FacePoint<T> {
    v: T,
    t: Option<T>,
    n: Option<T>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Face(SmallVec<[FacePoint<usize>; 4]>);
