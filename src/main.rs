use geng::prelude::*;

pub mod camera;
pub mod obj;
pub mod util;

pub use camera::*;
pub use obj::*;
pub use util::*;

#[derive(geng::Assets)]
pub struct Shaders {
    pub water: ugli::Program,
    pub obj: ugli::Program,
    pub obj2: ugli::Program,
}

#[derive(geng::Assets, Serialize, Deserialize)]
#[asset(json)]
pub struct Config {
    pub sea_color: Rgba<f32>,
}

#[derive(geng::Assets)]
pub struct Assets {
    pub shaders: Shaders,
    #[asset(path = "1391 Rowboat.obj")]
    pub boat: Obj,
    pub bobber: ugli::Texture,
    pub player: ugli::Texture,
    pub config: Config,
    #[asset(path = "PerlinNoise.png", postprocess = "make_repeated")]
    pub surface_noise: ugli::Texture,
    #[asset(path = "WaterDistortion.png", postprocess = "make_repeated")]
    pub distort_noise: ugli::Texture,
}

pub struct Player {
    pub pos: Vec2<f32>,
    pub rot: f32,
    pub target_pos: Vec2<f32>,
    pub fishing_pos: Option<Vec2<f32>>,
}

pub struct Game {
    framebuffer_size: Vec2<f32>,
    camera: Camera,
    geng: Geng,
    time: f32,
    assets: Rc<Assets>,
    white_texture: ugli::Texture,
    player: Player,
    quad: ugli::VertexBuffer<ObjVertex>,
}

impl Game {
    pub fn new(geng: &Geng, assets: &Rc<Assets>) -> Self {
        Self {
            time: 0.0,
            framebuffer_size: vec2(1.0, 1.0),
            geng: geng.clone(),
            assets: assets.clone(),
            camera: Camera {
                fov: f32::PI / 2.0,
                pos: Vec3::ZERO,
                distance: 10.0,
                rot_h: 0.0,
                rot_v: f32::PI / 4.0,
            },
            white_texture: ugli::Texture::new_with(geng.ugli(), vec2(1, 1), |_| Rgba::WHITE),
            player: Player {
                pos: Vec2::ZERO,
                target_pos: Vec2::ZERO,
                rot: 0.0,
                fishing_pos: None,
            },
            quad: ugli::VertexBuffer::new_static(
                geng.ugli(),
                vec![
                    ObjVertex {
                        a_v: vec3(-1.0, 0.0, -1.0),
                        a_uv: vec2(0.0, 0.0),
                        a_vn: Vec3::ZERO,
                    },
                    ObjVertex {
                        a_v: vec3(1.0, 0.0, -1.0),
                        a_uv: vec2(1.0, 0.0),
                        a_vn: Vec3::ZERO,
                    },
                    ObjVertex {
                        a_v: vec3(1.0, 0.0, 1.0),
                        a_uv: vec2(1.0, 1.0),
                        a_vn: Vec3::ZERO,
                    },
                    ObjVertex {
                        a_v: vec3(-1.0, 0.0, 1.0),
                        a_uv: vec2(0.0, 1.0),
                        a_vn: Vec3::ZERO,
                    },
                ],
            ),
        }
    }

    pub fn draw_quad(
        &self,
        framebuffer: &mut ugli::Framebuffer,
        matrix: Mat4<f32>,
        texture: &ugli::Texture,
    ) {
        ugli::draw(
            framebuffer,
            &self.assets.shaders.obj,
            ugli::DrawMode::TriangleFan,
            &self.quad,
            (
                ugli::uniforms! {
                    u_model_matrix: matrix,
                    u_texture: texture,
                },
                geng::camera3d_uniforms(&self.camera, self.framebuffer_size),
            ),
            ugli::DrawParameters {
                blend_mode: Some(ugli::BlendMode::default()),
                depth_func: Some(ugli::DepthFunc::Less),
                ..default()
            },
        );
    }

    pub fn draw_quad2(
        &self,
        framebuffer: &mut ugli::Framebuffer,
        matrix: Mat4<f32>,
        texture: &ugli::Texture,
    ) {
        ugli::draw(
            framebuffer,
            &self.assets.shaders.obj2,
            ugli::DrawMode::TriangleFan,
            &self.quad,
            (
                ugli::uniforms! {
                    u_model_matrix: matrix,
                    u_texture: texture,
                },
                geng::camera3d_uniforms(&self.camera, self.framebuffer_size),
            ),
            ugli::DrawParameters {
                blend_mode: Some(ugli::BlendMode::default()),
                depth_func: Some(ugli::DepthFunc::Less),
                ..default()
            },
        );
    }

    pub fn world_pos(&self, mouse_pos: Vec2<f32>) -> Vec2<f32> {
        let camera_ray = self.camera.pixel_ray(self.framebuffer_size, mouse_pos);
        camera_ray.from.xy() - camera_ray.dir.xy() * camera_ray.from.z / camera_ray.dir.z
    }
}

impl geng::State for Game {
    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        self.framebuffer_size = framebuffer.size().map(|x| x as f32);
        ugli::clear(
            framebuffer,
            Some(self.assets.config.sea_color),
            Some(1.0),
            None,
        );

