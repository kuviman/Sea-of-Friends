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
    pub space_color: Rgba<f32>,
    pub fish_shops: Vec<Vec2<f32>>,
    pub boat_types: Vec<BoatConfig>,
}

#[derive(Serialize, Deserialize)]
pub struct BoatConfig {
    pub cost: u32,
    pub scale: f32,
    pub shops: Vec<Vec2<f32>>,
}

#[derive(geng::Assets)]
pub struct ShopAssets {
    pub fish: ugli::Texture,
    pub temp: ugli::Texture,
    pub rowboat: ugli::Texture,
}

#[derive(geng::Assets)]
pub struct ShipAssets {
    #[asset(path = "model.obj")]
    pub obj: Obj,
    #[asset(load_with = "future::ready(Ok::<_, anyhow::Error>(Vec::new()))")]
    pub seats: Vec<Vec3<f32>>,
}

impl ShipAssets {
    fn postprocess(list: &mut [Self]) {
        for ship in list {
            let mut seats = std::collections::BTreeMap::<usize, Vec3<f32>>::new();
            for mesh in &ship.obj.meshes {
                if let Some(index) = mesh.name.strip_prefix("Seat.") {
                    let index = index.parse().unwrap();
                    let mut p = Vec3::ZERO;
                    for v in mesh.geometry.iter() {
                        p += v.a_v;
                    }
                    seats.insert(index, p / mesh.geometry.len() as f32);
                }
            }
            ship.obj
                .meshes
                .retain(|mesh| !mesh.name.starts_with("Seat."));
            ship.seats = seats.values().copied().collect();
        }
    }
}
#[derive(geng::Assets)]
pub struct PlayerAssets {
    pub eyes: ugli::Texture,
    pub hat: ugli::Texture,
    pub pants: ugli::Texture,
    pub shirt_holding: ugli::Texture,
    pub shirt: ugli::Texture,
    pub skin_fishing: ugli::Texture,
    pub skin: ugli::Texture,
    pub skin_holding: ugli::Texture,
}

#[derive(geng::Assets)]
pub struct Sounds {
    #[asset(range = "1..=5", path = "bobber_hit*.wav")]
    pub bobber_hit: Vec<geng::Sound>,
    // #[asset(range = "1..=5", path = "casting*.wav")]
    #[asset(range = "1..=1", path = "casting1*.wav")]
    pub casting: Vec<geng::Sound>,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub enum SoundType {
    BobberHit,
    Casting,
}

#[derive(geng::Assets)]
pub struct Assets {
    pub shaders: Shaders,
    #[asset(
        range = "1..=3",
        path = "ships/*",
        postprocess = "ShipAssets::postprocess"
    )]
    pub ships: Vec<ShipAssets>,
    pub bobber: ugli::Texture,
    pub player: PlayerAssets,
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
    pub sounds: Sounds,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum FishBehavior {
    Orbit,
    Chaos,
    Idle,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SpawnCircle {
    pub center: Vec2<f32>,
    pub radius: f32,
    pub behavior: FishBehavior,
    pub reversed: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FishConfig {
    pub name: String,
    pub cost: u32,
    pub spawn_circle: Option<SpawnCircle>,
    pub count: u32,
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
