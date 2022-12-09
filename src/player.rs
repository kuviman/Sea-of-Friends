use super::*;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FishingState {
    Idle,
    Spinning,
    Casting(Vec2<f32>),
    Waiting(Vec2<f32>),
    PreReeling { fish: Id, bobber_pos: Vec2<f32> },
    Reeling { fish: Id, bobber_pos: Vec2<f32> },
    Attached(Id),
}

#[derive(HasId, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Player {
    pub id: Id,
    pub pos: Position,
    pub fishing_state: FishingState,
    pub fish_in_hands: Option<FishType>,
    pub boat_level: u8,
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
            fishing_state: FishingState::Idle,
            fish_in_hands: None,
            boat_level: 0,
        }
    }
}

impl Game {
    pub fn update_local_player_data(&mut self, delta_time: f32) {
        let model = self.model.get();
        for player in itertools::chain![
            model
                .players
                .iter()
                .filter(|player| player.id != self.player_id),
            std::iter::once(&self.player),
        ] {
            let time = match player.fishing_state {
                FishingState::Casting(_) => Some(1.0),
                FishingState::PreReeling { .. } => Some(1.0),
                FishingState::Reeling { .. } => Some(1.0),
                _ => None,
            };
            if let Some(time) = time {
                *self.player_timings.entry(player.id).or_default() += delta_time / time;
            } else {
                self.player_timings.remove(&player.id);
            }
        }
    }
    pub fn draw_players(&self, framebuffer: &mut ugli::Framebuffer) {
        let model = self.model.get();
        self.draw_player(framebuffer, &self.player, &self.player.pos);
        for player in &model.players {
            if player.id == self.player_id {
                continue;
            }
            let Some(pos) = self.interpolated.get(&player.id) else { continue };
            let pos = pos.get();
            self.draw_player(framebuffer, player, &pos);
        }
    }
    fn draw_player(&self, framebuffer: &mut ugli::Framebuffer, player: &Player, pos: &Position) {
        let model_matrix = Mat4::translate(pos.pos.extend(0.0)) * Mat4::rotate_z(pos.rot);
        let height = Map::get().get_height(pos.pos);
        if height < 0.0 {
            for mesh in &self.assets.boat.meshes {
                ugli::draw(
                    framebuffer,
                    &self.assets.shaders.obj,
                    ugli::DrawMode::Triangles,
                    &mesh.geometry,
                    (
                        ugli::uniforms! {
                            u_model_matrix: model_matrix,
                            u_texture: mesh.material.texture.as_deref().unwrap_or(&self.white_texture),
                        },
                        geng::camera3d_uniforms(&self.camera, self.framebuffer_size),
                    ),
                    ugli::DrawParameters {
                        depth_func: Some(ugli::DepthFunc::Less),
                        ..default()
                    },
                );
            }
        }
        self.draw_quad(
            framebuffer,
            Mat4::translate(pos.pos.extend(height.max(0.0)))
                * Mat4::rotate_x(-self.camera.rot_v)
                * Mat4::scale(vec3(1.0, 0.0, 2.0) * 0.25)
                * Mat4::translate(vec3(0.0, 0.0, 1.0)),
            &self.assets.player,
        );
        let mut fishing_rod_rot = None;
        let mut bobber = None;
        match &player.fishing_state {
            FishingState::Idle => {}
            FishingState::Spinning => {
                fishing_rod_rot = Some(self.time * 5.0);
            }
            FishingState::Casting(target_pos) => {
                let t = self
                    .player_timings
                    .get(&player.id)
                    .copied()
                    .unwrap_or(0.0)
                    .min(1.0);
                fishing_rod_rot = Some(t);
                let start_pos = pos.pos.extend(1.0);
                bobber = Some(start_pos * (1.0 - t) + target_pos.extend(0.0) * t);
            }
            FishingState::Waiting(bobber_pos) | FishingState::PreReeling { bobber_pos, .. } => {
                fishing_rod_rot = Some(0.5);
                bobber = Some(bobber_pos.extend(0.0));
            }
            FishingState::Reeling { fish, bobber_pos } => {
                fishing_rod_rot = Some(1.0);
                bobber = Some(bobber_pos.extend(-1.0));
            }
            FishingState::Attached(id) => {
                if let Some(player) = self.model.get().players.get(&id) {
                    fishing_rod_rot = Some(1.0);
                    if let Some(p) = self.interpolated.get(&id) {
                        bobber = Some(p.get().pos.extend(0.5));
                    }
                }
            }
        }
        // Draw fishing rod
        if let Some(rot) = fishing_rod_rot {
            let texture = &self.assets.fishing_rod;
            let fishing_rod_matrix = Mat4::translate(pos.pos.extend(0.5))
                * Mat4::rotate_x(-self.camera.rot_v)
                * Mat4::rotate_y(rot)
                * Mat4::translate(vec3(0.0, 0.0, 0.5))
                * Mat4::scale(vec3(
                    texture.size().x as f32 / texture.size().y as f32,
                    1.0,
                    1.0,
                ));
            self.draw_quad(framebuffer, fishing_rod_matrix, texture);

            // Bobber
            if let Some(bobber_pos) = bobber {
                let fishing_rod_pos = (fishing_rod_matrix * vec4(0.0, 0.0, 1.0, 1.0)).xyz();
                ugli::draw(
                    framebuffer,
                    &self.assets.shaders.obj,
                    ugli::DrawMode::LineStrip { line_width: 1.0 },
                    &ugli::VertexBuffer::new_dynamic(self.geng.ugli(), {
                        const N: i32 = 10;
                        (0..=N)
                            .map(|i| {
                                let t = i as f32 / N as f32;
                                ObjVertex {
                                    a_v: fishing_rod_pos * (1.0 - t)
                                        + bobber_pos * t
                                        + vec3(0.0, 0.0, (t * 2.0 - 1.0).sqr() - 1.0),
                                    a_uv: Vec2::ZERO,
                                    a_vn: Vec3::ZERO,
                                }
                            })
                            .collect()
                    }),
                    (
                        ugli::uniforms! {
                            u_model_matrix: Mat4::identity(),
                            u_texture: &self.white_texture,
                        },
                        geng::camera3d_uniforms(&self.camera, self.framebuffer_size),
                    ),
                    ugli::DrawParameters {
                        depth_func: Some(ugli::DepthFunc::Less),
                        ..default()
                    },
                );
                self.draw_quad(
                    framebuffer,
                    Mat4::translate(bobber_pos)
                        * Mat4::scale_uniform(0.1)
                        * Mat4::rotate_x(-self.camera.rot_v),
                    &self.assets.bobber,
                );
            }
        }

        if let Some(fish) = player.fish_in_hands {
            self.draw_texture(
                framebuffer,
                pos.pos.extend(height.max(0.0) + 1.0),
                0.25,
                &self.assets.fishes[fish].texture,
                vec2(0.0, -1.0),
            )
        }
    }
}
