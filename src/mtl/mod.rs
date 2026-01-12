mod parser;

use std::collections::HashMap;
use std::path::PathBuf;

use winnow::{BStr, Parser};

use crate::WobjError;

pub fn parse_mtl(bytes: &[u8]) -> Result<HashMap<String, Material>, WobjError> {
    parser::parse_mtl
        .parse(BStr::new(bytes))
        .map_err(WobjError::from)
}

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
    /// (d) dissolve (factor, halo)
    pub dissolve: Option<(f32, bool)>,
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
    /// (bump) bump texture
    pub bump_map: Option<TextureMap>,
    /// (map_aat) texture anti-aliasing
    pub aa_map: Option<bool>,

    /// (refl) reflection map (type, map)
    pub relf: Vec<(String, TextureMap)>,
}

#[derive(Debug, Clone)]
pub enum ColorValue {
    RGB(f32, f32, f32),
    XYZ(f32, f32, f32),
    Spectral { file: PathBuf, factor: f32 },
}

#[derive(Debug, Clone)]
pub struct TextureMap {
    pub path: PathBuf,
    pub options: Vec<MapOption>,
}

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

#[derive(Debug, Clone, Copy)]
pub enum Channel {
    Red,
    Green,
    Blue,
    Matte,
    Luminance,
    ZDepth,
}
