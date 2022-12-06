use super::*;

#[derive(HasId, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Player {
    pub id: Id,
    pub pos: Position,
}

impl Player {
    pub fn new(id: Id, pos: Vec2<f32>) -> Self {
        Self {
            id,
            pos: Position {
                pos,
                vel: Vec2::ZERO,
                rot: global_rng().gen_range(0.0..2.0 * f32::PI),
                w: 0.0,
            },
        }
    }
}
