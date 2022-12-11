use super::*;

const DEEPEST_DEPTH: f32 = -1.5;
const HIGHEST_LAND: f32 = 0.5;
pub const SIZE: f32 = 100.0;
pub const SHORE_HEIGHT: f32 = -0.2;

pub struct MapGeometry {
    pub land: ugli::VertexBuffer<ObjVertex>,
    pub edge: ugli::VertexBuffer<ObjVertex>,
    pub water: ugli::VertexBuffer<ObjVertex>,
    pub edge_segments: Vec<[Vec2<f32>; 2]>,
    pub deep_segments: Vec<[Vec2<f32>; 2]>,
    pub shore_segments: Vec<[Vec2<f32>; 2]>,
}

fn create_triangles() -> Vec<[Vec2<f32>; 3]> {
    let mut triangles = Vec::new();
    const N: i32 = 128;
    for x in (-N..N).skip(1).step_by(2) {
        for y in (-N..N).skip(1).step_by(2) {
            let vertex = |x, y| vec2(x as f32 / N as f32 * SIZE, y as f32 / N as f32 * SIZE);
            let mut quad = |dx, dy| {
                triangles.push([vertex(x, y), vertex(x + dx, y), vertex(x + dx, y + dy)]);
                triangles.push([vertex(x, y), vertex(x + dx, y + dy), vertex(x, y + dy)]);
            };
            quad(-1, -1);
            quad(1, -1);
            quad(1, 1);
            quad(-1, 1);
        }
    }
    triangles
}

pub fn create_map_geometry(geng: &Geng, assets: &Assets) -> MapGeometry {
    let mut edge = Vec::new();
    let mut water = Vec::new();
    let mut land = Vec::new();
    let mut edge_segments = Vec::new();
    for tri in create_triangles() {
        let a = tri[0];
        let b = tri[1];
        let c = tri[2];
        let mut zeros = Vec::new();
        let mut water_vs = Vec::new();
        let mut check = |a: Vec2<f32>, b: Vec2<f32>| {
            let av = Map::get().get_channel_value(3, a);
            let bv = Map::get().get_channel_value(3, b);
            let mut a = (a, av);
            let mut b = (b, bv);
            if a.1 >= 0.5 {
                water_vs.push(a.0);
            }
            if a.1 > b.1 {
                mem::swap(&mut a, &mut b);
            }
            if a.1 < 0.5 && b.1 >= 0.5 {
                let t = (0.5 - a.1) / (b.1 - a.1);
                let z = a.0 + t * (b.0 - a.0);
                zeros.push(z);
                water_vs.push(z);
            }
        };
        check(a, b);
        check(b, c);
        check(c, a);
        if zeros.len() == 2 {
            let z1 = zeros[0];
            let z2 = zeros[1];
            edge_segments.push([z1, z2]);
            let quad = [
                z1.extend(0.0),
                z1.extend(1.0),
                z2.extend(1.0),
                z2.extend(0.0),
            ];
            let vertex = |mut v: Vec3<f32>| -> ObjVertex {
                v.z *= -15.0;
                ObjVertex {
                    a_v: v,
                    a_uv: Vec2::ZERO,
                    a_vn: Vec3::ZERO,
                }
            };
            edge.push(vertex(quad[0]));
            edge.push(vertex(quad[1]));
            edge.push(vertex(quad[2]));
            edge.push(vertex(quad[0]));
            edge.push(vertex(quad[2]));
            edge.push(vertex(quad[3]));
        }
        if !water_vs.is_empty() {
            for vs in water_vs[1..].windows(2) {
                for v in [water_vs[0], vs[0], vs[1]] {
                    water.push(ObjVertex {
                        a_v: v.extend(0.0),
                        a_uv: v.map(|x| (x + SIZE) / (2.0 * SIZE)),
                        a_vn: Vec3::ZERO,
                    });
                    land.push(ObjVertex {
                        a_v: v.extend(Map::get().get_height(v)),
                        a_uv: v.map(|x| (x + SIZE) / (2.0 * SIZE)),
                        a_vn: Vec3::ZERO,
                    });
                }
            }
        }
    }

    let mut shore_segments = Vec::new();
    for tri in create_triangles() {
        let a = tri[0];
        let b = tri[1];
        let c = tri[2];
        let mut zeros = Vec::new();
        let mut check = |a: Vec2<f32>, b: Vec2<f32>| {
            let av = Map::get().get_height(a);
            let bv = Map::get().get_height(b);
            let mut a = (a, av);
            let mut b = (b, bv);
            if a.1 > b.1 {
                mem::swap(&mut a, &mut b);
            }
            let mid = SHORE_HEIGHT;
            if a.1 < mid && b.1 >= mid {
                let t = (mid - a.1) / (b.1 - a.1);
                let z = a.0 + t * (b.0 - a.0);
                zeros.push(z);
            }
        };
        check(a, b);
        check(b, c);
        check(c, a);
        if zeros.len() == 2 {
            let z1 = zeros[0];
            let z2 = zeros[1];
            shore_segments.push([z1, z2]);
        }
    }

    let mut deep_segments = Vec::new();
    for tri in create_triangles() {
        let a = tri[0];
        let b = tri[1];
        let c = tri[2];
        let mut zeros = Vec::new();
        let mut check = |a: Vec2<f32>, b: Vec2<f32>| {
            let av = Map::get().get_height(a);
            let bv = Map::get().get_height(b);
            let mut a = (a, av);
            let mut b = (b, bv);
            if a.1 > b.1 {
                mem::swap(&mut a, &mut b);
            }
            let mid = -1.25;
            if a.1 < mid && b.1 >= mid {
                let t = (mid - a.1) / (b.1 - a.1);
                let z = a.0 + t * (b.0 - a.0);
                zeros.push(z);
            }
        };
        check(a, b);
        check(b, c);
        check(c, a);
        if zeros.len() == 2 {
            let z1 = zeros[0];
            let z2 = zeros[1];
            deep_segments.push([z1, z2]);
        }
    }

    MapGeometry {
        land: ugli::VertexBuffer::new_static(geng.ugli(), land),
        edge: ugli::VertexBuffer::new_static(geng.ugli(), edge),
        water: ugli::VertexBuffer::new_static(geng.ugli(), water),
        edge_segments,
        shore_segments,
        deep_segments,
    }
}

