use super::{Faces, MeshData, VertexData};

pub struct ObjMesh<'obj> {
    data: &'obj VertexData,
    mesh: &'obj MeshData,
}

impl<'obj> ObjMesh<'obj> {
    pub(super) fn new(data: &'obj VertexData, mesh: &'obj MeshData) -> Self {
        Self { data, mesh }
    }

    pub fn name(&self) -> Option<&str> {
        self.mesh.name.as_deref()
    }

    pub fn material(&self) -> Option<&str> {
        self.mesh.material.as_deref()
    }

    pub fn mtllib(&self) -> Option<&std::path::Path> {
        self.mesh.mtllib.as_deref()
    }

    pub fn groups(&self) -> &[String] {
        &self.mesh.groups
    }

    pub fn smoothing(&self) -> u32 {
        self.mesh.smoothing
    }

    pub fn faces(&self) -> &Faces {
        // self.faces is garanteed by the parser to be valid
        self.mesh.faces.as_ref().unwrap()
    }

    #[cfg(feature = "trimesh")]
    /// Create a triangulated mesh from faces
    pub fn triangulate(&self) -> (Indicies, Vertices) {
        use std::hash::Hash;

        use ahash::RandomState;
        use indexmap::IndexSet;

        let faces = self.faces();
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
                    positions.push(self.data.vertex[v]);
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
                    positions.push(self.data.vertex[v]);
                    uvs.push(self.data.texture[t]);
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
                    positions.push(self.data.vertex[v]);
                    normals.push(self.data.normal[n]);
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
                    positions.push(self.data.vertex[v]);
                    normals.push(self.data.normal[n]);
                    uvs.push(self.data.texture[t]);
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
