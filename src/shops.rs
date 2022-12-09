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
        for &pos in &self.assets.config.small_boat_shops {
            self.draw_texture(
                framebuffer,
                pos.extend(Map::get().get_height(pos)),
                1.0,
                &self.assets.shops.fish,
                vec2(0.0, -1.0),
            );
        }
    }
}
