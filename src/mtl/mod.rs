mod parser;

use std::path::{Path, PathBuf};

use winnow::{BStr, Parser};

pub fn parse_mtl<P: AsRef<Path>>(file: P) -> Result<Vec<Material>, Box<dyn std::error::Error>> {
    let mtl = std::fs::read(file).unwrap();

    match parser::parse_mtl.parse(BStr::new(&mtl)) {
        Ok(mtl) => Ok(mtl),
        Err(error) => {
            eprintln!("{error}");
            Err("error".into())
        }
    }
}

pub struct Material {
    pub name: String,
    /// (Ka) ambient reflectivity
    pub ambient: Option<ColorValue>,
    /// (Kd) diffuse reflectivity
    pub diffuse: Option<ColorValue>,
    /// (Ks) specular reflectivity
    pub specular: Option<ColorValue>,
    /// (Tf) transmission filter
    pub filter: Option<ColorValue>,
}

impl Material {
    pub fn new(name: String) -> Self {
        Self {
            name,
            ambient: None,
            diffuse: None,
            specular: None,
            filter: None,
        }
    }
}

#[allow(clippy::upper_case_acronyms)]
pub enum ColorValue {
    RGB(f32, f32, f32),
    XYZ(f32, f32, f32),
    Spectral { file: PathBuf, factor: f32 },
}
