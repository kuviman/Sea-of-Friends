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

pub struct FishMovementUpdate {
    vel: Vec2<f32>,
}

impl Model {
    pub fn flock(fish: &Fish, delta_time: f32, nearby_fish: &Vec<&Fish>) -> Vec2<f32> {
        let mut result: Vec2<f32> = Vec2::ZERO;
        for fish in nearby_fish {
            result += fish.pos.pos;
        }
        if !nearby_fish.is_empty() {
            result /= nearby_fish.len() as f32;
        }
        result.sub(fish.pos.pos) / 20.0 * delta_time
    }
    pub fn avoid(fish: &Fish, delta_time: f32, nearby_fish: &Vec<&Fish>) -> Vec2<f32> {
        let mut result: Vec2<f32> = Vec2::ZERO;
        for fish2 in nearby_fish {
            if fish2.pos.pos.sub(fish.pos.pos).len() < 1.0 {
                result -= fish2.pos.pos - fish.pos.pos;
            }
        }
        result * delta_time
    }
    pub fn match_velocity(fish: &Fish, delta_time: f32, nearby_fish: &Vec<&Fish>) -> Vec2<f32> {
        let mut result: Vec2<f32> = Vec2::ZERO;
        for fish in nearby_fish {
            result += fish.pos.vel;
        }
        if !nearby_fish.is_empty() {
            result /= nearby_fish.len() as f32;
        }
        result.sub(fish.pos.vel) / 8.0 * delta_time
    }
    pub fn currents(fish: &Fish, delta_time: f32) -> Vec2<f32> {
        let height = Map::get_height(Map::get(), fish.pos.pos);
        if height > -0.3 {
            // TODO: use normals here
            return Vec2 {
                x: (height + 0.301) * 10.0,
                y: 0.0,
            };
        }
        let fish_config = &FishConfigs::get().configs[fish.index];
        if let Some(spawn_circle) = &fish_config.spawn_circle {
            let dist = spawn_circle.center.sub(fish.pos.pos);
            // We're inside our spawn circle - follow behavior rules
            if dist.len() < spawn_circle.radius {
                match &spawn_circle.behavior {
                    FishBehavior::Idle => {
                        return Vec2::ZERO;
                    }
                    FishBehavior::Orbit => {
                        if let Some(rev) = &spawn_circle.reversed {
                            if *rev {
                                return Vec2 {
                                    x: -dist.y,
                                    y: dist.x,
                                } / spawn_circle.radius
                                    / 2.0;
                            }
                        }
                        return Vec2 {
                            x: dist.y,
                            y: -dist.x,
                        } / spawn_circle.radius
                            / 2.0;
                    }
                    FishBehavior::Chaos => {
                        let scaled_pos = fish.pos.pos / 5.0
                            + Vec2 {
                                x: fish.index as f32,
                                y: (fish.index % 2) as f32,
                            };
                        return Vec2 {
                            x: scaled_pos.x.cos() + scaled_pos.y.cos(),
                            y: scaled_pos.x.sin() + scaled_pos.y.sin(),
                        } * delta_time;
                    }
                }
            }
            // Outside our designated spawn area - head home
            return dist / dist.len() * (dist.len() - spawn_circle.radius) * delta_time;
        }
        // No spawn circle specified - fallback to chaos
        let scaled_pos = fish.pos.pos / 5.0
            + Vec2 {
                x: fish.index as f32,
                y: (fish.index % 2) as f32,
            };
        Vec2 {
            x: scaled_pos.x.cos() + scaled_pos.y.cos(),
            y: scaled_pos.x.sin() + scaled_pos.y.sin(),
        } * delta_time
    }
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

