use super::*;

impl Game {
    pub fn draw_inventory(&self, framebuffer: &mut ugli::Framebuffer) {
        let camera = geng::Camera2d {
            center: Vec2::ZERO,
            rotation: 0.0,
            fov: 10.0,
        };
        let size =
            (self.inventory.len() as f32 - 1.0) * 10.0 / self.assets.config.inventory_size as f32;
        for (index, &fish) in self.inventory.iter().enumerate() {
            let texture = &self.assets.fishes[fish];
            self.geng.draw_2d(
                framebuffer,
                &camera,
                &draw_2d::TexturedQuad::new(
                    AABB::point(Vec2::ZERO).extend_symmetric(vec2(
                        texture.size().x as f32 / texture.size().y as f32,
                        1.0,
                    )),
                    texture,
                )
                .transform(Mat3::rotate(f32::PI / 2.0))
                .translate(vec2(
                    (index as f32 / (self.inventory.len() - 1).max(1) as f32) * size - size / 2.0,
                    -5.0,
                )),
            );
        }
    }
}
