use super::*;

const RING_COLOR: Rgba<f32> = Rgba {
    r: 1.0,
    g: 1.0,
    b: 1.0,
    a: 0.6,
};
const DROPLET_COLOR: Rgba<f32> = Rgba {
    r: 0.0,
    g: 0.4,
    b: 0.8,
    a: 0.6,
};

pub struct Splash {
    pub position: Vec2<f32>,
    pub lifetime: f32,
}

impl Splash {
    pub fn new(position: Vec2<f32>) -> Self {
        Self {
            position,
            lifetime: 0.0,
        }
    }
}

impl Game {
    pub fn draw_splashes(&self, framebuffer: &mut ugli::Framebuffer) {
        for splash in &self.splashes {
            self.draw_splash(framebuffer, splash);
        }
    }

    pub fn draw_splash(&self, framebuffer: &mut ugli::Framebuffer, splash: &Splash) {
        // Expanding wave
        let matrix = Mat4::translate(splash.position.extend(0.0)) * Mat4::rotate_x(f32::PI / 2.0);
        ugli::draw(
            framebuffer,
            &self.assets.shaders.wave,
            ugli::DrawMode::TriangleFan,
            &self.quad,
            (
                ugli::uniforms! {
                    u_model_matrix: matrix,
                    u_color: RING_COLOR,
                    u_lifetime: splash.lifetime,
                },
                geng::camera3d_uniforms(&self.camera, self.framebuffer_size),
            ),
            ugli::DrawParameters {
                blend_mode: Some(ugli::BlendMode::default()),
                // depth_func: Some(ugli::DepthFunc::Less),
                ..default()
            },
        );

        // Droplets
        if let Some(pos) = self.camera.world_to_screen(
            framebuffer.size().map(|x| x as f32),
            splash.position.extend(0.0),
        ) {
            let camera = geng::Camera2d {
                center: Vec2::ZERO,
                rotation: 0.0,
                fov: 20.0,
            };
            let pos = camera.screen_to_world(framebuffer.size().map(|x| x as f32), pos);
            for i in 0..5 {
                let angle = (i as f32 - 2.5) * f32::PI / 6.0;
                let (vx, vy) = angle.sin_cos();
                let x = vx * splash.lifetime;
                let y = vy * 2.0 * (0.0 - splash.lifetime) * (splash.lifetime - 1.0);
                let pos = pos + vec2(x, y);
                let radius = angle.sin() * (1.0 - splash.lifetime * splash.lifetime) * 0.1;
                let circle = draw_2d::Ellipse::circle(pos, radius, DROPLET_COLOR);
                self.geng.draw_2d(framebuffer, &camera, &circle);
            }
        }
    }
}
