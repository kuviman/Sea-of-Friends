use super::*;

impl Game {
    pub fn play_sound_for_everyone(&self, pos: Vec2<f32>, sound_type: SoundType) {
        self.play_sound(pos, sound_type);
        self.model.send(Message::Broadcast(Event::Sound {
            player: self.player_id,
            sound_type,
            pos,
        }));
    }

    pub fn play_sound(&self, pos: Vec2<f32>, sound_type: SoundType) {
        let sounds: &[geng::Sound] = match sound_type {
            SoundType::Splash => &self.assets.sounds.splash,
            SoundType::Casting => &self.assets.sounds.casting,
            SoundType::Ding => std::slice::from_ref(&self.assets.sounds.ding),
            SoundType::StopFishing => std::slice::from_ref(&self.assets.sounds.stop_fishing),
            SoundType::ShowFish => std::slice::from_ref(&self.assets.sounds.show_fish),
            SoundType::Whip => &self.assets.sounds.whip,
            SoundType::DropFishLand => std::slice::from_ref(&self.assets.sounds.drop_fish_land),
            SoundType::DropFishWater => std::slice::from_ref(&self.assets.sounds.drop_fish_water),
            SoundType::EnterBoat => std::slice::from_ref(&self.assets.sounds.enter_boat),
            SoundType::ExitBoat => std::slice::from_ref(&self.assets.sounds.exit_boat),
            SoundType::SellFish => std::slice::from_ref(&self.assets.sounds.sell_fish),
            SoundType::UpgradeBoat => std::slice::from_ref(&self.assets.sounds.upgrade_boat),
        };
        let mut effect = sounds.choose(&mut global_rng()).unwrap().effect();
        effect.set_position(pos.map(|x| x as f64).extend(0.0));
        effect.set_max_distance(10.0);
        effect.play();
    }
}