        let mut updates: HashMap<Id, FishMovementUpdate> = HashMap::new();
        for fish in &self.fishes {
            let nearby_fish: Vec<&Fish> = self
                .fishes
                .iter()
                .filter(|f| {
                    f.pos.pos.sub(fish.pos.pos).len() < 3.0
                        && f.id() != fish.id()
                        && f.index == fish.index
                })
                .collect();
            let v1 = Self::flock(fish, delta_time, &nearby_fish);
            let v2 = Self::avoid(fish, delta_time, &nearby_fish);
            let v3 = Self::match_velocity(fish, delta_time, &nearby_fish);
            let v4 = Self::currents(fish, delta_time);

            let v = v1 + v2 + v3 + v4;
            // let cur = Self::get_map_color(fish.pos.pos)[0];
            // if cur > 0 {
            //     if Self::get_map_color(
            //         fish.pos.pos + (fish.target_pos - fish.pos.pos).clamp_len(..=1.0),
            //     )[0] > cur
            //     {
            //         const D: f32 = 1.0;
            //         fish.target_pos = fish.pos.pos + fish.pos.pos.sub(fish.target_pos).normalize() * 5.0
            //     }
            // }

            updates.insert(fish.id, FishMovementUpdate { vel: v });
        }
        for fish in &mut self.fishes {
            // // Scaring
            let run_away_distance = 5.0;
            for player in &self.players {
                if player.pos.vel.len() < 1.0 {
                    continue;
                }
                let scare_distance = 4.0;
                if (fish.pos.pos - player.pos.pos).len() < scare_distance {
                    fish.target_pos = player.pos.pos
                        + (fish.pos.pos - player.pos.pos).normalize_or_zero() * run_away_distance;
                    fish.scared = true;
                }
            }
            if reeling_fishes.contains(&fish.id) {
                fish.pos.vel = Vec2::ZERO;
                fish.pos.w = 0.0;
                continue;
            }
            if fish.scared {
                update_movement(
                    &mut fish.pos,
                    fish.target_pos,
                    MovementProps {
                        max_speed: 3.0,
                        max_rotation_speed: 5.0,
                        angular_acceleration: 10.0,
                        acceleration: 3.0,
                        water: true,
                    },
                    delta_time,
                );
                if (fish.pos.pos - fish.target_pos).len() < 1.0 {
                    fish.scared = false;
                }
                continue;
            }
            // Attraction
            let mut attracted = false;
            for player in &mut self.players {
                let bobber_attract_distance = 2.2;
                if let FishingState::Waiting(bobber_pos) = player.fishing_state {
                    if (bobber_pos - fish.pos.pos).len() < bobber_attract_distance {
                        attracted = true;
                        fish.target_pos = fish.pos.pos;
                        fish.pos.rot = normalize_angle((bobber_pos - fish.pos.pos).arg());
                        fish.pos.w = 0.0;
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
                            events.push(Event::Sound {
                                player: Id(u64::MAX),
                                sound_type: SoundType::Splash,
                                pos: fish.pos.pos,
                            });
                        }
                    }
                }
            }
            if attracted {
                let target_pos = fish.pos.pos;
                update_movement(
                    &mut fish.pos,
                    target_pos,
                    MovementProps {
                        max_speed: 2.0,
                        max_rotation_speed: 2.0,
                        angular_acceleration: 1.0,
                        acceleration: 1.0,
                        water: true,
                    },
                    delta_time,
                );
                continue;
            }
            if let Some(update) = updates.get(&fish.id) {
                fish.pos.vel += update.vel;
            }
            fish.pos.vel = fish.pos.vel.clamp_len(..=1.7);
            let new_rot = fish.pos.vel.normalize().arg();
            fish.pos.w = normalize_angle(new_rot - fish.pos.rot).clamp_abs(fish.pos.vel.len());
            if fish.pos.vel.len() < 0.2 {
                fish.pos.w = 0.0;
            }
            fish.pos.rot = normalize_angle(fish.pos.rot + fish.pos.w);
            fish.pos.pos += fish.pos.vel * delta_time;
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
        let texture = &self.assets.fishes[fish.index].texture;
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
            pos.pos.extend(-0.1),
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
                    u_color: Rgba::WHITE,
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
