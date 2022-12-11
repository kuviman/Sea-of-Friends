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
pub use util::*;

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
        Self {
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
            &self.quad,
            (
                ugli::uniforms! {
                    u_model_matrix: matrix,
                    u_color: color,
                    u_texture: texture,
                },
                geng::camera3d_uniforms(&self.camera, self.framebuffer_size),
            ),
            ugli::DrawParameters {
                blend_mode: Some(ugli::BlendMode::default()),
                depth_func: Some(ugli::DepthFunc::LessOrEqual),
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
}

impl geng::State for Game {
    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        self.framebuffer_size = framebuffer.size().map(|x| x as f32);
        ugli::clear(
            framebuffer,
            Some(self.assets.config.space_color),
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
                ..default()
            },
        );

        self.draw_inventory(framebuffer);
    }

    fn update(&mut self, delta_time: f64) {
        let delta_time = delta_time as f32;

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

        for fish in &mut self.caught_fish {
            fish.lifetime += delta_time;
            if fish.lifetime >= 1.0 {
                self.fishdex.insert(fish.index);
                self.inventory.push(fish.index);
            }
        }
        self.caught_fish.retain(|fish| fish.lifetime < 1.0);
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
                                let fishing_shop_distance = self
                                    .assets
                                    .config
                                    .fish_shops
                                    .iter()
                                    .map(|&pos| r32((pos - self.player.pos.pos).len()))
                                    .min()
                                    .unwrap()
                                    .raw();
                                if fishing_shop_distance < 2.0 {
                                    self.money += self.assets.fishes[fish].config.cost;
                                } else {
                                    self.model.send(Message::SpawnFish {
                                        index: fish,
                                        pos: self.player.pos.pos,
                                    });
                                }
                            }
                        }
                        {
                            for (index, boat_type) in
                                self.assets.config.boat_types.iter().enumerate()
                            {
                                let boat_level = index as u8 + 1;
                                if let Some(distance) = boat_type
                                    .shops
                                    .iter()
                                    .filter(|&&pos| {
                                        let pos = pos.extend(Map::get().get_height(pos));
                                        let ray = self.camera.pixel_ray(
                                            self.framebuffer_size,
                                            self.geng.window().mouse_pos().map(|x| x as f32),
                                        );
                                        Vec3::cross(ray.dir.normalize_or_zero(), pos - ray.from)
                                            .len()
                                            < 1.0
                                    })
                                    .map(|&pos| r32((pos - self.player.pos.pos).len()))
                                    .min()
                                {
                                    can_fish = false;
                                    if distance.raw() < 2.0
                                        && self.money >= boat_type.cost
                                        && self.player.boat_level < boat_level
                                    {
                                        self.money -= boat_type.cost;
                                        self.player.boat_level = boat_level;
                                    }
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
                                    if let Some(fish) = self.model.get().fishes.get(&fish) {
                                        // self.fishdex.insert(fish.index);
                                        // self.inventory.push(fish.index);
                                        if self.inventory.len() > self.assets.config.inventory_size
                                        {
                                            self.model.send(Message::SpawnFish {
                                                index: self.inventory.remove(0),
                                                pos: self.player.pos.pos,
                                            });
                                        }
                                    }
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
                        if self.player.seated.is_none() && land(self.player.pos.pos) {
                            for other_player in &self.model.get().players {
                                if other_player.id == self.player_id {
                                    continue;
                                }
                                if other_player.seated.is_some() {
                                    continue;
                                }
                                let Some(p) = self.interpolated.get(&other_player.id) else { continue };
                                if land(p.get().pos) {
                                    continue;
                                }

                                let mut other_player_radius = 1.0;
                                if other_player.boat_level > 0 {
                                    other_player_radius *= self.assets.config.boat_types
                                        [(other_player.boat_level - 1) as usize]
                                        .scale;
                                }
                                // Make sure we are in range of their boat
                                if (p.get().pos - self.player.pos.pos).len()
                                    > (2.5 + other_player_radius / 2.0)
                                {
                                    continue;
                                }

                                // check if we clicked within bounds of other_player
                                if (p.get().pos - pos).len() < other_player_radius {
                                    seated = true;
                                    let mut seats: HashSet<usize> = (1..self.assets.ships
                                        [other_player.boat_level.max(1) as usize - 1]
                                        .seats
                                        .len())
                                        .collect();
                                    for p in &self.model.get().players {
                                        if let Some(seated) = p.seated {
                                            if seated.player == other_player.id {
                                                seats.remove(&seated.seat);
                                            }
                                        }
                                    }
                                    if let Some(seat) = seats.into_iter().next() {
                                        self.player.seated = Some(Seated {
                                            player: other_player.id,
                                            seat,
                                        });
                                    }
                                }
                            }
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
                            self.player.pos.pos = pos;
                            self.player.pos.vel = Vec2::ZERO;
                        }
                        self.player_control = PlayerMovementControl::GoTo(pos);
                    }
                    geng::MouseButton::Middle => {
                        self.player.pos.pos = pos;
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
