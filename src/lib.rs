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
    pub fn parse<P: AsRef<Path>>(file: P) {
        let obj = std::fs::read(file).unwrap();

        match parse_obj.parse(BStr::new(&obj)) {
            Ok(obj) => println!("OBJ: {obj:?}"),
            Err(error) => eprintln!("Error: {error}"),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct Object {
    name: Option<String>,
    material: Option<String>,
    mtllib: Option<PathBuf>,
    groups: Vec<String>,
    smoothing_group: u32,
    faces: Vec<Face>,
}

#[derive(Debug, Default, Clone)]
pub struct Face {
    vertex: [usize; 3],
    normal: [usize; 3],
    texture: [usize; 3],
}
