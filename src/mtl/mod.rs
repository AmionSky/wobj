mod parser;

use std::path::PathBuf;

use ahash::HashMap;
use winnow::{BStr, Parser};

use crate::WobjError;

/// Wavefront MTL data
#[derive(Debug, Clone)]
pub struct Mtl(HashMap<String, Material>);

impl Mtl {
    /// Parses MTL file data
    pub fn parse(bytes: &[u8]) -> Result<Self, WobjError> {
        parser::parse_mtl
            .parse(BStr::new(bytes))
            .map_err(WobjError::from)
            .map(Self::new)
    }

    fn new(materials: HashMap<String, Material>) -> Self {
        Self(materials)
    }

    /// Gets the material with the specified name
    pub fn get(&self, name: &str) -> Option<&Material> {
        self.0.get(name)
    }

    /// Gets the underlying HashMap
    pub fn inner(&self) -> &HashMap<String, Material> {
        &self.0
    }

    /// Takes the underlying HashMap
    pub fn into_inner(self) -> HashMap<String, Material> {
        self.0
    }
}

/// Wavefront MTL material data
#[derive(Debug, Default, Clone)]
pub struct Material {
    /// (Ka) ambient reflectivity
    pub ambient: Option<ColorValue>,
    /// (Kd) diffuse reflectivity
    pub diffuse: Option<ColorValue>,
    /// (Ks) specular reflectivity
    pub specular: Option<ColorValue>,
    /// (Tf) transmission filter
    pub filter: Option<ColorValue>,
    /// (illum) illumination model
    pub illum: Option<u8>,
    /// (d/Tr) dissolve factor
    pub dissolve: Option<f32>,
    /// (d -halo) dissolve halo
    pub halo: bool,
    /// (Ns) specular exponent
    pub exponent: Option<f32>,
    /// (sharpness) reflection sharpness
    pub sharpness: Option<f32>,
    /// (Ni) optical density
    pub density: Option<f32>,

    /// (map_Ka) ambient texture
    pub ambient_map: Option<TextureMap>,
    /// (map_Kd) diffuse texture
    pub diffuse_map: Option<TextureMap>,
    /// (map_Ks) specular texture
    pub specular_map: Option<TextureMap>,
    /// (map_Ns) specular exponent texture
    pub exponent_map: Option<TextureMap>,
    /// (map_d) dissolve texture
    pub dissolve_map: Option<TextureMap>,
    /// (decal) decal texture
    pub decal_map: Option<TextureMap>,
    /// (disp) displacement texture
    pub disp_map: Option<TextureMap>,
    /// (bump/map_bump) bump texture
    pub bump_map: Option<TextureMap>,
    /// (map_aat) texture anti-aliasing
    pub anti_aliasing: bool,

    /// (refl) reflection map (type, map)
    pub reflection: Option<Refl>,

    /// (Pr) roughness
    pub roughness: Option<f32>,
    /// (Pm) metallic
    pub metallic: Option<f32>,
    /// (Ps) sheen
    pub sheen: Option<f32>,
    /// (Pc) clearcoat thickness
    pub cc_thickness: Option<f32>,
    /// (Pcr) clearcoat roughness
    pub cc_roughness: Option<f32>,
    /// (Ke) emissive
    pub emissive: Option<ColorValue>,
    /// (aniso) anisotropy
    pub anisotropy: Option<f32>,
    /// (anisor) anisotropy rotation
    pub anisotropy_rotation: Option<f32>,

    /// (map_Pr) roughness texture
    pub roughness_map: Option<TextureMap>,
    /// (map_Pm) metallic texture
    pub metallic_map: Option<TextureMap>,
    /// (map_Ps) sheen texture
    pub sheen_map: Option<TextureMap>,
    /// (map_Ke) emissive texture
    pub emissive_map: Option<TextureMap>,
    /// (norm) normal texture
    pub normal_map: Option<TextureMap>,
}

/// Color value
#[derive(Debug, Clone)]
pub enum ColorValue {
    /// RGB values
    RGB(f32, f32, f32),
    /// CIEXYZ values
    XYZ(f32, f32, f32),
    /// Spectral curve via .rfl file
    Spectral { file: Box<PathBuf>, factor: f32 },
}

impl ColorValue {
    fn rgb(v: (f32, f32, f32)) -> Self {
        Self::RGB(v.0, v.1, v.2)
    }

    fn xyz(v: (f32, f32, f32)) -> Self {
        Self::XYZ(v.0, v.1, v.2)
    }
}

/// Texture map
#[derive(Debug, Clone)]
pub struct TextureMap(Box<(PathBuf, Vec<MapOption>)>);

impl TextureMap {
    fn new(path: PathBuf, options: Vec<MapOption>) -> Self {
        Self(Box::new((path, options)))
    }

    /// Path to the texture file
    pub fn path(&self) -> &PathBuf {
        &self.0.0
    }

    /// Modifier options
    pub fn options(&self) -> &[MapOption] {
        &self.0.1
    }

    /// Takes the underlying data
    pub fn take(self) -> (PathBuf, Vec<MapOption>) {
        *self.0
    }
}

/// Texture map options
#[derive(Debug, Clone)]
pub enum MapOption {
    /// (blendu) horizontal blending
    BlendU(bool),
    /// (blendv) vertical blending
    BlendV(bool),
    /// (bm) bump multiplier
    BumpMultiplier(f32),
    /// (boost) mip-mapped clarity boost
    Boost(f32),
    /// (cc) color correction
    ColorCorrection(bool),
    /// (clamp) UV clamping
    Clamp(bool),
    /// (imfchan) channel to use
    Channel(Channel),
    /// (mm) base & gain values
    MM(f32, f32),
    /// (o) UV offset (u, v, w)
    Offset(f32, f32, f32),
    /// (s) UV scale (u, v, w)
    Scale(f32, f32, f32),
    /// (t) UV turbulance (u, v, w)
    Turbulence(f32, f32, f32),
    /// (texres) resolution
    Resolution(u16),
}

/// Texture map channel
#[derive(Debug, Clone, Copy)]
pub enum Channel {
    Red,
    Green,
    Blue,
    Matte,
    Luminance,
    ZDepth,
}

/// Reflection map
#[derive(Debug, Clone)]
pub enum Refl {
    Sphere(TextureMap),
    Cube(HashMap<String, TextureMap>),
}
