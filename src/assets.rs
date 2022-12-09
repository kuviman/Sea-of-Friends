use super::*;

#[derive(geng::Assets)]
pub struct Shaders {
    pub water: ugli::Program,
    pub land: ugli::Program,
    pub land2: ugli::Program,
    pub obj: ugli::Program,
    pub obj2: ugli::Program,
    pub edge: ugli::Program,
}

#[derive(geng::Assets, Serialize, Deserialize)]
#[asset(json)]
pub struct Config {
    pub inventory_size: usize,
    pub sea_color: Rgba<f32>,
    pub fish_shops: Vec<Vec2<f32>>,
    pub small_boat_shops: Vec<Vec2<f32>>,
    pub small_boat_cost: u32,
}

#[derive(geng::Assets)]
pub struct ShopAssets {
    pub fish: ugli::Texture,
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
    #[asset(load_with = "load_fishes(&geng, &base_path.join(\"fish\"))")]
    pub fishes: Vec<FishAssets>,
    pub fishing_rod: ugli::Texture,
    pub map: ugli::Texture,
    pub map_color: ugli::Texture,
    #[asset(path = "music.mp3", postprocess = "make_looped")]
    pub music: geng::Sound,
    pub shops: ShopAssets,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FishConfig {
    pub name: String,
    pub cost: u32,
}

pub struct FishAssets {
    pub texture: ugli::Texture,
    pub config: FishConfig,
}

fn load_fishes(geng: &Geng, path: &std::path::Path) -> geng::AssetFuture<Vec<FishAssets>> {
    let geng = geng.clone();
    let path = path.to_owned();
    async move {
        let json = <String as geng::LoadAsset>::load(&geng, &path.join("list.json")).await?;
        let list: Vec<FishConfig> = serde_json::from_str(&json)?;
        let textures: Vec<ugli::Texture> = future::join_all(list.iter().map(|config| {
            <ugli::Texture as geng::LoadAsset>::load(
                &geng,
                &path.join(format!("{}.png", config.name)),
            )
        }))
        .await
        .into_iter()
        .collect::<Result<_, _>>()?;
        Ok(textures
            .into_iter()
            .zip(list)
            .map(|(texture, config)| FishAssets { texture, config })
            .collect())
    }
    .boxed_local()
}
