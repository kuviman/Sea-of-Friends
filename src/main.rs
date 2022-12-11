#![allow(dead_code, unused_variables)]
use geng::net::simple as simple_net;
use geng::prelude::*;

pub mod assets;
pub mod camera;
pub mod fish;
pub mod interpolation;
pub mod inventory;
pub mod land;
pub mod local_player;
pub mod model;
pub mod movement;
pub mod obj;
pub mod player;
pub mod shops;
pub mod sound;
pub mod splash;
pub mod util;

pub use assets::*;
pub use camera::*;
pub use fish::*;
pub use interpolation::*;
pub use inventory::*;
pub use land::*;
pub use local_player::*;
pub use model::*;
pub use movement::*;
pub use obj::*;
pub use player::*;
pub use shops::*;
pub use sound::*;
pub use splash::*;
pub use util::*;

pub const SHOPPING_DISTANCE: f32 = 2.0;

// TODO: write the unit tests
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
    map_geometry: MapGeometry,
    caught_fish: Collection<CaughtFish>,
    inventory: Vec<FishType>,
    hovered_inventory_slot: Option<usize>,
    money: u32,
    fishdex: HashSet<FishType>,
    splashes: Vec<Splash>,
    players_trail_times: HashMap<Id, f32>,
    boat_sound_effects: HashMap<Id, geng::SoundEffect>,
    tutorial: String,
    tutorial_timer: f32,
    land_environment: Vec<ugli::VertexBuffer<ObjInstance>>,
    trees_environment: Vec<ugli::VertexBuffer<ObjInstance>>,
    shallow_environment: Vec<ugli::VertexBuffer<ObjInstance>>,
    editing_name: bool,
    show_names: bool,
    target_cam_distance: f32,
    show_reel_tutorial: bool,
}

#[derive(Debug, Clone, HasId)]
struct CaughtFish {
    id: Id,
    index: FishType,
    player: Id,
    lifetime: f32,
    caught_at: Vec2<f32>,
}

