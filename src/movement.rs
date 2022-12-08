use super::*;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Position {
    pub pos: Vec2<f32>,
    pub vel: Vec2<f32>,
    pub rot: f32,
    pub w: f32,
}

pub struct MovementProps {
    pub max_speed: f32,
    pub max_rotation_speed: f32,
    pub angular_acceleration: f32,
    pub acceleration: f32,
    pub water: bool,
}

pub fn update_movement(
    pos: &mut Position,
    target_pos: Vec2<f32>,
    props: MovementProps,
    delta_time: f32,
) {
    let delta_pos = target_pos - pos.pos;
    let target_w = (normalize_angle(
        if delta_pos.len() > 0.1 {
            delta_pos.arg()
        } else {
            pos.rot
        } - pos.rot,
    ) * 10.0)
        .clamp_abs(props.max_rotation_speed);
    pos.w += (target_w - pos.w).clamp_abs(props.angular_acceleration);
    pos.rot += pos.w * delta_time;
    let mut target_vel = delta_pos.clamp_len(..=props.max_speed);
    if props.water {
        target_vel *= Vec2::dot(
            delta_pos.normalize_or_zero(),
            vec2(1.0, 0.0).rotate(pos.rot),
        )
        .max(0.0);
    }
    pos.vel += (target_vel - pos.vel).clamp_len(..=props.acceleration * delta_time);
    pos.pos += pos.vel * delta_time;
}
