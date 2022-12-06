use geng::net::simple as simple_net;
use geng::prelude::*;

pub mod assets;
pub mod camera;
pub mod interpolation;
pub mod model;
pub mod obj;
pub mod util;

pub use assets::*;
pub use camera::*;
pub use interpolation::*;
pub use model::*;
pub use obj::*;
pub use util::*;

pub enum PlayerMovementControl {
    GoTo(Vec2<f32>),
    GoDirection(Vec2<f32>),
}

pub struct Player {
    pub pos: Position,
    pub control: PlayerMovementControl,
    pub fishing_pos: Option<Vec2<f32>>,
}

pub struct Game {
    player_id: Id,
    model: simple_net::Remote<Model>,
    interpolated: HashMap<Id, InterpolatedPosition>,
    framebuffer_size: Vec2<f32>,
    camera: Camera,
    geng: Geng,
    time: f32,
    assets: Rc<Assets>,
    white_texture: ugli::Texture,
    player: Player,
    quad: ugli::VertexBuffer<ObjVertex>,
    ping_time: f32,
    send_ping: bool,
}

impl Game {
    pub fn new(
        geng: &Geng,
        assets: &Rc<Assets>,
        player_id: Id,
        model: simple_net::Remote<Model>,
    ) -> Self {
        Self {
            player_id,
            model,
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
                pos: Position {
                    pos: Vec2::ZERO,
                    vel: Vec2::ZERO,
                    rot: 0.0,
                    w: 0.0,
                },
                control: PlayerMovementControl::GoDirection(Vec2::ZERO),
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
            interpolated: HashMap::new(),
            ping_time: 0.0,
            send_ping: false,
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

    pub fn draw_player(
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
        self.draw_player(framebuffer, &self.player.pos, self.player.fishing_pos);
        for (id, pos) in &self.interpolated {
            if *id == self.player_id {
                continue;
            }
            let pos = pos.get();
            self.draw_player(framebuffer, &pos, None);
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
            let model_matrix = Mat4::translate(self.player.pos.pos.extend(0.0))
                * Mat4::rotate_z(self.player.pos.rot);
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
                Mat4::translate(self.player.pos.pos.extend(0.0))
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

        let events = self.model.update();
        for i in self.interpolated.values_mut() {
            i.update(delta_time);
        }
        for event in events {
            match event {
                Event::Pong => {
                    self.model.send(Message::Update(self.player.pos.clone()));
                    {
                        let model = self.model.get();
                        self.interpolated
                            .retain(|id, _| model.positions.contains_key(id));
                        for (id, pos) in &model.positions {
                            if let Some(i) = self.interpolated.get_mut(id) {
                                i.server_update(pos);
                            } else {
                                self.interpolated
                                    .insert(*id, InterpolatedPosition::new(pos));
                            }
                        }
                    }
                    self.send_ping = true;
                }
            }
        }
        self.ping_time += delta_time;
        if self.ping_time > 1.0 / <Model as simple_net::Model>::TICKS_PER_SECOND && self.send_ping {
            self.ping_time = 0.0;
            self.send_ping = false;
            self.model.send(Message::Ping);
        }

        self.time += delta_time;

        // Player move
        let mut wasd = Vec2::<f32>::ZERO;
        if self.geng.window().is_key_pressed(geng::Key::W)
            || self.geng.window().is_key_pressed(geng::Key::Up)
        {
            wasd.y += 1.0;
        }
        if self.geng.window().is_key_pressed(geng::Key::A)
            || self.geng.window().is_key_pressed(geng::Key::Left)
        {
            wasd.x -= 1.0;
        }
        if self.geng.window().is_key_pressed(geng::Key::S)
            || self.geng.window().is_key_pressed(geng::Key::Down)
        {
            wasd.y -= 1.0;
        }
        if self.geng.window().is_key_pressed(geng::Key::D)
            || self.geng.window().is_key_pressed(geng::Key::Right)
        {
            wasd.x += 1.0;
        }
        if wasd != Vec2::ZERO
            || matches!(self.player.control, PlayerMovementControl::GoDirection(_))
        {
            self.player.control = PlayerMovementControl::GoDirection(wasd);
        }
        const MAX_SPEED: f32 = 2.0;
        let delta_pos = match self.player.control {
            PlayerMovementControl::GoTo(pos) => pos,
            PlayerMovementControl::GoDirection(dir) => self.player.pos.pos + dir * MAX_SPEED,
        } - self.player.pos.pos;
        const MAX_ROTATION_SPEED: f32 = 2.0;
        let target_w =
            normalize_angle(delta_pos.arg() - self.player.pos.rot).clamp_abs(MAX_ROTATION_SPEED);
        const ANGULAR_ACCELERATION: f32 = 1.0;
        self.player.pos.w += (target_w - self.player.pos.w).clamp_abs(ANGULAR_ACCELERATION);
        self.player.pos.rot += self.player.pos.w * delta_time;
        let target_vel = delta_pos.clamp_len(..=MAX_SPEED)
            * Vec2::dot(
                delta_pos.normalize_or_zero(),
                vec2(1.0, 0.0).rotate(self.player.pos.rot),
            )
            .max(0.0);
        const ACCELERATION: f32 = 1.0;
        self.player.pos.vel +=
            (target_vel - self.player.pos.vel).clamp_len(..=ACCELERATION * delta_time);
        self.player.pos.pos += self.player.pos.vel * delta_time;
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
                        self.player.control = PlayerMovementControl::GoTo(pos);
                    }
                    _ => {}
                }
            }
            geng::Event::MouseMove { position, .. } => {
                let pos = self.world_pos(position.map(|x| x as f32));
                if self
                    .geng
                    .window()
                    .is_button_pressed(geng::MouseButton::Right)
                {
                    self.player.control = PlayerMovementControl::GoTo(pos);
                }
            }
            _ => {}
        }
    }
}

fn main() {
    logger::init().unwrap();
    geng::setup_panic_handler();
    // let geng = Geng::new("Sea of Friends");
    simple_net::run(
        "Sea of Friends",
        Model::new,
        move |geng, player_id, model| {
            geng::LoadingScreen::new(
                &geng,
                geng::EmptyLoadingScreen,
                geng::LoadAsset::load(&geng, &static_path().join("assets")),
                {
                    let geng = geng.clone();
                    move |assets| {
                        let assets = assets.unwrap();
                        let assets = Rc::new(assets);
                        model.send(Message::Ping);
                        Game::new(&geng, &assets, player_id, model)
                    }
                },
            )
        },
    );
}
