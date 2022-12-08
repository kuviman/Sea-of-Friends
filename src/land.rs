use image::GenericImageView;

use super::*;

const DEEPEST_DEPTH: f32 = -1.5;
const HIGHEST_LAND: f32 = 0.5;
const SIZE: f32 = 100.0;

pub fn create_land_geometry(geng: &Geng, assets: &Assets) -> ugli::VertexBuffer<ObjVertex> {
    let mut vs = Vec::new();
    const N: i32 = 128;
    for x in (-N..N).skip(1).step_by(2) {
        for y in (-N..N).skip(1).step_by(2) {
            let vertex = |x, y| {
                let uv = vec2(
                    x as f32 / N as f32 * 0.5 + 0.5,
                    y as f32 / N as f32 * 0.5 + 0.5,
                );
                let pos = vec2(x as f32 / N as f32 * SIZE, y as f32 / N as f32 * SIZE);
                ObjVertex {
                    a_v: vec3(pos.x, pos.y, Map::get().get_height(pos)),
                    a_uv: uv,
                    a_vn: Vec3::ZERO,
                }
            };
            let mut quad = |dx, dy| {
                vs.push(vertex(x, y));
                vs.push(vertex(x + dx, y));
                vs.push(vertex(x + dx, y + dy));
                vs.push(vertex(x, y));
                vs.push(vertex(x + dx, y + dy));
                vs.push(vertex(x, y + dy));
            };
            quad(-1, -1);
            quad(1, -1);
            quad(1, 1);
            quad(-1, 1);
        }
    }
    ugli::VertexBuffer::new_static(geng.ugli(), vs)
}

pub struct Map {
    image: image::RgbaImage,
}

static mut MAP: Option<Map> = None;

impl Map {
    pub fn get() -> &'static Map {
        unsafe { MAP.get_or_insert_with(Map::load) }
    }
    pub fn load() -> Self {
        let image = image::open(static_path().join("assets").join("map.png"))
            .unwrap()
            .into_rgba8();
        Self { image }
    }
    pub fn get_height(&self, pos: Vec2<f32>) -> f32 {
        let uv = pos.map(|x| ((x + SIZE) / (2.0 * SIZE)) * self.image.width() as f32);
        let heights: [[f32; 2]; 2] = std::array::from_fn(|dx| {
            std::array::from_fn(|dy| {
                let color =
                    self.get_pixel(uv.map(|x| x.floor() as i32) + vec2(dx as i32, dy as i32));
                color.0[0] as f32 / 0xff as f32 * (HIGHEST_LAND - DEEPEST_DEPTH) + DEEPEST_DEPTH
            })
        });
        let pos = uv.map(|x| x - x.floor());
        fn lerp(a: f32, b: f32, t: f32) -> f32 {
            a * (1.0 - t) + b * t
        }
        lerp(
            lerp(heights[0][0], heights[0][1], pos.y),
            lerp(heights[1][0], heights[1][1], pos.y),
            pos.x,
        )
    }
    fn get_pixel(&self, pos: Vec2<i32>) -> image::Rgba<u8> {
        *self.image.get_pixel(
            pos.x.clamp(0, self.image.width() as i32 - 1) as u32,
            (self.image.height() as i32 - pos.y - 1).clamp(0, self.image.height() as i32 - 1)
                as u32,
        )
    }
}
