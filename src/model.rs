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
    pub players: HashMap<Id, Position>,
    #[diff = "clone"]
    pub fishes: Collection<Fish>,
}

impl Model {
    pub fn new() -> Self {
        let mut id_gen = IdGen::new();
        let fish_types: Vec<String> = serde_json::from_reader(
            std::fs::File::open(static_path().join("assets").join("fish").join("list.json"))
                .unwrap(),
        )
        .unwrap();
        Self {
            players: HashMap::new(),
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Position {
    pub pos: Vec2<f32>,
    pub vel: Vec2<f32>,
    pub rot: f32,
    pub w: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Message {
    Ping,
    Update(Position),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Event {
    Pong,
}

impl simple_net::Model for Model {
    type PlayerId = Id;
    type Message = Message;
    type Event = Event;
    const TICKS_PER_SECOND: f32 = 10.0;
    fn new_player(&mut self, events: &mut Vec<Self::Event>) -> Self::PlayerId {
        let id = self.id_gen.gen();
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
            Message::Update(pos) => {
                self.players.insert(*player_id, pos);
            }
        }
        vec![]
    }

    fn tick(&mut self, events: &mut Vec<Self::Event>) {
        let delta_time = 1.0 / Self::TICKS_PER_SECOND;
        self.update_fishes(delta_time);
    }
}