impl Game {
    pub fn new(
        geng: &Geng,
        assets: &Rc<Assets>,
        player_id: Id,
        model: simple_net::Remote<Model>,
    ) -> Self {
        assets.music.play().set_volume(0.1);
        let mut land_environment: Vec<ugli::VertexBuffer<ObjInstance>> =
            (0..assets.environment.land.len())
                .map(|_| ugli::VertexBuffer::new_static(geng.ugli(), vec![]))
                .collect();
        let mut trees_environment: Vec<ugli::VertexBuffer<ObjInstance>> =
            (0..assets.environment.trees.len())
                .map(|_| ugli::VertexBuffer::new_static(geng.ugli(), vec![]))
                .collect();
        let mut shallow_environment: Vec<ugli::VertexBuffer<ObjInstance>> =
            (0..assets.environment.shallow.len())
                .map(|_| ugli::VertexBuffer::new_static(geng.ugli(), vec![]))
                .collect();
        {
            // Generate the environment
            let mut rng = StdRng::seed_from_u64(1234);
            let (w, h) = Map::get().get_dimensions();
            for x in 0..w {
                for y in 0..h {
                    let pixel_value = Map::get()
                        .get_pixel(Vec2 {
                            x: x as i32,
                            y: y as i32,
                        })
                        .0[0];
                    if pixel_value > 220 {
                        let mut pos = Vec2 {
                            x: 100.0 * (x as f32 / w as f32 * 2.0 - 1.0),
                            y: 100.0 * (y as f32 / h as f32 * 2.0 - 1.0),
                        };
                        if Map::get().get_is_void(pos) || Map::get().is_ice(pos) {
                            continue;
                        }
                        let weight = pixel_value as u32 - 220;
                        let rng_choice = rng.gen_range(0..8500);
                        if rng_choice < 8500 - weight {
                            continue;
                        }
                        pos.y += rng_choice as f32 * 0.00005;
                        let height = Map::get().get_height(pos);
                        let mut big_tree = rng.gen_range(0..40);
                        if pos.len() < 20.0 {
                            big_tree = 10;
                        }
                        let mut idx = rng.gen_range(1..assets.environment.trees.len());
                        let flip = rng.gen_range(0..=1);
                        let flip_scale = if flip == 0 { 1.0 } else { -1.0 };
                        if big_tree == 0 {
                            idx = 0;
                        }
                        let texture = &assets.environment.trees[idx];
                        trees_environment[idx].push(ObjInstance {
                            i_model_matrix: Mat4::translate(pos.extend(height))
                                * Mat4::scale({
                                    let s = texture.size().map(|x| x as f32) * 0.0012;
                                    vec3(
                                        (s.x + 0.04 * weight as f32) * flip_scale,
                                        1.0,
                                        s.y + 0.04 * weight as f32,
                                    )
                                })
                                * Mat4::translate(vec3(0.0, 0.0, 1.0)),
                        });
                    }
                }
            }
            for _ in 0..10000 {
                let pos = vec2(rng.gen_range(-SIZE..SIZE), rng.gen_range(-SIZE..SIZE));
                let height = Map::get().get_height(pos);
                if Map::get().get_is_void(pos) || Map::get().is_ice(pos) {
                    continue;
                }
                if height > 0.0 {
                    let idx = rng.gen_range(0..assets.environment.land.len());
                    let texture = &assets.environment.land[idx];
                    land_environment[idx].push(ObjInstance {
                        i_model_matrix: Mat4::translate(pos.extend(height))
                            * Mat4::scale({
                                let s = texture.size().map(|x| x as f32) * 0.002;
                                vec3(s.x, 1.0, s.y)
                            })
                            * Mat4::translate(vec3(0.0, 0.0, 1.0)),
                    });
                } else if height > -1.0 && height < -0.5 {
                    let idx = rng.gen_range(0..assets.environment.shallow.len());
                    let texture = &assets.environment.shallow[idx];
                    shallow_environment[idx].push(ObjInstance {
                        i_model_matrix: Mat4::translate(pos.extend(height))
                            * Mat4::scale({
                                let s = texture.size().map(|x| x as f32) * 0.001;
                                vec3(s.x, 1.0, s.y)
                            })
                            * Mat4::translate(vec3(0.0, 0.0, 1.0)),
                    });
                }
            }
        }
        Self {
            show_names: true,
            show_reel_tutorial: true,
            target_cam_distance: 20.0,
            editing_name: true,
            trees_environment,
            land_environment,
            shallow_environment,
            player_id,
            model,
            time: 0.0,
            framebuffer_size: vec2(1.0, 1.0),
            geng: geng.clone(),
            assets: assets.clone(),
            camera: Camera {
                fov: 40.0 * f32::PI / 180.0,
                pos: Vec3::ZERO,
                distance: 20.0,
                rot_h: 0.0,
                rot_v: 60.0 * f32::PI / 180.0,
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
            map_geometry: create_map_geometry(geng, assets),
            interpolated: HashMap::new(),
            ping_time: 0.0,
            send_ping: false,
            player_timings: HashMap::new(),
            caught_fish: Collection::new(),
            inventory: Vec::new(),
            hovered_inventory_slot: None,
            money: 0,
            fishdex: HashSet::new(),
            splashes: Vec::new(),
            players_trail_times: HashMap::new(),
            boat_sound_effects: HashMap::new(),
            tutorial: "left mouse to fish\nright mouse to move".to_owned(),
            tutorial_timer: 100000000.0,
        }
    }

    pub fn draw_quad(
        &self,
        framebuffer: &mut ugli::Framebuffer,
        matrix: Mat4<f32>,
        texture: &ugli::Texture,
        color: Rgba<f32>,
    ) {
        ugli::draw(
            framebuffer,
            &self.assets.shaders.obj,
            ugli::DrawMode::TriangleFan,
            ugli::instanced(
                &self.quad,
                &ugli::VertexBuffer::new_dynamic(
                    self.geng.ugli(),
                    vec![ObjInstance {
                        i_model_matrix: matrix,
                    }],
                ),
            ),
            (
                ugli::uniforms! {
                    u_color: color,
                    u_texture: texture,
                },
                geng::camera3d_uniforms(&self.camera, self.framebuffer_size),
            ),
            ugli::DrawParameters {
                // blend_mode: Some(ugli::BlendMode::default()),
                depth_func: Some(ugli::DepthFunc::LessOrEqual),
                ..default()
            },
        );
    }
    pub fn draw_texture(
        &self,
        framebuffer: &mut ugli::Framebuffer,
        pos: Vec3<f32>,
        height: f32,
        texture: &ugli::Texture,
        origin: Vec2<f32>,
    ) {
        self.draw_quad(
            framebuffer,
            Mat4::translate(pos)
                * Mat4::scale(
                    vec3(texture.size().x as f32 / texture.size().y as f32, 1.0, 1.0) * height,
                )
                // * Mat4::rotate_z((self.camera.eye_pos().xy() - pos.xy()).arg() + f32::PI / 2.0)
                // * Mat4::rotate_x(-self.camera.rot_v)
                * Mat4::translate(vec3(-origin.x, 0.0, -origin.y)),
            texture,
            Rgba::WHITE,
        );
    }

    pub fn world_pos(&self, mouse_pos: Vec2<f32>) -> Vec2<f32> {
        let camera_ray = self.camera.pixel_ray(self.framebuffer_size, mouse_pos);
        camera_ray.from.xy() - camera_ray.dir.xy() * camera_ray.from.z / camera_ray.dir.z
    }

    pub fn draw_environment(&self, framebuffer: &mut ugli::Framebuffer) {
        for (index, instances) in self.trees_environment.iter().enumerate() {
            ugli::draw(
                framebuffer,
                &self.assets.shaders.obj,
                ugli::DrawMode::TriangleFan,
                ugli::instanced(&self.quad, instances),
                (
                    ugli::uniforms! {
                        u_color: Rgba::WHITE,
                        u_texture: &self.assets.environment.trees[index],
                    },
                    geng::camera3d_uniforms(&self.camera, self.framebuffer_size),
                ),
                ugli::DrawParameters {
                    // blend_mode: Some(ugli::BlendMode::default()),
                    depth_func: Some(ugli::DepthFunc::LessOrEqual),
                    ..default()
                },
            );
        }
        for (index, instances) in self.land_environment.iter().enumerate() {
            ugli::draw(
                framebuffer,
                &self.assets.shaders.obj,
                ugli::DrawMode::TriangleFan,
                ugli::instanced(&self.quad, instances),
                (
                    ugli::uniforms! {
                        u_color: Rgba::WHITE,
                        u_texture: &self.assets.environment.land[index],
                    },
                    geng::camera3d_uniforms(&self.camera, self.framebuffer_size),
                ),
                ugli::DrawParameters {
                    // blend_mode: Some(ugli::BlendMode::default()),
                    depth_func: Some(ugli::DepthFunc::LessOrEqual),
                    ..default()
                },
            );
        }
        for (index, instances) in self.shallow_environment.iter().enumerate() {
            ugli::draw(
                framebuffer,
                &self.assets.shaders.obj,
                ugli::DrawMode::TriangleFan,
                ugli::instanced(&self.quad, instances),
                (
                    ugli::uniforms! {
                        u_color: Rgba::WHITE,
                        u_texture: &self.assets.environment.shallow[index],
                    },
                    geng::camera3d_uniforms(&self.camera, self.framebuffer_size),
                ),
                ugli::DrawParameters {
                    // blend_mode: Some(ugli::BlendMode::default()),
                    depth_func: Some(ugli::DepthFunc::LessOrEqual),
                    ..default()
                },
            );
        }
    }
}

impl geng::State for Game {
    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        self.framebuffer_size = framebuffer.size().map(|x| x as f32);
        self.geng.draw_2d(
            framebuffer,
            &geng::Camera2d {
                center: Vec2::ZERO,
                rotation: 0.0,
                fov: (self.assets.background.size().y as f32).min(
                    self.assets.background.size().x as f32 * self.framebuffer_size.y
                        / self.framebuffer_size.x,
                ) * 0.8,
            },
            &draw_2d::TexturedQuad::unit(&self.assets.background)
                .scale(self.assets.background.size().map(|x| x as f32 / 2.0))
                .translate(
                    self.assets.background.size().map(|x| x as f32 / 2.0)
                        * self
                            .camera
                            .pos
                            .xy()
                            .map(|x| (-x / SIZE).clamp(-1.0, 1.0) * 0.2),
                ),
        );
        ugli::clear(
            framebuffer,
            None, // Some(self.assets.config.space_color),
            Some(1.0),
            None,
        );