fn to_segment(p1: Vec2<f32>, p2: Vec2<f32>, point: Vec2<f32>) -> Vec2<f32> {
    if Vec2::dot(point - p1, p2 - p1) < 0.0 {
        return p1 - point;
    }
    if Vec2::dot(point - p2, p1 - p2) < 0.0 {
        return p2 - point;
    }
    let n = (p2 - p1).rotate_90();
    // dot(point + n * t - p1, n) = 0
    // dot(point - p1, n) + dot(n, n) * t = 0
    let t = Vec2::dot(p1 - point, n) / Vec2::dot(n, n);
    n * t
}

pub fn vec_to(segments: &[[Vec2<f32>; 2]], point: Vec2<f32>) -> Vec2<f32> {
    segments
        .iter()
        .map(|&[p1, p2]| to_segment(p1, p2, point))
        .min_by_key(|v| r32(v.len_sqr()))
        .unwrap()
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
        let image = image::load(
            std::io::Cursor::new(include_bytes!("../static/assets/map.png")),
            image::ImageFormat::Png,
        )
        .unwrap()
        .into_rgba8();
        Self { image }
    }
    pub fn get_height(&self, pos: Vec2<f32>) -> f32 {
        self.get_channel_value(0, pos) * (HIGHEST_LAND - DEEPEST_DEPTH) + DEEPEST_DEPTH
    }
    pub fn get_channel_value(&self, channel: usize, pos: Vec2<f32>) -> f32 {
        let uv = pos.map(|x| ((x + SIZE) / (2.0 * SIZE)) * self.image.width() as f32);
        let values: [[f32; 2]; 2] = std::array::from_fn(|dx| {
            std::array::from_fn(|dy| {
                let color =
                    self.get_pixel(uv.map(|x| x.floor() as i32) + vec2(dx as i32, dy as i32));
                color.0[channel] as f32 / 0xff as f32
            })
        });
        let pos = uv.map(|x| x - x.floor());
        fn lerp(a: f32, b: f32, t: f32) -> f32 {
            a * (1.0 - t) + b * t
        }
        lerp(
            lerp(values[0][0], values[0][1], pos.y),
            lerp(values[1][0], values[1][1], pos.y),
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
