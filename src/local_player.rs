use super::*;

pub enum PlayerMovementControl {
    GoTo(Vec2<f32>),
    GoDirection(Vec2<f32>),
}

impl Game {
    pub fn update_my_player(&mut self, delta_time: f32) {
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

        let props = if Map::get().get_height(self.player.pos.pos) > 0.0 {
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
            PlayerMovementControl::GoDirection(dir) => self.player.pos.pos + dir * props.max_speed,
        };
        update_movement(&mut self.player.pos, target_pos, props, delta_time);

        // handle collisions
        let player_radius = 1.0;
        if Map::get().get_height(self.player.pos.pos) < 0.0 {
            for other_player in &self.model.get().players {
                if other_player.id == self.player_id {
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
        if let FishingState::Attached(id) = self.player.fishing_state {
            if let Some(other_player) = self.model.get().players.get(&id) {
                if let Some(p) = self.interpolated.get(&id) {
                    let delta_pos = self.player.pos.pos - p.get().pos;
                    const LINE_LEN: f32 = 5.0;
                    if delta_pos.len() > LINE_LEN {
                        let n = delta_pos.normalize_or_zero();
                        let penetration = delta_pos.len() - LINE_LEN;
                        self.player.pos.pos -= n * penetration;
                        self.player.pos.vel -= n * Vec2::dot(n, self.player.pos.vel).max(0.0);
                    }
                }
            } else {
                self.player.fishing_state = FishingState::Idle;
            }
        }
        if self.player.boat_level < 3 {
            let to_edge = vec_to(&self.map_geometry.edge_segments, self.player.pos.pos);
            if to_edge.len() < player_radius {
                let n = -to_edge.normalize_or_zero();
                let penetration = player_radius - to_edge.len();
                self.player.pos.pos += n * penetration;
                self.player.pos.vel -= n * Vec2::dot(n, self.player.pos.vel).min(0.0);
            }
        }
        if self.player.boat_level < 1 {
            let to_shore = vec_to(&self.map_geometry.shore_segments, self.player.pos.pos);
            let player_radius = if Map::get().get_height(self.player.pos.pos) < 0.0 {
                player_radius
            } else {
                0.3
            };
            if to_shore.len() < player_radius {
                let n = -to_shore.normalize_or_zero();
                let penetration = player_radius - to_shore.len();
                self.player.pos.pos += n * penetration;
                self.player.pos.vel -= n * Vec2::dot(n, self.player.pos.vel).min(0.0);
            }
        }
        if self.player.boat_level < 2 {
            let to_deep = vec_to(&self.map_geometry.deep_segments, self.player.pos.pos);
            let player_radius = if Map::get().get_height(self.player.pos.pos) < 0.0 {
                player_radius
            } else {
                0.3
            };
            if to_deep.len() < player_radius {
                let n = -to_deep.normalize_or_zero();
                let penetration = player_radius - to_deep.len();
                self.player.pos.pos += n * penetration;
                self.player.pos.vel -= n * Vec2::dot(n, self.player.pos.vel).min(0.0);
            }
        }
        for &pos in &self.assets.config.fish_shops {
            let delta_pos = self.player.pos.pos - pos;
            const SHOP_R: f32 = 1.0;
            if delta_pos.len() < SHOP_R {
                let n = delta_pos.normalize_or_zero();
                let penetration = SHOP_R - delta_pos.len();
                self.player.pos.pos += n * penetration;
                self.player.pos.vel -= n * Vec2::dot(n, self.player.pos.vel).min(0.0);
            }
        }

        // Fishing
        if let Some(time) = self.player_timings.get(&self.player_id) {
            if *time > 1.0 {
                let mut ignore = false;
                match self.player.fishing_state {
                    FishingState::Casting(bobber_pos) => {
                        if Map::get().get_height(bobber_pos) < 0.0 {
                            self.player.fishing_state = FishingState::Waiting(bobber_pos);
                        } else {
                            self.player.fishing_state = FishingState::Idle;
                        }
                        for other_player in &self.model.get().players {
                            let Some(p) = self.interpolated.get(&other_player.id) else { continue };
                            if (p.get().pos - bobber_pos).len() < player_radius {
                                if other_player.id == self.player_id {
                                    self.player.fishing_state = FishingState::Idle;
                                } else {
                                    self.player.fishing_state =
                                        FishingState::Attached(other_player.id);
                                }
                            }
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
