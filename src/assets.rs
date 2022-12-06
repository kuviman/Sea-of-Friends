use super::*;

#[derive(geng::Assets)]
pub struct Shaders {
    pub water: ugli::Program,
    pub obj: ugli::Program,
    pub obj2: ugli::Program,
}

#[derive(geng::Assets, Serialize, Deserialize)]
#[asset(json)]
pub struct Config {
    pub sea_color: Rgba<f32>,
}

#[derive(geng::Assets)]
pub struct Assets {
    pub shaders: Shaders,
    #[asset(path = "1391 Rowboat.obj")]
    pub boat: Obj,
    pub bobber: ugli::Texture,
    pub player: ugli::Texture,
    pub config: Config,
    #[asset(path = "PerlinNoise.png", postprocess = "make_repeated")]
    pub surface_noise: ugli::Texture,
    #[asset(path = "WaterDistortion.png", postprocess = "make_repeated")]
    pub distort_noise: ugli::Texture,
}
