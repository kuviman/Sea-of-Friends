use super::*;

impl Game {
    pub fn play_my_sound(&self, pos: Vec3<f32>, sound_type: SoundType) {
        self.play_sound(pos, sound_type);
        self.model.send(Message::Broadcast(Event::Sound {
            player: self.player_id,
            sound_type,
            pos,
        }));
    }

    pub fn play_sound(&self, pos: Vec3<f32>, sound_type: SoundType) {
        let sounds: &[geng::Sound] = match sound_type {
            SoundType::BobberHit => &self.assets.sounds.bobber_hit,
            SoundType::Casting => &self.assets.sounds.casting,
        };
        let mut effect = sounds.choose(&mut global_rng()).unwrap().effect();
        effect.set_position(pos.map(|x| x as f64));
        effect.set_max_distance(10.0);
        effect.play();
    }
}
