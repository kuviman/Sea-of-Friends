use super::*;

impl Game {
    pub fn draw_inventory(&mut self, framebuffer: &mut ugli::Framebuffer) {
        let camera = geng::Camera2d {
            center: Vec2::ZERO,
            rotation: 0.0,
            fov: 20.0,
        };
        let size =
            (self.inventory.len() as f32 - 1.0) * 10.0 / self.assets.config.inventory_size as f32;
        let mut hovered = None;
        for (index, &fish) in self.inventory.iter().enumerate() {
            let pos = vec2(
                (index as f32 / (self.inventory.len() - 1).max(1) as f32) * size - size / 2.0,
                -camera.fov / 2.0,
            );
            let texture = &self.assets.fishes[fish];
            let fish_card = draw_2d::TexturedQuad::new(
                AABB::point(Vec2::ZERO)
                    .extend_symmetric(vec2(texture.size().x as f32 / texture.size().y as f32, 1.0)),
                texture,
            )
            .transform(Mat3::rotate(f32::PI / 2.0))
            .translate(pos);
            self.geng.draw_2d(framebuffer, &camera, &fish_card);
            if fish_card.bounding_box().contains(camera.screen_to_world(
                self.framebuffer_size,
                self.geng.window().mouse_pos().map(|x| x as f32),
            )) {
                hovered = Some((index, texture, pos));
            }
        }
        self.hovered_inventory_slot = None;
        if let Some((index, texture, pos)) = hovered {
            self.hovered_inventory_slot = Some(index);
            let fish_card = draw_2d::TexturedQuad::new(
                AABB::point(Vec2::ZERO).extend_symmetric(
                    vec2(texture.size().x as f32 / texture.size().y as f32, 1.0) * 1.5,
                ),
                texture,
            )
            .transform(Mat3::rotate(f32::PI / 2.0))
            .translate(pos);
            self.geng.draw_2d(framebuffer, &camera, &fish_card);
        }
    }
}
