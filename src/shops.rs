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
                &self.assets.shops.fish,
                &self.assets.shops.fish,
                &self.assets.shops.fish,
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
}
