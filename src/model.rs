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
    pub positions: HashMap<Id, Position>,
}

impl Model {
    pub fn new() -> Self {
        Self {
            id_gen: IdGen::new(),
            positions: HashMap::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Position {
    pub pos: Vec2<f32>,
    pub vel: Vec2<f32>,
    pub rot: f32,
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
        self.positions.remove(player_id);
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
                self.positions.insert(*player_id, pos);
            }
        }
        vec![]
    }

    fn tick(&mut self, events: &mut Vec<Self::Event>) {}
}
