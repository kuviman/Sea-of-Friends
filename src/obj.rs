use super::*;

#[derive(Clone)]
pub struct Material {
    pub name: String,
    pub texture: Option<Rc<ugli::Texture>>,
    pub ambient_color: Rgba<f32>,
    pub diffuse_color: Rgba<f32>,
}

#[derive(ugli::Vertex, Debug, Copy, Clone)]
pub struct ObjVertex {
    pub a_v: Vec3<f32>,
    pub a_uv: Vec2<f32>,
    pub a_vn: Vec3<f32>,
}

pub struct ObjMesh {
    pub name: String,
    pub geometry: ugli::VertexBuffer<ObjVertex>,
    pub material: Material,
}

pub struct Obj {
    pub meshes: Vec<ObjMesh>,
    // pub size: f32,
}

#[derive(ugli::Vertex, Debug, Clone)]
pub struct ObjInstance {
    pub i_model_matrix: Mat4<f32>,
    pub i_color: Rgba<f32>,
}

impl geng::LoadAsset for Obj {
    fn load(geng: &Geng, path: &std::path::Path) -> geng::AssetFuture<Self> {
        let geng = geng.clone();
        let path = path.to_owned();
        async move {
            let dir = path.parent().unwrap();
            let mut meshes = Vec::new();
            let obj_source = <String as geng::LoadAsset>::load(&geng, &path).await?;
            let mut current_name = String::from("__unnamed__");
            let mut v = Vec::new();
            let mut vn = Vec::new();
            let mut vt = Vec::new();
            let mut current_material: Option<Material> = Some(Material {
                name: "".to_owned(),
                texture: None,
                ambient_color: Rgba::WHITE,
                diffuse_color: Rgba::WHITE,
            });
            let mut current_geometry = Vec::new();
            let mut materials = HashMap::<String, Material>::new();
            for line in obj_source.lines().chain(std::iter::once("o _")) {
                let line = line.trim();
                if line.starts_with("v ") {
                    let mut parts = line.split_whitespace();
                    parts.next();
                    let x: f32 = parts.next().unwrap().parse().unwrap();
                    let y: f32 = parts.next().unwrap().parse().unwrap();
                    let z: f32 = parts.next().unwrap().parse().unwrap();
                    v.push(vec3(x, y, z));
                } else if line.starts_with("vn ") {
                    let mut parts = line.split_whitespace();
                    parts.next();
                    let x: f32 = parts.next().unwrap().parse().unwrap();
                    let y: f32 = parts.next().unwrap().parse().unwrap();
                    let z: f32 = parts.next().unwrap().parse().unwrap();
                    vn.push(vec3(x, y, z));
                } else if line.starts_with("vt ") {
                    let mut parts = line.split_whitespace();
                    parts.next();
                    let x: f32 = parts.next().unwrap().parse().unwrap();
                    let y: f32 = parts.next().unwrap().parse().unwrap();
                    vt.push(vec2(x, y));
                } else if line.starts_with("f ") {
                    let mut parts = line.split_whitespace();
                    parts.next();
                    let to_vertex = |s: &str| {
                        let mut parts = s.split('/');
                        let i_v: usize = parts.next().unwrap().parse().unwrap();
                        let i_uv: Option<usize> = match parts.next().unwrap() {
                            "" => None,
                            s => Some(s.parse().unwrap()),
                        };
                        let i_vn: Option<usize> = parts.next().map(|s| s.parse().unwrap());
                        ObjVertex {
                            a_v: v[i_v - 1],
                            a_vn: i_vn.map(|i| vn[i - 1]).unwrap_or(Vec3::ZERO),
                            a_uv: match i_uv {
                                Some(i_uv) => vt[i_uv - 1],
                                None => vec2(0.0, 0.0),
                            },
                        }
                    };
                    let mut cur = Vec::new();
                    for s in parts {
                        cur.push(to_vertex(s));
                    }
                    for i in 2..cur.len() {
                        current_geometry.push(cur[0]);
                        current_geometry.push(cur[i - 1]);
                        current_geometry.push(cur[i]);
                    }
                } else if line.starts_with("o ")
                    || line.starts_with("g ")
                    || line.starts_with("usemtl ")
                {
                    if !current_geometry.is_empty() {
                        meshes.push(ObjMesh {
                            name: current_name.clone(),
                            geometry: ugli::VertexBuffer::new_static(geng.ugli(), current_geometry),
                            material: current_material.clone().unwrap(),
                        });
                        current_geometry = Vec::new();
                    }
                    if line.starts_with("o ") || line.starts_with("g ") {
                        current_name = String::from(&line[2..]);
                    } else if let Some(name) = line.strip_prefix("usemtl ") {
                        current_material = Some(materials[name].clone());
                    }
                } else if let Some(mtl_path) = line.strip_prefix("mtllib ") {
                    for material in parse_mtl(&geng, dir, &dir.join(mtl_path)).await? {
                        materials.insert(material.name.clone(), material);
                    }
                }
            }
            Ok(Obj {
                meshes,
                // size,
            })
        }
        .boxed_local()
    }
    const DEFAULT_EXT: Option<&'static str> = Some("obj");
}

