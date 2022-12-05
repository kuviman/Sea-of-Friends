use geng::prelude::*;

pub mod camera;
pub mod obj;

pub use camera::*;
pub use obj::*;

#[derive(geng::Assets)]
pub struct Shaders {
    pub obj: ugli::Program,
}

#[derive(geng::Assets)]
pub struct Assets {
    pub shaders: Shaders,
    #[asset(path = "1391 Rowboat.obj")]
    pub boat: Obj,
}

pub struct Game {
    framebuffer_size: Vec2<f32>,
    camera: Camera,
    geng: Geng,
    assets: Rc<Assets>,
    white_texture: ugli::Texture,
}

impl Game {
    pub fn new(geng: &Geng, assets: &Rc<Assets>) -> Self {
        Self {
            framebuffer_size: vec2(1.0, 1.0),
            geng: geng.clone(),
            assets: assets.clone(),
            camera: Camera {
                fov: f32::PI / 2.0,
                pos: Vec3::ZERO,
                distance: 100.0,
                rot_h: 0.0,
                rot_v: f32::PI / 4.0,
            },
            white_texture: ugli::Texture::new_with(geng.ugli(), vec2(1, 1), |_| Rgba::WHITE),
        }
    }
}

impl geng::State for Game {
    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        self.framebuffer_size = framebuffer.size().map(|x| x as f32);
        ugli::clear(framebuffer, Some(Rgba::BLUE), Some(1.0), None);
        for mesh in &self.assets.boat.meshes {
            ugli::draw(
                framebuffer,
                &self.assets.shaders.obj,
                ugli::DrawMode::Triangles,
                &mesh.geometry,
                (
                    ugli::uniforms! {
                        u_model_matrix: Mat4::rotate_x(f32::PI / 2.0),
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