        // Drawing player
        let model_matrix =
            Mat4::translate(self.player.pos.extend(0.0)) * Mat4::rotate_z(self.player.rot);
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
            Mat4::translate(self.player.pos.extend(0.0))
                * Mat4::rotate_x(-self.camera.rot_v)
                * Mat4::scale(vec3(1.0, 0.0, 2.0) * 0.25)
                * Mat4::translate(vec3(0.0, 0.0, 1.0)),
            &self.assets.player,
        );
        if let Some(pos) = self.player.fishing_pos {
            self.draw_quad(
                framebuffer,
                Mat4::translate(pos.extend(0.0))
                    * Mat4::scale_uniform(0.1)
                    * Mat4::rotate_x(-self.camera.rot_v),
                &self.assets.bobber,
            );
        }

        let mut depth_texture =
            ugli::Texture::new_uninitialized(self.geng.ugli(), framebuffer.size());
        {
            let mut depth_buffer = ugli::Renderbuffer::<ugli::DepthComponent>::new(
                self.geng.ugli(),
                framebuffer.size(),
            );
            let mut framebuffer = ugli::Framebuffer::new(
                self.geng.ugli(),
                ugli::ColorAttachment::Texture(&mut depth_texture),
                ugli::DepthAttachment::Renderbuffer(&mut depth_buffer),
            );
            let framebuffer = &mut framebuffer;
            ugli::clear(framebuffer, Some(Rgba::TRANSPARENT_BLACK), Some(1.0), None);
            let model_matrix =
                Mat4::translate(self.player.pos.extend(0.0)) * Mat4::rotate_z(self.player.rot);
            for mesh in &self.assets.boat.meshes {
                ugli::draw(
                    framebuffer,
                    &self.assets.shaders.obj2,
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
            self.draw_quad2(
                framebuffer,
                Mat4::translate(self.player.pos.extend(0.0))
                    * Mat4::rotate_x(-self.camera.rot_v)
                    * Mat4::scale(vec3(1.0, 0.0, 2.0) * 0.25)
                    * Mat4::translate(vec3(0.0, 0.0, 1.0)),
                &self.assets.player,
            );
            if let Some(pos) = self.player.fishing_pos {
                self.draw_quad2(
                    framebuffer,
                    Mat4::translate(pos.extend(0.0))
                        * Mat4::scale_uniform(0.1)
                        * Mat4::rotate_x(-self.camera.rot_v),
                    &self.assets.bobber,
                );
            }

            ugli::draw(
                framebuffer,
                &self.assets.shaders.obj2,
                ugli::DrawMode::TriangleFan,
                &self.quad,
                (
                    ugli::uniforms! {
                        u_model_matrix: Mat4::translate(vec3(0.0, 0.0, -1.0)) * Mat4::rotate_x(f32::PI / 2.0) * Mat4::scale_uniform(10.0),
                        u_texture: &self.white_texture,
                    },
                    geng::camera3d_uniforms(&self.camera, self.framebuffer_size),
                ),
                ugli::DrawParameters {
                    depth_func: Some(ugli::DepthFunc::Less),
                    ..default()
                },
            );
        }

        ugli::draw(
            framebuffer,
            &self.assets.shaders.water,
            ugli::DrawMode::TriangleFan,
            &self.quad,
            (
                ugli::uniforms! {
                    surfaceNoise: &self.assets.surface_noise,
                    distortNoise: &self.assets.distort_noise,
                    u_depth_texture: &depth_texture,
                    u_framebuffer_size: self.framebuffer_size,
                    u_model_matrix: Mat4::rotate_x(f32::PI / 2.0) * Mat4::scale_uniform(10.0),
                    u_time: self.time,
                },
                geng::camera3d_uniforms(&self.camera, self.framebuffer_size),
            ),
            ugli::DrawParameters {
                blend_mode: Some(ugli::BlendMode::default()),
                depth_func: Some(ugli::DepthFunc::Less),
                ..default()
            },
        );
    }

    fn update(&mut self, delta_time: f64) {
        let delta_time = delta_time as f32;

        self.time += delta_time;

        // Player move
        let delta_pos = self.player.target_pos - self.player.pos;
        if delta_pos.len() > 0.5 {
            self.player.pos += delta_pos.clamp_len(..=delta_time);
            self.player.rot +=
                normalize_angle(delta_pos.arg() - self.player.rot).clamp_abs(delta_time);
        }
    }

    fn handle_event(&mut self, event: geng::Event) {
        match event {
            geng::Event::MouseDown { position, button } => {
                let pos = self.world_pos(position.map(|x| x as f32));
                match button {
                    geng::MouseButton::Left => {
                        if self.player.fishing_pos.is_some() {
                            self.player.fishing_pos = None;
                        } else {
                            self.player.fishing_pos = Some(pos);
                        }
                    }
                    geng::MouseButton::Right => {
                        self.player.target_pos = pos;
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
}

fn main() {
    logger::init().unwrap();
    geng::setup_panic_handler();
    let geng = Geng::new("Sea of Friends");
    geng::run(
        &geng,
        geng::LoadingScreen::new(
            &geng,
            geng::EmptyLoadingScreen,
            geng::LoadAsset::load(&geng, &static_path().join("assets")),
            {
                let geng = geng.clone();
                move |assets| {
                    let assets = assets.unwrap();
                    let assets = Rc::new(assets);
                    Game::new(&geng, &assets)
                }
            },
        ),
    )
}
