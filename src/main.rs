use geng::net::simple as simple_net;
use geng::prelude::*;

pub mod assets;
pub mod camera;
pub mod fish;
pub mod interpolation;
pub mod land;
pub mod local_player;
pub mod model;
pub mod movement;
pub mod obj;
pub mod player;
pub mod util;

pub use assets::*;
pub use camera::*;
pub use fish::*;
pub use interpolation::*;
pub use land::*;
pub use local_player::*;
pub use model::*;
pub use movement::*;
pub use obj::*;
pub use player::*;
pub use util::*;

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
    player_control: PlayerMovementControl,
    player_timings: HashMap<Id, f32>,
    quad: ugli::VertexBuffer<ObjVertex>,
    ping_time: f32,
    send_ping: bool,
    land_geometry: ugli::VertexBuffer<ObjVertex>,
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
                fov: f32::PI / 3.0,
                pos: Vec3::ZERO,
                distance: 10.0,
                rot_h: 0.0,
                rot_v: f32::PI / 4.0,
            },
            white_texture: ugli::Texture::new_with(geng.ugli(), vec2(1, 1), |_| Rgba::WHITE),
            player: Player::new(player_id, Vec2::ZERO),
            player_control: PlayerMovementControl::GoDirection(Vec2::ZERO),
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
            land_geometry: create_land_geometry(geng, assets),
            interpolated: HashMap::new(),
            ping_time: 0.0,
            send_ping: false,
            player_timings: HashMap::new(),
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

        ugli::draw(
            framebuffer,
            &self.assets.shaders.land,
            ugli::DrawMode::Triangles,
            &self.land_geometry,
            (
                ugli::uniforms! {
                    u_heightmap: &self.assets.map,
                    u_texture: &self.assets.map_color,
                },
                geng::camera3d_uniforms(&self.camera, self.framebuffer_size),
            ),
            ugli::DrawParameters {
                depth_func: Some(ugli::DepthFunc::Less),
                ..default()
            },
        );
        self.draw_fishes(framebuffer);
        self.draw_players(framebuffer);

        // TODO
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

            ugli::draw(
                framebuffer,
                &self.assets.shaders.land2,
                ugli::DrawMode::Triangles,
                &self.land_geometry,
                (
                    ugli::uniforms! {
                        u_heightmap: &self.assets.map,
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
                    u_model_matrix: Mat4::rotate_x(f32::PI / 2.0) * Mat4::scale_uniform(100.0),
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

        self.camera.pos = self.player.pos.pos.extend(0.0);

        let events = self.model.update();
        for i in self.interpolated.values_mut() {
            i.update(delta_time);
        }
        for event in events {
            match event {
                Event::Pong => {
                    self.model.send(Message::Update(self.player.clone()));
                    {
                        let model = self.model.get();
                        self.interpolated.retain(|id, _| {
                            model.players.get(id).is_some() || model.fishes.get(id).is_some()
                        });
                        for (id, pos) in itertools::chain![
                            model.players.iter().map(|player| (&player.id, &player.pos)),
                            model.fishes.iter().map(|fish| (&fish.id, &fish.pos))
                        ] {
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
                Event::Reel { player, fish } => {
                    if player == self.player_id {
                        if let FishingState::Waiting(bobber_pos) = self.player.fishing_state {
                            self.player.fishing_state =
                                FishingState::PreReeling { fish, bobber_pos };
                        }
                    }
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

        self.update_my_player(delta_time);
        self.update_local_player_data(delta_time);
    }

    fn handle_event(&mut self, event: geng::Event) {
        match event {
            geng::Event::MouseDown { position, button } => {
                let pos = self.world_pos(position.map(|x| x as f32));
                match button {
                    geng::MouseButton::Left => match self.player.fishing_state {
                        FishingState::Idle => {
                            self.player.fishing_state = FishingState::Spinning;
                        }
                        FishingState::Reeling { fish, .. } => {
                            self.model.send(Message::Catch(fish));
                            self.player.fishing_state = FishingState::Idle;
                        }
                        _ => {
                            self.player.fishing_state = FishingState::Idle;
                        }
                    },
                    geng::MouseButton::Right => {
                        self.player_control = PlayerMovementControl::GoTo(pos);
                    }
                    geng::MouseButton::Middle => {
                        self.player.pos.pos = pos;
                    }
                    _ => {}
                }
            }
            geng::Event::MouseUp { position, button } => {
                let pos = self.world_pos(position.map(|x| x as f32));
                match button {
                    geng::MouseButton::Left => {
                        if let FishingState::Spinning = self.player.fishing_state {
                            self.player.fishing_state = FishingState::Casting(pos);
                        }
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
                    self.player_control = PlayerMovementControl::GoTo(pos);
                }
            }
            geng::Event::Wheel { delta } => {
                self.camera.distance =
                    (self.camera.distance * 1.01f32.powf(-delta as f32)).clamp(1.0, 300.0);
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
