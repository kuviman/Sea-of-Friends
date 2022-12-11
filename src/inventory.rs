use super::*;

impl Game {
    pub fn draw_inventory(&mut self, framebuffer: &mut ugli::Framebuffer) {
        let camera = geng::Camera2d {
            center: Vec2::ZERO,
            rotation: 0.0,
            fov: 20.0,
        };

        for fish in &self.caught_fish {
            let (pos, rot) = if fish.player == self.player_id {
                // Fly to the inventory
                let Some(pos) = self.camera.world_to_screen(
                        framebuffer.size().map(|x| x as f32),
                        fish.caught_at.extend(0.0))
                    else { continue; };
                let offset = framebuffer.size().x as f32 / 2.0;
                let pos = vec2(pos.x - offset, pos.y);
                let t = 1.0 - fish.lifetime.min(1.0);
                let height_parameter = 5.0;
                let height = pos.y * (0.0 - t) * (height_parameter * (t - 1.0) - 1.0);
                let rot = self.time.sin() + t * 3.0;
                (vec2(t * pos.x + offset, height), rot)
            } else {
                // Fly to the player
                let Some(target) = self.interpolated.get(&fish.player) else { continue; };
                let target = target.get().pos;
                let delta = target - fish.caught_at;
                let length = delta.len();
                let direction = delta / length;
                let t = fish.lifetime.min(1.0);
                let height_parameter = 5.0;
                let height = (0.0 - t) * (height_parameter * (t - 1.0) - 1.0);
                let pos = fish.caught_at + direction * t * length;
                let pos = vec3(pos.x, pos.y, height);
                let Some(pos) = self.camera.world_to_screen(framebuffer.size().map(|x| x as f32), pos) else {
                    continue;
                };
                let rot = self.time.sin() + t * 3.0;
                (pos, rot)
            };

            let texture = &self.assets.fishes[fish.index].texture;
            let fish_card = draw_2d::TexturedQuad::new(
                AABB::point(Vec2::ZERO).extend_symmetric(
                    vec2(texture.size().x as f32 / texture.size().y as f32, 1.0) * 10.0,
                ),
                texture,
            )
            .transform(Mat3::rotate(rot))
            .translate(pos);
            self.geng
                .draw_2d(framebuffer, &geng::PixelPerfectCamera, &fish_card);
        }

        let size =
            (self.inventory.len() as f32 - 1.0) * 10.0 / self.assets.config.inventory_size as f32;
        let mut hovered = None;
        for (index, &fish) in self.inventory.iter().enumerate() {
            let pos = vec2(
                (index as f32 / (self.inventory.len() - 1).max(1) as f32) * size - size / 2.0,
                -camera.fov / 2.0,
            );
            let texture = &self.assets.fishes[fish].texture;
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

        self.geng.draw_2d(
            framebuffer,
            &camera,
            &draw_2d::Text::unit(
                &**self.geng.default_font(),
                format!("$ {}", self.money),
                Rgba::WHITE,
            )
            .translate(vec2(0.0, camera.fov / 2.0 - 1.0)),
        );
        self.geng.draw_2d(
            framebuffer,
            &camera,
            &draw_2d::Text::unit(
                &**self.geng.default_font(),
                format!(
                    "fishdex: {}/{}",
                    self.fishdex.len(),
                    self.assets.fishes.len(),
                ),
                Rgba::WHITE,
            )
            .scale_uniform(0.5)
            .translate(vec2(10.0, camera.fov / 2.0 - 1.0)),
        );
    }
}
