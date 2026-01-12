mod parser;

use std::path::PathBuf;

use smallvec::SmallVec;
use winnow::{BStr, Parser};

use crate::WobjError;

#[derive(Debug, Default)]
pub struct Obj {
    vertex: Vec<[f32; 3]>,
    normal: Vec<[f32; 3]>,
    texture: Vec<[f32; 2]>,
    objects: Vec<Object>,
}

impl Obj {
    pub fn parse(bytes: &[u8]) -> Result<Self, WobjError> {
        parser::parse_obj
            .parse(BStr::new(bytes))
            .map_err(WobjError::from)
    }

    pub fn objects(&self) -> &[Object] {
        &self.objects
    }

    pub fn vertices(&self) -> &[[f32; 3]] {
        &self.vertex
    }

    pub fn normals(&self) -> &[[f32; 3]] {
        &self.normal
    }

    pub fn uvs(&self) -> &[[f32; 2]] {
        &self.texture
    }

    #[cfg(feature = "trimesh")]
    /// Create a triangulated mesh from faces
    pub fn trimesh(&self, faces: &[Face]) -> (Indicies, Vertices) {
        use ahash::RandomState;
        use indexmap::IndexSet;

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

#[cfg(feature = "trimesh")]
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Indicies(pub Vec<usize>);

#[cfg(feature = "trimesh")]
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

    pub fn material(&self) -> Option<&String> {
        self.material.as_ref()
    }

    pub fn mtllib(&self) -> Option<&PathBuf> {
        self.mtllib.as_ref()
    }

    pub fn groups(&self) -> &[String] {
        &self.groups
    }

    pub fn smoothing(&self) -> u32 {
        self.smoothing
    }

    pub fn faces(&self) -> &[Face] {
        &self.faces
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Face(SmallVec<[FacePoint<usize>; 3]>);

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
struct FacePoint<T> {
    v: T,
    t: Option<T>,
    n: Option<T>,
}
