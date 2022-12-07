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
    pub fn update_fishes(&mut self, delta_time: f32, events: &mut Vec<Event>) {
        let reeling_fishes: HashSet<Id> = self
            .players
            .iter()
            .flat_map(|player| {
                if let FishingState::Reeling { fish, .. } | FishingState::PreReeling { fish, .. } =
                    player.fishing_state
                {
                    Some(fish)
                } else {
                    None
                }
            })
            .collect();
        for fish in &mut self.fishes {
            if (fish.pos.pos - fish.target_pos).len() < 1.0 {
                const D: f32 = 10.0;
                fish.target_pos =
                    vec2(global_rng().gen_range(-D..D), global_rng().gen_range(-D..D));
                fish.scared = false;
            }

            // Scaring
            let run_away_distance = 5.0;
            for player in &self.players {
                if player.pos.vel.len() < 1.0 {
                    continue;
                }
                let scare_distance = 2.0;
                if (fish.pos.pos - player.pos.pos).len() < scare_distance {
                    fish.target_pos = player.pos.pos
                        + (fish.pos.pos - player.pos.pos).normalize_or_zero() * run_away_distance;
                    fish.scared = true;
                }
            }

            // Fishing attraction
            if !reeling_fishes.contains(&fish.id) && !fish.scared {
                for player in &mut self.players {
                    let bobber_attract_distance = 2.0;
                    if let FishingState::Waiting(bobber_pos) = player.fishing_state {
                        if (bobber_pos - fish.pos.pos).len() < bobber_attract_distance {
                            fish.target_pos = fish.pos.pos;
                            fish.pos.rot = (bobber_pos - fish.pos.pos).arg();
                            if global_rng().gen_bool(0.005) {
                                fish.pos.pos = bobber_pos;
                                fish.target_pos = bobber_pos
                                    + vec2(run_away_distance, 0.0)
                                        .rotate(global_rng().gen_range(0.0..2.0 * f32::PI));
                                fish.scared = true;
                                player.fishing_state = FishingState::PreReeling {
                                    fish: fish.id,
                                    bobber_pos,
                                };
                                events.push(Event::Reel {
                                    player: player.id,
                                    fish: fish.id,
                                });
                            }
                        }
                    }
                }
            }
            let target_pos = if reeling_fishes.contains(&fish.id) {
                fish.pos.pos
            } else {
                fish.target_pos
            };
            update_movement(
                &mut fish.pos,
                target_pos,
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
    pub fn draw_fishes(&self, framebuffer: &mut ugli::Framebuffer) {
        let model = self.model.get();
        for fish in &model.fishes {
            let Some(pos) = self.interpolated.get(&fish.id) else { continue };
            let pos = pos.get();
            self.draw_fish(framebuffer, fish, &pos);
        }
    }
    pub fn draw_fish(&self, framebuffer: &mut ugli::Framebuffer, fish: &Fish, pos: &Position) {
        let texture = &self.assets.fishes[fish.index];
        let matrix = Mat4::translate(
            // {
            //     let mut pos = pos.pos;
            //     for player in &self.model.get().players {
            //         if let FishingState::Reeling {
            //             fish: fish_id,
            //             bobber_pos,
            //         } = player.fishing_state
            //         {
            //             if fish_id == fish.id {
            //                 pos = bobber_pos;
            //             }
            //         }
            //     }
            //     pos
            // }
            pos.pos.extend(-1.0),
        ) * Mat4::rotate_z(pos.rot)
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
