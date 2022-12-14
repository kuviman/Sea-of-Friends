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
    pub time: f32,
}

impl Model {
    pub fn init() -> Self {
        let id_gen = IdGen::new();
        let mut result = Self {
            players: Collection::new(),
            fishes: Collection::new(),
            id_gen,
            time: 0.0,
        };
        for i in 0..FishConfigs::get().configs.len() {
            result.spawn_fish_group(i);
        }
        result
    }

    pub fn spawn_fish(&mut self, i: usize) {
        let mut inner_radius: f32 = 0.0;
        let fish_config = &FishConfigs::get().configs[i];
        let radius = fish_config.spawn_circle.radius;
        let center = fish_config.spawn_circle.center;
        if let Some(r) = fish_config.spawn_circle.inner_radius {
            inner_radius = r;
        }
        // polar coordinates because we're fancy
        let r = global_rng().gen_range(inner_radius..radius);
        let angle = global_rng().gen_range(0.0..(f32::PI * 2.0));
        self.fishes.insert(Fish::new(
            self.id_gen.gen(),
            i,
            vec2(center.x + r * angle.cos(), center.y + r * angle.sin()),
        ))
    }

    pub fn spawn_fish_group(&mut self, i: usize) {
        let fish_config = &FishConfigs::get().configs[i];
        for j in 0..fish_config.count {
            self.spawn_fish(i);
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
    RespawnFish { index: usize },
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
        if let Some(p) = self.players.get(player_id) {
            for fish in p.inventory.clone() {
                self.spawn_fish(fish);
            }
        }
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
            Message::RespawnFish { index } => {
                self.spawn_fish(index);
            }
        }
        vec![]
    }

    fn tick(&mut self, events: &mut Vec<Self::Event>) {
        let delta_time = 1.0 / Self::TICKS_PER_SECOND;
        self.time += delta_time;
        self.update_fishes(delta_time, events);
    }
}
