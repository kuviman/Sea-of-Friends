use super::*;

#[derive(HasId, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Player {
    pub id: Id,
    pub pos: Position,
}

impl Player {
    pub fn new(id: Id, pos: Vec2<f32>) -> Self {
        Self {
            id,
            pos: Position {
                pos,
                vel: Vec2::ZERO,
                rot: global_rng().gen_range(0.0..2.0 * f32::PI),
                w: 0.0,
            },
        }
    }
}

impl Game {
    pub fn draw_players(&self, framebuffer: &mut ugli::Framebuffer) {
        let model = self.model.get();
        self.draw_player(framebuffer, &self.player.pos, self.player.fishing_pos);
        for player in &model.players {
            if player.id == self.player_id {
                continue;
            }
            let Some(pos) = self.interpolated.get(&player.id) else { continue };
            let pos = pos.get();
            self.draw_player(framebuffer, &pos, None);
        }
    }
    fn draw_player(
        &self,
        framebuffer: &mut ugli::Framebuffer,
        pos: &Position,
        fishing_pos: Option<Vec2<f32>>,
    ) {
        let model_matrix = Mat4::translate(pos.pos.extend(0.0)) * Mat4::rotate_z(pos.rot);
        for mesh in &self.assets.boat.meshes {
            ugli::draw(
                framebuffer,
                &self.assets.shaders.obj,
                ugli::DrawMode::Triangles,
                &mesh.geometry,
                (
                    ugli::uniforms! {
                        u_model_matrix: model_matrix,
                        u_texture: mesh.material.texture.as_deref().unwrap_or(&self.white_texture),
                    },
                    geng::camera3d_uniforms(&self.camera, self.framebuffer_size),
                ),
                ugli::DrawParameters {
                    depth_func: Some(ugli::DepthFunc::Less),
                    ..default()
                },
            );
        }
        self.draw_quad(
            framebuffer,
            Mat4::translate(pos.pos.extend(0.0))
                * Mat4::rotate_x(-self.camera.rot_v)
                * Mat4::scale(vec3(1.0, 0.0, 2.0) * 0.25)
                * Mat4::translate(vec3(0.0, 0.0, 1.0)),
            &self.assets.player,
        );
        if let Some(pos) = fishing_pos {
            self.draw_quad(
                framebuffer,
                Mat4::translate(pos.extend(0.0))
                    * Mat4::scale_uniform(0.1)
                    * Mat4::rotate_x(-self.camera.rot_v),
                &self.assets.bobber,
            );
        }
    }
}
