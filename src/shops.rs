use super::*;

impl Game {
    pub fn draw_shops(&self, framebuffer: &mut ugli::Framebuffer) {
        for &pos in &self.assets.config.fish_shops {
            self.draw_texture(
                framebuffer,
                pos.extend(Map::get().get_height(pos)),
                1.0,
                &self.assets.shops.fish,
                vec2(0.0, -1.0),
            );
        }
        for (index, boat_type) in self.assets.config.boat_types.iter().enumerate() {
            let texture = [
                &self.assets.shops.itsboats,
                &self.assets.shops.big_boat_shop,
                &self.assets.shops.air_shop,
            ][index];
            for &pos in &boat_type.shops {
                self.draw_texture(
                    framebuffer,
                    pos.extend(Map::get().get_height(pos)),
                    1.0,
                    texture,
                    vec2(0.0, -1.0),
                );
            }
        }
    }

    pub fn can_sell_fish(&self) -> bool {
        let fishing_shop_distance = self
            .assets
            .config
            .fish_shops
            .iter()
            .map(|&pos| r32((pos - self.player.pos.pos).len()))
            .min()
            .unwrap()
            .raw();
        fishing_shop_distance < SHOPPING_DISTANCE
    }

    pub fn is_hovering_boat_shop(&self) -> Option<(usize, &BoatConfig)> {
        for (index, boat_type) in self.assets.config.boat_types.iter().enumerate() {
            let boat_level = index as u8 + 1;
            if let Some(distance) = boat_type
                .shops
                .iter()
                .filter(|&&pos| {
                    let pos = pos.extend(Map::get().get_height(pos));
                    let ray = self.camera.pixel_ray(
                        self.framebuffer_size,
                        self.geng.window().mouse_pos().map(|x| x as f32),
                    );
                    Vec3::cross(ray.dir.normalize_or_zero(), pos - ray.from).len() < 1.0
                })
                .map(|&pos| r32((pos - self.player.pos.pos).len()))
                .min()
            {
                if distance.raw() < 2.0 && self.player.boat_level < boat_level {
                    return Some((index, boat_type));
                }
            }
        }
        None
    }
}
