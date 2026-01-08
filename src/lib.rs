mod obj;

use std::path::{Path, PathBuf};

use obj::parse_obj;
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
            Err(error) => { eprintln!("{error}"); Err("error".into()) },
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

    pub fn faces(&self) -> Vec<[usize; 3]> {
        self.faces.iter().map(|f| f.vertex).collect()
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Face {
    vertex: [usize; 3],
    normal: Option<[usize; 3]>,
    texture: Option<[usize; 3]>,
}
