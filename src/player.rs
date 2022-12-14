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

impl FishingState {
    pub fn bobber_pos(&self) -> Option<Vec2<f32>> {
        match self {
            Self::Waiting(bobber_pos)
            | Self::PreReeling { bobber_pos, .. }
            | Self::Reeling { bobber_pos, .. } => Some(*bobber_pos),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlayerColors {
    pub hat: Rgba<f32>,
    pub pants: Rgba<f32>,
    pub shirt: Rgba<f32>,
    pub skin: Rgba<f32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Seated {
    pub player: Id,
    pub seat: usize,
}

#[derive(HasId, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Player {
    pub id: Id,
    pub name: String,
    pub pos: Position,
    pub fishing_state: FishingState,
    pub fish_in_hands: Option<FishType>,
    pub boat_level: u8,
    pub colors: PlayerColors,
    pub seated: Option<Seated>,
    pub inventory: Vec<FishType>,
}

impl Player {
    pub fn new(id: Id, pos: Vec2<f32>) -> Self {
        Self {
            id,
            name: String::new(),
            pos: Position {
                pos,
                vel: Vec2::ZERO,
                rot: global_rng().gen_range(0.0..2.0 * f32::PI),
                w: 0.0,
            },
            fishing_state: FishingState::Idle,
            fish_in_hands: None,
            boat_level: 0,
            colors: {
                let saturation = 0.7;
                let value = 0.7;
                let top = Hsva::new(global_rng().gen(), saturation, value, 1.0).into();
                PlayerColors {
                    hat: top,
                    pants: Hsva::new(global_rng().gen(), saturation, value, 1.0).into(),
                    shirt: top,
                    skin: Rgba::WHITE,
                }
            },
            seated: None,
            inventory: Vec::new(),
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
                FishingState::Waiting(_) => Some(1.0),
                _ => None,
            };
            if let Some(time) = time {
                *self.player_timings.entry(player.id).or_default() += delta_time / time;
            } else {
                self.player_timings.remove(&player.id);
            }
            let effect = self.boat_sound_effects.entry(player.id).or_insert_with(|| {
                let mut effect = self.assets.sounds.boat_moving.effect();
                effect.set_volume(0.0);
                effect.set_max_distance(10.0);
                effect.play();
                effect
            });

            let Some(pos) = self.interpolated.get(&player.id) else { continue };
            let pos = pos.get();
            effect.set_position(pos.pos.extend(0.0).map(|x| x as f64));
            if player.seated.is_some() || Map::get().get_height(pos.pos) > 0.0 {
                effect.set_volume(0.0);
            } else {
                effect.set_volume(pos.vel.len() as f64 / 2.0);
            }
        }
        for id in self.boat_sound_effects.keys().copied().collect::<Vec<_>>() {
            if model.players.get(&id).is_none() {
                self.boat_sound_effects.remove(&id).unwrap().stop();
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
    fn draw_player_character(
        &self,
        framebuffer: &mut ugli::Framebuffer,
        player: &Player,
        character_pos: Vec3<f32>,
    ) {
        // TODO: Add crabs
        let mut matrix = Mat4::translate(character_pos)
            * Mat4::rotate_z(
                (self.camera.eye_pos().xy() - character_pos.xy()).arg() + f32::PI / 2.0,
            );
        let rot = if Map::get().get_height(character_pos.xy()) > SHORE_HEIGHT {
            (self.time * 10.0).sin() * player.pos.vel.len().min(1.0) * 0.1
        } else {
            0.0
        };
        matrix *= Mat4::translate(vec3(0.0, 0.0, rot.abs())) * Mat4::rotate_y(rot);
        let body_matrix =
            matrix * Mat4::scale(vec3(1.0, 0.0, 2.0) * 0.25) * Mat4::translate(vec3(0.0, 0.0, 1.0));
        let (skin, shirt) = if player.fish_in_hands.is_some() {
            (
                &self.assets.player.skin_holding,
                &self.assets.player.shirt_holding,
            )
        } else if player.fishing_state != FishingState::Idle {
            (&self.assets.player.skin_fishing, &self.assets.player.shirt)
        } else {
            (&self.assets.player.skin, &self.assets.player.shirt)
        };
        self.draw_quad(
            framebuffer,
            body_matrix,
            &self.assets.player.pants,
            player.colors.pants,
        );
        self.draw_quad(
            framebuffer,
            body_matrix,
            &self.assets.player.hat,
            player.colors.hat,
        );
        self.draw_quad(framebuffer, body_matrix, shirt, player.colors.shirt);
        self.draw_quad(framebuffer, body_matrix, skin, player.colors.skin);
        self.draw_quad(
            framebuffer,
            body_matrix,
            &self.assets.player.eyes,
            Rgba::WHITE,
        );

        let mut fishing_rod_rot = None;
        let mut bobber = None;
        match &player.fishing_state {
            FishingState::Idle => {}
            FishingState::Spinning => {
                fishing_rod_rot = Some(-0.2);
            }
            FishingState::Casting(target_pos) => {
                let t = self
                    .player_timings
                    .get(&player.id)
                    .copied()
                    .unwrap_or(0.0)
                    .min(1.0);
                fishing_rod_rot = Some(t);

                // Parabolic bobber throw
                let delta = *target_pos - character_pos.xy();
                let length = delta.len();
                let direction = delta / length;
                let height_parameter = 7.5;
                let height = (1.0 - t) * (height_parameter * t + 1.0);
                let pos = character_pos.xy() + direction * t * length;
                bobber = Some(pos.extend(height));
            }
            FishingState::Waiting(bobber_pos) => {
                let t = self.player_timings.get(&player.id).copied().unwrap_or(0.0);
                {
                    let t = t.min(1.0);
                    let smoothstep = 3.0 * t * t - 2.0 * t * t * t;
                    let rot = (1.0 - smoothstep) * 0.5 + 0.5;
                    fishing_rod_rot = Some(rot);
                }

                // Damped oscillation
                let gamma = 1.0; // damping coefficient
                let frequency = 1.5 * f32::PI;
                // Shift to make it start at 0 depth
                let t = (t - f32::PI / 2.0 / frequency).abs();
                let amplitude = -(-gamma * t).exp() * 0.4;
                let bobber_depth = amplitude * (frequency * t).cos();
                bobber = Some(bobber_pos.extend(bobber_depth));
            }
            FishingState::PreReeling { bobber_pos, .. } => {
                fishing_rod_rot = Some(0.5);
                bobber = Some(bobber_pos.extend(0.0));
            }
            FishingState::Reeling { bobber_pos, .. } => {
                let t = self
                    .player_timings
                    .get(&player.id)
                    .copied()
                    .unwrap_or(0.0)
                    .min(1.0);
                // Damped oscillation
                let gamma = 3.0; // damping coefficient
                let frequency = 2.0 * f32::PI;
                let amplitude = (-gamma * t).exp() * 0.5;
                let rot = -amplitude * (frequency * t).cos() + 1.0;
                fishing_rod_rot = Some(rot);

                // Damped oscillation
                let gamma = 3.0; // damping coefficient
                let frequency = 4.0 * f32::PI;
                let amplitude = (-gamma * t).exp() * 0.2;
                let bobber_depth = amplitude * (frequency * t).cos() - 0.2;
                bobber = Some(bobber_pos.extend(bobber_depth));
            }
            FishingState::Attached(id) => {
                if let Some(player) = self.model.get().players.get(id) {
                    fishing_rod_rot = Some(1.0);
                    if let Some(p) = self.interpolated.get(id) {
                        bobber = Some(p.get().pos.extend(0.5));
                    }
                }
            }
        }
        // Draw fishing rod
        if let Some(rot) = fishing_rod_rot {
            let texture = &self.assets.fishing_rod;
            let mirrored = bobber
                .map(|bobber| bobber.x < character_pos.x)
                .unwrap_or(false);
            let fishing_rod_matrix = matrix
                * Mat4::translate(vec3(0.0, -0.1, 0.4))
                * Mat4::scale(vec3(if mirrored { -1.0 } else { 1.0 }, 1.0, 1.0))
                * Mat4::rotate_y(rot)
                * Mat4::scale(vec3(
                    texture.size().x as f32 / texture.size().y as f32,
                    1.0,
                    1.0,
                ))
                * Mat4::translate(-vec3(0.3, 0.5, 0.11).map(|x| x * 2.0 - 1.0));
            self.draw_quad(framebuffer, fishing_rod_matrix, texture, Rgba::WHITE);

            // Bobber
            if let Some(bobber_pos) = bobber {
                let fishing_rod_pos = (fishing_rod_matrix * vec4(0.0, 0.0, 1.0, 1.0)).xyz();
                ugli::draw(
                    framebuffer,
                    &self.assets.shaders.obj,
                    ugli::DrawMode::LineStrip { line_width: 1.0 },
                    ugli::instanced(
                        &ugli::VertexBuffer::new_dynamic(self.geng.ugli(), {
                            const N: i32 = 10;
                            (0..=N)
                                .map(|i| {
                                    let t = i as f32 / N as f32;
                                    ObjVertex {
                                        a_v: fishing_rod_pos * (1.0 - t)
                                            + bobber_pos * t
                                            + vec3(
                                                0.0,
                                                0.0,
                                                (1.0 - (t * 2.0 - 1.0).sqr()) * {
                                                    if matches!(
                                                        player.fishing_state,
                                                        FishingState::Casting(_)
                                                    ) {
                                                        let t = self
                                                            .player_timings
                                                            .get(&player.id)
                                                            .copied()
                                                            .unwrap_or(0.0)
                                                            .min(1.0);
                                                        1.0 - t * 2.0
                                                    } else {
                                                        -1.0
                                                    }
                                                },
                                            ),
                                        a_uv: Vec2::ZERO,
                                        a_vn: Vec3::ZERO,
                                    }
                                })
                                .collect()
                        }),
                        &ugli::VertexBuffer::new_dynamic(
                            self.geng.ugli(),
                            vec![ObjInstance {
                                i_model_matrix: Mat4::identity(),
                            }],
                        ),
                    ),
                    (
                        ugli::uniforms! {
                            u_color: Rgba::WHITE,
                            u_texture: &self.white_texture,
                        },
                        geng::camera3d_uniforms(&self.camera, self.framebuffer_size),
                    ),
                    ugli::DrawParameters {
                        depth_func: Some(ugli::DepthFunc::Less),
                        ..default()
                    },
                );
                let texture = &self.assets.bobber;
                self.draw_quad(
                    framebuffer,
                    Mat4::translate(bobber_pos)
                        * Mat4::scale_uniform(0.1)
                        * Mat4::rotate_x(-self.camera.rot_v)
                        * Mat4::scale(vec3(
                            texture.size().x as f32 / texture.size().y as f32,
                            1.0,
                            1.0,
                        ))
                        * Mat4::translate(vec3(0.0, 0.0, 0.5)),
                    texture,
                    Rgba::WHITE,
                );
            }
        }

        if let Some(fish) = player.fish_in_hands {
            self.draw_texture(
                framebuffer,
                character_pos + vec3(0.0, 0.0, 1.0),
                0.25,
                &self.assets.fishes[fish].texture,
                vec2(0.0, -1.0),
            )
        }

        if self.show_names {
            let ui_cam = geng::Camera2d {
                center: Vec2::ZERO,
                rotation: 0.0,
                fov: self.camera.distance * 2.0,
            };
            if let Some(screen) = self
                .camera
                .world_to_screen(self.framebuffer_size, character_pos + vec3(0.0, 0.0, 1.5))
            {
                self.draw_text(
                    framebuffer,
                    &ui_cam,
                    &player.name,
                    ui_cam.screen_to_world(self.framebuffer_size, screen),
                );
            }
        }
    }
    fn draw_player(&self, framebuffer: &mut ugli::Framebuffer, player: &Player, pos: &Position) {
        if let Some(seated) = player.seated {
            if let Some(captain) = self.model.get().players.get(&seated.player) {
                let boat_type_index = captain.boat_level.max(1) as usize - 1;
                let ship = &self.assets.ships[boat_type_index];
                let pos = if captain.id == self.player_id {
                    self.player.pos.clone()
                } else {
                    let Some(pos) = self.interpolated.get(&captain.id) else { return };
                    pos.get()
                };
                let model_matrix = Mat4::translate(pos.pos.extend(0.0))
                    * Mat4::rotate_z(pos.rot)
                    * Mat4::scale_uniform(self.assets.config.boat_types[boat_type_index].scale);
                self.draw_player_character(
                    framebuffer,
                    player,
                    (model_matrix * ship.seats[seated.seat].extend(1.0)).xyz(),
                );
            }
            return;
        }
        let height = Map::get().get_height(pos.pos);
        if height < SHORE_HEIGHT {
            let boat_type_index = player.boat_level.max(1) as usize - 1;
            let model_matrix = Mat4::translate(pos.pos.extend(0.0))
                * Mat4::rotate_z(pos.rot)
                * Mat4::scale_uniform(self.assets.config.boat_types[boat_type_index].scale);
            let ship = &self.assets.ships[boat_type_index];
            let obj = &ship.obj;
            for mesh in &obj.meshes {
                ugli::draw(
                    framebuffer,
                    &self.assets.shaders.obj,
                    ugli::DrawMode::Triangles,
                    ugli::instanced(
                        &mesh.geometry,
                        &ugli::VertexBuffer::new_dynamic(
                            self.geng.ugli(),
                            vec![ObjInstance {
                                i_model_matrix: model_matrix,
                            }],
                        ),
                    ),
                    (
                        ugli::uniforms! {
                            u_color: mesh.material.diffuse_color,
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
            self.draw_player_character(
                framebuffer,
                player,
                (model_matrix * ship.seats[0].extend(1.0)).xyz(),
            );
        } else {
            self.draw_player_character(framebuffer, player, pos.pos.extend(height));
        }
    }
}