async fn parse_mtl(
    geng: &Geng,
    dir: &std::path::Path,
    path: &std::path::Path,
) -> anyhow::Result<Vec<Material>> {
    struct MaterialFuture {
        name: String,
        ambient_color: Rgba<f32>,
        diffuse_color: Rgba<f32>,
        texture: geng::AssetFuture<Option<ugli::Texture>>,
    }

    impl MaterialFuture {
        async fn into_future(self) -> Material {
            let texture = self.texture.await;
            let texture = texture.unwrap().map(Rc::new);
            Material {
                name: self.name,
                ambient_color: self.ambient_color,
                diffuse_color: self.diffuse_color,
                texture,
            }
        }
    }

    let mut materials = Vec::<MaterialFuture>::new();
    let mtl_source = <String as geng::LoadAsset>::load(geng, path).await?;
    let mut current_texture = future::ready(Ok(None)).boxed_local();
    let mut current_name = "__unnamed__";
    let mut current_ambient_color = Rgba::WHITE;
    let mut current_diffuse_color = Rgba::WHITE;
    for line in mtl_source.lines().chain(std::iter::once("newmtl _")) {
        let line = line.trim();
        if let Some(texture_path) = line.strip_prefix("map_Kd ") {
            let texture_path = texture_path.trim();
            // let texture_path = texture_path.split_whitespace().last().unwrap();
            // WTF .
            if texture_path != "." {
                current_texture =
                    <ugli::Texture as geng::LoadAsset>::load(geng, &dir.join(texture_path))
                        .map_ok(|texture| {
                            // make_repeated(&mut texture);
                            Some(texture)
                        })
                        .boxed_local();
            }
        } else if let Some(name) = line.strip_prefix("newmtl ") {
            let name = name.trim();
            materials.push(MaterialFuture {
                name: current_name.to_owned(),
                ambient_color: current_ambient_color,
                diffuse_color: current_diffuse_color,
                texture: mem::replace(&mut current_texture, future::ready(Ok(None)).boxed_local()),
                // ambient_color: current_ambient_color,
                // diffuse_color: current_diffuse_color,
            });
            current_name = name;
        } else if let Some(color) = line.strip_prefix("Ka ") {
            let mut parts = color.split_whitespace();
            let r: f32 = parts.next().unwrap().parse().unwrap();
            let g: f32 = parts.next().unwrap().parse().unwrap();
            let b: f32 = parts.next().unwrap().parse().unwrap();
            current_ambient_color = Rgba::new(r, g, b, 1.0);
        } else if let Some(color) = line.strip_prefix("Kd ") {
            let mut parts = color.split_whitespace();
            let r: f32 = parts.next().unwrap().parse().unwrap();
            let g: f32 = parts.next().unwrap().parse().unwrap();
            let b: f32 = parts.next().unwrap().parse().unwrap();
            current_diffuse_color = Rgba::new(r, g, b, 1.0);
        }
    }
    Ok(future::join_all(materials.into_iter().map(MaterialFuture::into_future)).await)
}
