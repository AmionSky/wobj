mod mesh;
mod parser;

pub use mesh::*;

use winnow::{BStr, Parser};

use crate::WobjError;

#[derive(Debug)]
pub struct Obj {
    data: VertexData,
    meshes: Vec<MeshData>,
}

impl Obj {
    pub fn parse(bytes: &[u8]) -> Result<Self, WobjError> {
        parser::parse_obj
            .parse(BStr::new(bytes))
            .map_err(WobjError::from)
    }

    pub fn meshes<'obj>(&'obj self) -> Vec<ObjMesh<'obj>> {
        self.meshes
            .iter()
            .map(|m| ObjMesh::new(&self.data, m))
            .collect()
    }

    pub fn vertices(&self) -> &[[f32; 3]] {
        &self.data.vertex
    }

    pub fn normals(&self) -> &[[f32; 3]] {
        &self.data.normal
    }

    pub fn uvs(&self) -> &[[f32; 2]] {
        &self.data.texture
    }
}

#[derive(Debug, Default, Clone)]
struct VertexData {
    vertex: Vec<[f32; 3]>,
    normal: Vec<[f32; 3]>,
    texture: Vec<[f32; 2]>,
}

#[derive(Debug, Default, Clone)]
struct MeshData {
    name: Option<String>,
    material: Option<String>,
    mtllib: Option<std::path::PathBuf>,
    groups: Vec<String>,
    smoothing: u32,
    faces: Option<Faces>,
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
