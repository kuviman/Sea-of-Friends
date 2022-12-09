use super::*;

pub fn normalize_angle(mut a: f32) -> f32 {
    while a > f32::PI {
        a -= 2.0 * f32::PI;
    }
    while a < -f32::PI {
        a += 2.0 * f32::PI;
    }
    a
}

pub fn make_repeated(texture: &mut ugli::Texture) {
    texture.set_wrap_mode(ugli::WrapMode::Repeat);
}

pub fn make_looped(sound: &mut geng::Sound) {
    sound.looped = true;
}
