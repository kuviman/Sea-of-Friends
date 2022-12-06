use super::*;

pub enum PlayerMovementControl {
    GoTo(Vec2<f32>),
    GoDirection(Vec2<f32>),
}

pub struct LocalPlayer {
    pub pos: Position,
    pub control: PlayerMovementControl,
    pub fishing_pos: Option<Vec2<f32>>,
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
            || matches!(self.player.control, PlayerMovementControl::GoDirection(_))
        {
            self.player.control = PlayerMovementControl::GoDirection(wasd);
        }

        let props = MovementProps {
            max_speed: 2.0,
            max_rotation_speed: 2.0,
            angular_acceleration: 1.0,
            acceleration: 1.0,
        };
        let target_pos = match self.player.control {
            PlayerMovementControl::GoTo(pos) => pos,
            PlayerMovementControl::GoDirection(dir) => self.player.pos.pos + dir * props.max_speed,
        };
        update_movement(&mut self.player.pos, target_pos, props, delta_time);

        // handle collisions
        for other_player in &self.model.get().players {
            if other_player.id == self.player_id {
                continue;
            }
            let Some(p) = self.interpolated.get(&other_player.id) else { continue };
            let delta_pos = self.player.pos.pos - p.get().pos;
            let r = 1.0;
            if delta_pos.len() < 2.0 * r {
                let n = delta_pos.normalize_or_zero();
                let penetration = 2.0 * r - delta_pos.len();
                self.player.pos.pos += n * penetration;
                self.player.pos.vel -= n * Vec2::dot(n, self.player.pos.vel).min(0.0);
            }
        }
    }
}
