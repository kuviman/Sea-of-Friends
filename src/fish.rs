use super::*;

#[derive(HasId, PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct Fish {
    pub id: Id,
    pub index: usize,
    pub pos: Position,
    pub target_pos: Vec2<f32>,
    pub scared: bool,
}

impl Fish {
    pub fn new(id: Id, index: usize, pos: Vec2<f32>) -> Self {
        Self {
            id,
            index,
            pos: Position {
                pos,
                vel: Vec2::ZERO,
                rot: 0.0,
                w: 0.0,
            },
            target_pos: pos,
            scared: false,
        }
    }
}

impl Model {
    pub fn update_fishes(&mut self, delta_time: f32) {
        for fish in &mut self.fishes {
            if (fish.pos.pos - fish.target_pos).len() < 1.0 {
                const D: f32 = 10.0;
                fish.target_pos =
                    vec2(global_rng().gen_range(-D..D), global_rng().gen_range(-D..D));
                fish.scared = false;
            }
            for player in self.players.values() {
                let scare_distance = 2.0;
                let run_away_distance = 5.0;
                if (fish.pos.pos - player.pos).len() < scare_distance {
                    fish.target_pos = player.pos
                        + (fish.pos.pos - player.pos).normalize_or_zero() * run_away_distance;
                    fish.scared = true;
                }
            }
            update_movement(
                &mut fish.pos,
                fish.target_pos,
                if fish.scared {
                    MovementProps {
                        max_speed: 3.0,
                        max_rotation_speed: 5.0,
                        angular_acceleration: 10.0,
                        acceleration: 3.0,
                    }
                } else {
                    MovementProps {
                        max_speed: 0.5,
                        max_rotation_speed: 2.0,
                        angular_acceleration: 1.0,
                        acceleration: 0.5,
                    }
                },
                delta_time,
            );
        }
    }
}

impl Game {
    pub fn draw_fish(&self, framebuffer: &mut ugli::Framebuffer, fish: &Fish, pos: &Position) {
        let texture = &self.assets.fishes[fish.index];
        let matrix = Mat4::translate(pos.pos.extend(-1.0))
            * Mat4::rotate_z(pos.rot)
            * Mat4::scale(texture.size().map(|x| x as f32 / 500.0).extend(1.0))
            * Mat4::rotate_x(f32::PI / 2.0);
        ugli::draw(
            framebuffer,
            &self.assets.shaders.obj,
            ugli::DrawMode::TriangleFan,
            &self.quad,
            (
                ugli::uniforms! {
                    u_model_matrix: matrix,
                    u_texture: texture,
                },
                geng::camera3d_uniforms(&self.camera, self.framebuffer_size),
            ),
            ugli::DrawParameters {
                blend_mode: Some(ugli::BlendMode::default()),
                depth_func: Some(ugli::DepthFunc::Less),
                ..default()
            },
        );
    }
}
