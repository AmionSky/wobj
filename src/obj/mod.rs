mod parser;

use std::path::PathBuf;

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
    pub fn trimesh(&self, faces: &Faces) -> (Indicies, Vertices) {
        use std::hash::Hash;

        use ahash::RandomState;
        use indexmap::IndexSet;

        let mut indices = Vec::with_capacity(faces.len() * 3);

        fn collect<T>(indices: &mut Vec<usize>, faces: &Vec<Vec<T>>) -> IndexSet<T, RandomState>
        where
            T: Clone + Hash + Eq,
        {
            let mut points = IndexSet::with_capacity_and_hasher(indices.len(), RandomState::new());

            // Triangulate faces
            for face in faces {
                // the parser guarantees that there are at least 3 points
                for i in 2..face.len() {
                    let (a, b, c) = (0, i - 1, i);
                    indices.push(points.insert_full(face[a].clone()).0);
                    indices.push(points.insert_full(face[b].clone()).0);
                    indices.push(points.insert_full(face[c].clone()).0);
                }
            }

            points
        }

        // Turn point indexes into vertices
        let vertices = match faces {
            Faces::V(faces) => {
                let points = collect(&mut indices, faces);

                let mut positions = Vec::with_capacity(points.len());
                for v in points {
                    positions.push(self.vertex[v]);
                }

                Vertices {
                    positions,
                    normals: None,
                    uvs: None,
                }
            }
            Faces::VT(faces) => {
                let points = collect(&mut indices, faces);

                let mut positions = Vec::with_capacity(points.len());
                let mut uvs = Vec::with_capacity(points.len());
                for (v, t) in points {
                    positions.push(self.vertex[v]);
                    uvs.push(self.texture[t]);
                }

                Vertices {
                    positions,
                    normals: None,
                    uvs: Some(uvs),
                }
            }
            Faces::VN(faces) => {
                let points = collect(&mut indices, faces);

                let mut positions = Vec::with_capacity(points.len());
                let mut normals = Vec::with_capacity(points.len());
                for (v, n) in points {
                    positions.push(self.vertex[v]);
                    normals.push(self.normal[n]);
                }

                Vertices {
                    positions,
                    normals: Some(normals),
                    uvs: None,
                }
            }
            Faces::VTN(faces) => {
                let points = collect(&mut indices, faces);

                let mut positions = Vec::with_capacity(points.len());
                let mut normals = Vec::with_capacity(points.len());
                let mut uvs = Vec::with_capacity(points.len());
                for (v, t, n) in points {
                    positions.push(self.vertex[v]);
                    normals.push(self.normal[n]);
                    uvs.push(self.texture[t]);
                }

                Vertices {
                    positions,
                    normals: Some(normals),
                    uvs: Some(uvs),
                }
            }
        };

        (Indicies(indices), vertices)
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
    faces: Option<Faces>,
}

impl Object {
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn material(&self) -> Option<&str> {
        self.material.as_deref()
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

    pub fn faces(&self) -> &Faces {
        // self.faces is garanteed by the parser to be valid
        self.faces.as_ref().unwrap()
    }
}

// Faces<Points<Index...>>
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Faces {
    V(Vec<Vec<usize>>),
    VT(Vec<Vec<(usize, usize)>>),
    VN(Vec<Vec<(usize, usize)>>),
    VTN(Vec<Vec<(usize, usize, usize)>>),
}

impl Faces {
    pub const fn len(&self) -> usize {
        match self {
            Faces::V(faces) => faces.len(),
            Faces::VT(faces) => faces.len(),
            Faces::VN(faces) => faces.len(),
            Faces::VTN(faces) => faces.len(),
        }
    }

    pub const fn is_empty(&self) -> bool {
        match self {
            Faces::V(faces) => faces.is_empty(),
            Faces::VT(faces) => faces.is_empty(),
            Faces::VN(faces) => faces.is_empty(),
            Faces::VTN(faces) => faces.is_empty(),
        }
    }
}
