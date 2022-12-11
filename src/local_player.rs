use super::*;

pub const MAX_LINE_LEN: f32 = 7.0;

pub enum PlayerMovementControl {
    GoTo(Vec2<f32>),
    GoDirection(Vec2<f32>),
}

impl Game {
    pub fn update_my_player(&mut self, delta_time: f32) {
        let in_water = Map::get().get_height(self.player.pos.pos) < SHORE_HEIGHT;
        let mut player_radius = 1.0;
        if self.player.boat_level > 0 && in_water {
            player_radius *=
                self.assets.config.boat_types[(self.player.boat_level - 1) as usize].scale;
        }
        self.camera.pos = self.player.pos.pos.extend(0.0);
        if let Some(seated) = self.player.seated {
            if let Some(other) = self.model.get().players.get(&seated.player) {
                if let Some(pos) = self.interpolated.get(&other.id) {
                    self.camera.pos = pos.get().pos.extend(0.0);
                }
                self.player.pos.pos = other.pos.pos;
                if Map::get().get_height(other.pos.pos) > SHORE_HEIGHT {
                    self.player.seated = None;
                }
            } else {
                self.player.seated = None;
                self.player.pos.pos = Vec2::ZERO;
                self.player.pos.vel = Vec2::ZERO;
            }
        }
        if self.player.seated.is_none() {
            let mut wasd = Vec2::<f32>::ZERO;
            if self.geng.window().is_key_pressed(geng::Key::W)
                || self.geng.window().is_key_pressed(geng::Key::Up)
            {
                wasd.y += 1.0;
            }
            if self.geng.window().is_key_pressed(geng::Key::A)
                || self.geng.window().is_key_pressed(geng::Key::Left)
            {
                wasd.x -= 1.0;
            }
            if self.geng.window().is_key_pressed(geng::Key::S)
                || self.geng.window().is_key_pressed(geng::Key::Down)
            {
                wasd.y -= 1.0;
            }
            if self.geng.window().is_key_pressed(geng::Key::D)
                || self.geng.window().is_key_pressed(geng::Key::Right)
            {
                wasd.x += 1.0;
            }
            if wasd != Vec2::ZERO
                || matches!(self.player_control, PlayerMovementControl::GoDirection(_))
            {
                self.player_control = PlayerMovementControl::GoDirection(wasd);
            }

            let props = if !in_water {
                MovementProps {
                    max_speed: 2.0,
                    max_rotation_speed: 2.0,
                    angular_acceleration: 1.0,
                    acceleration: 10.0,
                    water: false,
                }
            } else {
                MovementProps {
                    max_speed: 2.0,
                    max_rotation_speed: 2.0,
                    angular_acceleration: 1.0,
                    acceleration: 1.0,
                    water: true,
                }
            };
            let target_pos = match self.player_control {
                PlayerMovementControl::GoTo(pos) => pos,
                PlayerMovementControl::GoDirection(dir) => {
                    self.player.pos.pos + dir * props.max_speed
                }
            };

            update_movement(&mut self.player.pos, target_pos, props.clone(), delta_time);
            if let FishingState::Attached(id) = self.player.fishing_state {
                if let Some(other_player) = self.model.get().players.get(&id) {
                    if let Some(p) = self.interpolated.get(&id) {
                        let delta_pos = self.player.pos.pos - p.get().pos;
                        const MIN_LINE_LEN: f32 = 4.0;
                        if delta_pos.len() > MIN_LINE_LEN {
                            update_movement(
                                &mut self.player.pos,
                                p.get().pos,
                                MovementProps {
                                    acceleration: props.acceleration * 2.0,
                                    ..props
                                },
                                delta_time,
                            );
                        }
                        if delta_pos.len() > MAX_LINE_LEN {
                            self.player.fishing_state = FishingState::Idle;
                            self.play_sound_for_everyone(
                                self.player.pos.pos,
                                SoundType::StopFishing,
                            );
                        }
                    }
                } else {
                    self.player.fishing_state = FishingState::Idle;
                    self.play_sound_for_everyone(self.player.pos.pos, SoundType::StopFishing);
                }
            }

            // handle collisions
            if in_water {
                for other_player in &self.model.get().players {
                    if other_player.id == self.player_id {
                        continue;
                    }
                    if other_player.seated.is_some() {
                        continue;
                    }
                    let land = |pos| Map::get().get_height(pos) > SHORE_HEIGHT;
                    if land(other_player.pos.pos) {
                        continue;
                    }
                    let Some(p) = self.interpolated.get(&other_player.id) else { continue };
                    let delta_pos = self.player.pos.pos - p.get().pos;
                    if delta_pos.len() < 2.0 * player_radius {
                        let n = delta_pos.normalize_or_zero();
                        let penetration = 2.0 * player_radius - delta_pos.len();
                        self.player.pos.pos += n * penetration;
                        self.player.pos.vel -= n * Vec2::dot(n, self.player.pos.vel).min(0.0);
                    }
                }
            }
            if let Some(bobber_pos) = self.player.fishing_state.bobber_pos() {
                let delta_pos = bobber_pos - self.player.pos.pos;
                if delta_pos.len() > MAX_LINE_LEN {
                    self.player.fishing_state = FishingState::Idle;
                    self.play_sound_for_everyone(self.player.pos.pos, SoundType::StopFishing);
                }
            }
            // collide with world edge
            if self.player.boat_level < 3 {
                let to_edge = vec_to(&self.map_geometry.edge_segments, self.player.pos.pos);
                if to_edge.len() < player_radius {
                    let n = -to_edge.normalize_or_zero();
                    let penetration = player_radius - to_edge.len();
                    self.player.pos.pos += n * penetration;
                    self.player.pos.vel -= n * Vec2::dot(n, self.player.pos.vel).min(0.0);
                }
            }
            // collide with shore
            {
                let to_shore = vec_to(&self.map_geometry.shore_segments, self.player.pos.pos);
                let player_radius = if in_water { player_radius } else { 0.3 };
                if to_shore.len() < player_radius {
                    let n = -to_shore.normalize_or_zero();
                    let penetration = player_radius - to_shore.len();
                    self.player.pos.pos += n * penetration;
                    self.player.pos.vel -= n * Vec2::dot(n, self.player.pos.vel).min(0.0);
                }
            }
            // collide with deep sea boundary
            if self.player.boat_level < 2 {
                let to_deep = vec_to(&self.map_geometry.deep_segments, self.player.pos.pos);
                let player_radius = if in_water { player_radius } else { 0.3 };
                if to_deep.len() < player_radius {
                    self.tutorial = "you need a bigger boat to travel the deep sea".to_owned();
                    self.tutorial_timer = 5.0;
                    let n = -to_deep.normalize_or_zero();
                    let penetration = player_radius - to_deep.len();
                    self.player.pos.pos += n * penetration;
                    self.player.pos.vel -= n * Vec2::dot(n, self.player.pos.vel).min(0.0);
                }
            }
            for &pos in itertools::chain![
                &self.assets.config.fish_shops,
                self.assets
                    .config
                    .boat_types
                    .iter()
                    .flat_map(|boat_type| &boat_type.shops)
            ] {
                let delta_pos = self.player.pos.pos - pos;
                const SHOP_R: f32 = 1.2;
                if delta_pos.len() < SHOP_R {
                    let n = delta_pos.normalize_or_zero();
                    let penetration = SHOP_R - delta_pos.len();
                    self.player.pos.pos += n * penetration;
                    self.player.pos.vel -= n * Vec2::dot(n, self.player.pos.vel).min(0.0);
                }
            }
        }

        // Fishing
        if let Some(time) = self.player_timings.get(&self.player_id) {
            if *time > 1.0 {
                let mut ignore = false;
                match self.player.fishing_state {
                    FishingState::Casting(bobber_pos) => {
                        let mut sound_type = None;
                        if Map::get().get_height(bobber_pos) < SHORE_HEIGHT
                            && Map::get().get_channel_value(3, bobber_pos) > 0.5
                        {
                            // This is water
                            self.player.fishing_state = FishingState::Waiting(bobber_pos);
                            sound_type = Some(SoundType::Splash);
                            self.splashes.push(Splash::new(bobber_pos));
                            self.tutorial = "left click to reel when the fish bites".to_owned();
                            self.tutorial_timer = 10.0;
                        } else {
                            // This is land/space
                            // TODO: make this code self explanatory
                            for fish in &self.model.get().fishes {
                                if (fish.pos.pos - bobber_pos).len() < 1.0 {
                                    self.caught_fish.insert(CaughtFish {
                                        id: fish.id,
                                        index: fish.index,
                                        player: self.player_id,
                                        lifetime: 0.0,
                                        caught_at: fish.pos.pos,
                                    });
                                    self.model.send(Message::Catch(fish.id));
                                    self.play_sound_for_everyone(fish.pos.pos, SoundType::Ding);
                                }
                            }
                            self.player.fishing_state = FishingState::Idle;
                        }
                        // TODO: make more comments
                        for other_player in &self.model.get().players {
                            if other_player.seated.is_some() {
                                continue;
                            }
                            let Some(p) = self.interpolated.get(&other_player.id) else { continue };
                            if (p.get().pos - bobber_pos).len() < player_radius {
                                if other_player.id == self.player_id {
                                    self.player.fishing_state = FishingState::Idle;
                                    self.play_sound_for_everyone(
                                        self.player.pos.pos,
                                        SoundType::StopFishing,
                                    );
                                } else {
                                    self.player.fishing_state =
                                        FishingState::Attached(other_player.id);
                                    self.player_control =
                                        PlayerMovementControl::GoDirection(Vec2::ZERO);
                                }
                            }
                        }
                        if let Some(sound_type) = sound_type {
                            self.play_sound_for_everyone(bobber_pos, sound_type);
                        }
                    }
                    FishingState::PreReeling { fish, bobber_pos } => {
                        self.player.fishing_state = FishingState::Reeling { fish, bobber_pos };
                    }
                    FishingState::Reeling { fish, bobber_pos } => {
                        self.player.fishing_state = FishingState::Waiting(bobber_pos);
                    }
                    FishingState::Waiting(_) => {
                        ignore = true;
                    }
                    _ => {}
                }
                if !ignore {
                    self.player_timings.remove(&self.player_id);
                }
            }
        }

        self.player.fish_in_hands = self
            .hovered_inventory_slot
            .and_then(|index| self.inventory.get(index).copied());
    }
}