        ugli::draw(
            framebuffer,
            &self.assets.shaders.land,
            ugli::DrawMode::Triangles,
            &self.map_geometry.land,
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
        self.draw_shops(framebuffer);
        self.draw_environment(framebuffer);

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
                &self.map_geometry.land,
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
            ugli::draw(
                framebuffer,
                &self.assets.shaders.land2,
                ugli::DrawMode::Triangles,
                &self.map_geometry.edge,
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
            ugli::DrawMode::Triangles,
            &self.map_geometry.water,
            (
                ugli::uniforms! {
                    u_heightmap: &self.assets.map,
                    surfaceNoise: &self.assets.surface_noise,
                    distortNoise: &self.assets.distort_noise,
                    u_depth_texture: &depth_texture,
                    u_framebuffer_size: self.framebuffer_size,
                    u_model_matrix: Mat4::identity(),
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
        ugli::draw(
            framebuffer,
            &self.assets.shaders.edge,
            ugli::DrawMode::Triangles,
            &self.map_geometry.edge,
            (
                ugli::uniforms! {
                    u_time: self.time,
                    surfaceNoise: &self.assets.surface_noise,
                },
                geng::camera3d_uniforms(&self.camera, self.framebuffer_size),
            ),
            ugli::DrawParameters {
                depth_func: Some(ugli::DepthFunc::Less),
                blend_mode: Some(ugli::BlendMode::default()),
                ..default()
            },
        );

        self.draw_splashes(framebuffer);
        self.draw_inventory(framebuffer);
    }

    fn update(&mut self, delta_time: f64) {
        let delta_time = delta_time as f32;

        self.tutorial_timer -= delta_time;

        if self.editing_name {
            self.camera.distance = 5.0;
        } else {
            self.camera.distance +=
                (self.target_cam_distance - self.camera.distance).clamp_abs(delta_time * 30.0);
        }

        self.player.inventory = self.inventory.clone(); // NOICE

        self.geng
            .audio()
            .set_listener_position(self.player.pos.pos.extend(0.0).map(|x| x as f64));
        self.geng
            .audio()
            .set_listener_orientation(vec3(0.0, 1.0, 0.0), vec3(0.0, 0.0, 1.0));

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
                Event::CaughtFish {
                    player,
                    fish,
                    fish_type,
                    position,
                } => {
                    self.caught_fish.insert(CaughtFish {
                        id: fish,
                        index: fish_type,
                        player,
                        lifetime: 0.0,
                        caught_at: position,
                    });
                }
                Event::Sound {
                    player,
                    sound_type,
                    pos,
                } => {
                    if player != self.player_id {
                        self.play_sound(pos, sound_type);
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

        let model = self.model.get();
        for player in &model.players {
            if Map::get().get_height(player.pos.pos) < 0.0
                && player.seated.is_none()
                && player.pos.vel.len() > 1.0
            {
                // Captain moving
                let time = self.players_trail_times.entry(player.id).or_insert(0.0);
                *time -= delta_time;
                if *time <= 0.0 {
                    *time += 0.2;
                    self.splashes.push(Splash::new(player.pos.pos, 0, 0.5));
                }
            }
        }

        for fish in &mut self.caught_fish {
            fish.lifetime += delta_time;
            if fish.lifetime >= 1.0 && fish.player == self.player_id {
                self.fishdex.insert(fish.index);
                self.inventory.push(fish.index);
            }
        }
        self.caught_fish.retain(|fish| fish.lifetime < 1.0);

        if self.inventory.len() > self.assets.config.inventory_size {
            self.tutorial =
                "your inventory is limited!\nyou should maybe go sell some fish?".to_owned();
            self.tutorial_timer = 5.0;
            self.model.send(Message::SpawnFish {
                index: self.inventory.remove(0),
                pos: self.player.pos.pos,
            });
            self.play_sound_for_everyone(
                self.player.pos.pos,
                if Map::get().get_height(self.player.pos.pos) > 0.0 {
                    SoundType::DropFishLand
                } else {
                    SoundType::DropFishWater
                },
            );
        }

        for splash in &mut self.splashes {
            splash.lifetime += delta_time * splash.speed;
        }
        self.splashes.retain(|splash| splash.lifetime < 1.0);
    }

    fn handle_event(&mut self, event: geng::Event) {
        match event {
            geng::Event::MouseDown { position, button } => {
                let pos = self.world_pos(position.map(|x| x as f32));
                match button {
                    geng::MouseButton::Left => {
                        let mut can_fish = true;
                        if let Some(index) = self.hovered_inventory_slot {
                            can_fish = false;
                            if index < self.inventory.len() {
                                let fish = self.inventory.remove(index);
                                if self.can_sell_fish() {
                                    self.money += self.assets.fishes[fish].config.cost;
                                    self.play_sound_for_everyone(
                                        self.player.pos.pos,
                                        SoundType::SellFish,
                                    );
                                    self.model.send(Message::RespawnFish { index: fish });
                                } else {
                                    self.play_sound_for_everyone(
                                        self.player.pos.pos,
                                        if Map::get().get_height(self.player.pos.pos) > 0.0 {
                                            SoundType::DropFishLand
                                        } else {
                                            SoundType::DropFishWater
                                        },
                                    );
                                    self.model.send(Message::SpawnFish {
                                        index: fish,
                                        pos: self.player.pos.pos,
                                    });
                                }
                            }
                        }
                        if let Some((index, boat_type)) = self.is_hovering_boat_shop() {
                            let boat_level = index as u8 + 1;
                            can_fish = false;
                            if self.money >= boat_type.cost {
                                self.money -= boat_type.cost;
                                self.player.boat_level = boat_level;
                                self.play_sound_for_everyone(
                                    self.player.pos.pos,
                                    SoundType::UpgradeBoat,
                                );
                                if boat_level == 1 {
                                    self.tutorial =
                                        "right click water when near it\nto get into the boat"
                                            .to_owned();
                                    self.tutorial_timer = 10.0;
                                }
                                if boat_level == 2 {
                                    self.tutorial = "you can now explore the deep sea".to_owned();
                                    self.tutorial_timer = 10.0;
                                }
                                if boat_level == 3 {
                                    self.tutorial =
                                        "you can now explore beyond the edge of the world"
                                            .to_owned();
                                    self.tutorial_timer = 10.0;
                                }
                            }
                        }
                        if can_fish {
                            match self.player.fishing_state {
                                FishingState::Idle => {
                                    self.player.fishing_state = FishingState::Spinning;
                                }
                                FishingState::Reeling { fish, .. } => {
                                    self.model.send(Message::Catch(fish));
                                    self.player.fishing_state = FishingState::Idle;
                                    self.play_sound(self.player.pos.pos, SoundType::Ding);
                                }
                                _ => {
                                    self.player.fishing_state = FishingState::Idle;
                                    self.play_sound_for_everyone(
                                        self.player.pos.pos,
                                        SoundType::StopFishing,
                                    );
                                }
                            }
                        }
                    }
                    geng::MouseButton::Right => {
                        let mut teleport = None;
                        let land = |pos| Map::get().get_height(pos) > SHORE_HEIGHT;
                        let mut seated = false;
                        if let Some((other_player, seat)) = self.can_join(&mut seated) {
                            // let other_player = other_player.clone();
                            self.player.seated = Some(Seated {
                                player: other_player.id,
                                seat,
                            });
                            self.play_sound_for_everyone(
                                other_player.pos.pos,
                                SoundType::EnterBoat,
                            );
                        }
                        let raycast = |to, from| {
                            let mut hit: Option<Vec2<f32>> = None;
                            let raycast_resolution = 100.0;
                            let segment = (to - from) / raycast_resolution;
                            // We are trying to go onto land
                            if land(to) {
                                for i in 0..(raycast_resolution as u32) {
                                    let check_pos = from + segment * i as f32;
                                    if land(check_pos) && hit.is_none() {
                                        hit = Some(check_pos + segment * 2.0); // Add a little bit of buffer to the result
                                    }
                                    // we passed clear through an island - invalidate the hit
                                    if !land(check_pos) && hit.is_some() {
                                        hit = None;
                                    }
                                }
                            } else {
                                // We are trying to go into the water
                                for i in 0..(raycast_resolution as u32) {
                                    let check_pos = from + segment * i as f32;
                                    if !land(check_pos) {
                                        hit = Some(check_pos + segment * 2.0); // Add a little bit of buffer to the result
                                        break;
                                    }
                                }
                            }
                            hit
                        };
                        if self.player.seated.is_some() && land(pos) {
                            // teleport between land <> water (friend's boat)
                            if let Some(hit_pos) = raycast(pos, self.player.pos.pos) {
                                // Verify the hit pos is within our reach
                                if hit_pos.sub(self.player.pos.pos).len() < 4.0 {
                                    teleport = Some(hit_pos);
                                    self.player.seated = None;
                                }
                            }
                        }
                        if !seated
                            && self.player.boat_level > 0
                            && land(pos) != land(self.player.pos.pos)
                        {
                            // teleport between land <> water (our own boat)
                            let mut player_radius = 1.0;
                            player_radius *= self.assets.config.boat_types
                                [(self.player.boat_level - 1) as usize]
                                .scale;

                            if let Some(hit_pos) = raycast(pos, self.player.pos.pos) {
                                // Verify the hit pos is within our reach
                                if hit_pos.sub(self.player.pos.pos).len() < player_radius + 0.5 {
                                    teleport = Some(hit_pos);
                                }
                            }
                        }
                        if let Some(pos) = teleport {
                            self.play_sound_for_everyone(
                                pos,
                                if land(pos) {
                                    SoundType::ExitBoat
                                } else {
                                    SoundType::EnterBoat
                                },
                            );
                            self.player.pos.pos = pos;
                            self.player.pos.vel = Vec2::ZERO;
                        }
                        self.player_control = PlayerMovementControl::GoTo(pos);
                    }
                    geng::MouseButton::Middle => {
                        // self.player.pos.pos = pos;
                        println!("{}", pos)
                    }
                }
            }
            geng::Event::MouseUp { position, button } => {
                let pos = self.world_pos(position.map(|x| x as f32));
                match button {
                    geng::MouseButton::Left => {
                        if let FishingState::Spinning = self.player.fishing_state {
                            self.player.fishing_state = FishingState::Casting(
                                self.player.pos.pos
                                    + (pos - self.player.pos.pos).clamp_len(..=MAX_LINE_LEN - 0.1),
                            );
                            self.play_sound_for_everyone(self.player.pos.pos, SoundType::Casting);
                            self.play_sound_for_everyone(self.player.pos.pos, SoundType::Whip);
                        }
                    }
                    geng::MouseButton::Right => {
                        // TODO: ask kuviman why we did this?
                        // kuviman answers: because we had to
                        // badcop answers: ok
                        // Nertsal: why are we having a discussion in comments in source code
                        // kuviman: because why not
                        // self.player_control = PlayerMovementControl::GoDirection(Vec2::ZERO);
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
                self.target_cam_distance =
                    (self.target_cam_distance * 1.005f32.powf(-delta as f32)).clamp(10.0, 30.0);
            }
            geng::Event::KeyDown { key } => {
                if key == geng::Key::Enter {
                    self.editing_name = !self.editing_name;
                }
                if key == geng::Key::Tab {
                    self.show_names = !self.show_names;
                }
                if key == geng::Key::PageDown {
                    self.player.pos.pos = Vec2::ZERO;
                    self.player.seated = None;
                    self.player_control = PlayerMovementControl::GoDirection(Vec2::ZERO);
                }
                if self.editing_name {
                    if key == geng::Key::Backspace {
                        self.player.name.pop();
                    }
                    if self.player.name.len() < 15 {
                        let s = format!("{key:?}");
                        if s.len() == 1 {
                            self.player.name.push_str(&s);
                        }
                    }
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
        Model::init,
        move |geng, player_id, model| {
            geng::LoadingScreen::new(
                geng,
                geng::EmptyLoadingScreen,
                geng::LoadAsset::load(geng, &static_path().join("assets")),
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
