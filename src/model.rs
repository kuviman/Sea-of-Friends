use super::*;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Id(u64);

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct IdGen {
    next_id: u64,
}

impl IdGen {
    pub fn new() -> Self {
        Self { next_id: 0 }
    }
    pub fn gen(&mut self) -> Id {
        let id = Id(self.next_id);
        self.next_id += 1;
        id
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Diff)]
pub struct Model {
    #[diff = "clone"]
    pub id_gen: IdGen,
    #[diff = "clone"]
    pub players: Collection<Player>,
    #[diff = "clone"]
    pub fishes: Collection<Fish>,
}

static mut MAP: Option<image::RgbaImage> = None;

impl Model {
    pub fn get_map_color(pos: Vec2<f32>) -> image::Rgba<u8> {
        let map = unsafe {
            MAP.get_or_insert_with(|| {
                image::open(static_path().join("assets").join("map.png"))
                    .unwrap()
                    .into_rgba8()
            })
        };
        const SIZE: f32 = 100.0;
        let pos = pos.map(|x| {
            ((((x + SIZE) / (2.0 * SIZE)) * map.width() as f32) as u32).clamp(0, map.width() - 1)
        });
        *map.get_pixel(pos.x, pos.y)
    }
    pub fn new() -> Self {
        let mut id_gen = IdGen::new();
        let fish_types: Vec<String> = serde_json::from_reader(
            std::fs::File::open(static_path().join("assets").join("fish").join("list.json"))
                .unwrap(),
        )
        .unwrap();
        Self {
            players: Collection::new(),
            fishes: {
                let mut fishes = Collection::new();
                for _ in 0..100 {
                    const D: f32 = 10.0;
                    fishes.insert(Fish::new(
                        id_gen.gen(),
                        global_rng().gen_range(0..fish_types.len()),
                        vec2(global_rng().gen_range(-D..D), global_rng().gen_range(-D..D)),
                    ))
                }
                fishes
            },
            id_gen,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Message {
    Ping,
    Update(Player),
    Catch(Id),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Event {
    Pong,
    Reel { player: Id, fish: Id },
}

impl simple_net::Model for Model {
    type PlayerId = Id;
    type Message = Message;
    type Event = Event;
    const TICKS_PER_SECOND: f32 = 10.0;
    fn new_player(&mut self, events: &mut Vec<Self::Event>) -> Self::PlayerId {
        let id = self.id_gen.gen();
        self.players.insert(Player::new(id, Vec2::ZERO));
        id
    }

    fn drop_player(&mut self, events: &mut Vec<Self::Event>, player_id: &Self::PlayerId) {
        self.players.remove(player_id);
    }

    fn handle_message(
        &mut self,
        events: &mut Vec<Self::Event>,
        player_id: &Self::PlayerId,
        message: Self::Message,
    ) -> Vec<Event> {
        match message {
            Message::Ping => return vec![Event::Pong],
            Message::Update(data) => {
                if data.id == *player_id {
                    *self.players.get_mut(player_id).unwrap() = data;
                }
            }
            Message::Catch(id) => {
                self.fishes.remove(&id);
            }
        }
        vec![]
    }

    fn tick(&mut self, events: &mut Vec<Self::Event>) {
        let delta_time = 1.0 / Self::TICKS_PER_SECOND;
        self.update_fishes(delta_time, events);
    }
}
