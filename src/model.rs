use super::*;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Id(pub u64);

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct IdGen {
    next_id: u64,
}

pub type FishType = usize;

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

impl Default for IdGen {
    fn default() -> Self {
        Self::new()
    }
}

pub struct FishConfigs {
    pub configs: Vec<FishConfig>,
}
static mut FISH_CONFIG: Option<FishConfigs> = None;

impl FishConfigs {
    pub fn get() -> &'static FishConfigs {
        unsafe { FISH_CONFIG.get_or_insert_with(FishConfigs::load) }
    }
    pub fn load() -> Self {
        Self {
            configs: serde_json::from_reader(
                std::fs::File::open(static_path().join("assets").join("fish").join("list.json"))
                    .unwrap(),
            )
            .unwrap(),
        }
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

impl Model {
    pub fn init() -> Self {
        let mut id_gen = IdGen::new();
        Self {
            players: Collection::new(),
            fishes: {
                let mut fishes = Collection::new();
                for i in 0..FishConfigs::get().configs.len() {
                    let fish_config = &FishConfigs::get().configs[i];
                    for j in 0..fish_config.count {
                        let mut D: f32 = 10.0;
                        let mut center = Vec2::ZERO;
                        if let Some(spawn_circle) = &fish_config.spawn_circle {
                            D = spawn_circle.radius;
                            center = spawn_circle.center;
                        }
                        fishes.insert(Fish::new(
                            id_gen.gen(),
                            i,
                            vec2(
                                center.x + global_rng().gen_range(-D..D),
                                center.y + global_rng().gen_range(-D..D),
                            ),
                        ))
                    }
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
    SpawnFish { index: usize, pos: Vec2<f32> },
    Broadcast(Event),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Event {
    Pong,
    Reel {
        player: Id,
        fish: Id,
    },
    CaughtFish {
        player: Id,
        fish: Id,
        fish_type: FishType,
        position: Vec2<f32>,
    },
    Sound {
        player: Id,
        sound_type: SoundType,
        pos: Vec2<f32>,
    },
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
                if let Some(fish) = self.fishes.remove(&id) {
                    events.push(Event::CaughtFish {
                        fish: id,
                        fish_type: fish.index,
                        player: *player_id,
                        position: fish.pos.pos,
                    });
                }
            }
            Message::SpawnFish { index, pos } => {
                self.fishes.insert(Fish::new(self.id_gen.gen(), index, pos));
            }
            Message::Broadcast(event) => {
                events.push(event);
            }
        }
        vec![]
    }

    fn tick(&mut self, events: &mut Vec<Self::Event>) {
        let delta_time = 1.0 / Self::TICKS_PER_SECOND;
        self.update_fishes(delta_time, events);
    }
}
