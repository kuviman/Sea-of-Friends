use super::*;

#[derive(geng::Assets)]
pub struct Shaders {
    pub font: ugli::Program,
    pub water: ugli::Program,
    pub land: ugli::Program,
    pub land2: ugli::Program,
    pub fish: ugli::Program,
    pub obj: ugli::Program,
    pub edge: ugli::Program,
    pub wave: ugli::Program,
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
    pub name: String,
    pub cost: u32,
    pub scale: f32,
    pub shops: Vec<Vec2<f32>>,
}

#[derive(geng::Assets)]
pub struct ShopAssets {
    pub fish: ugli::Texture,
    // pub temp: ugli::Texture,
    // pub rowboat: ugli::Texture,
    pub itsboats: ugli::Texture,
    pub air_shop: ugli::Texture,
    pub big_boat_shop: ugli::Texture,
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
    #[asset(postprocess = "make_looped")]
    pub boat_moving: geng::Sound,
    #[asset(range = "1..=5", path = "casting*.wav")]
    pub casting: Vec<geng::Sound>,
    pub ding: geng::Sound,
    pub drop_fish_land: geng::Sound,
    pub drop_fish_water: geng::Sound,
    pub enter_boat: geng::Sound,
    pub exit_boat: geng::Sound,
    pub sell_fish: geng::Sound,
    pub show_fish: geng::Sound,
    #[asset(range = "1..=5", path = "splash*.wav")]
    pub splash: Vec<geng::Sound>,
    pub stop_fishing: geng::Sound,
    pub upgrade_boat: geng::Sound,
    #[asset(range = "1..=2", path = "whip*.wav")]
    pub whip: Vec<geng::Sound>,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub enum SoundType {
    Casting,
    Ding,
    DropFishLand,
    DropFishWater,
    EnterBoat,
    ExitBoat,
    SellFish,
    ShowFish,
    Splash,
    StopFishing,
    UpgradeBoat,
    Whip,
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
    #[asset(load_with = "load_environment(&geng, &base_path.join(\"environment\"))")]
    pub environment: EnvironmentAssets,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum FishBehavior {
    Orbit,
    Chaos,
    Idle,
    Space,
    Land,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SpawnCircle {
    pub center: Vec2<f32>,
    pub radius: f32,
    pub inner_radius: Option<f32>,
    pub behavior: FishBehavior,
    pub reversed: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FishConfig {
    pub name: String,
    pub cost: u32,
    pub spawn_circle: SpawnCircle,
    pub count: u32,
    pub size: f32,
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

pub struct EnvironmentAssets {
    pub land: Vec<ugli::Texture>,
    pub shallow: Vec<ugli::Texture>,
}

fn load_environment(geng: &Geng, path: &std::path::Path) -> geng::AssetFuture<EnvironmentAssets> {
    let geng = geng.clone();
    let path = path.to_owned();
    async move {
        let json = <String as geng::LoadAsset>::load(&geng, &path.join("list.json")).await?;
        let list: HashMap<String, Vec<String>> = serde_json::from_str(&json)?;
        async fn load_textures(
            geng: &Geng,
            path: &std::path::Path,
            names: &[String],
        ) -> Vec<ugli::Texture> {
            future::join_all(
                names
                    .iter()
                    .map(|name| geng::LoadAsset::load(geng, &path.join(format!("{}.png", name)))),
            )
            .await
            .into_iter()
            .collect::<anyhow::Result<_>>()
            .unwrap()
        }
        Ok(EnvironmentAssets {
            land: load_textures(&geng, &path, &list["land"]).await,
            shallow: load_textures(&geng, &path, &list["shallow"]).await,
        })
    }
    .boxed_local()
}
