use geng::prelude::*;

#[derive(geng::Assets)]
pub struct Assets {}

pub struct Game {
    geng: Geng,
    assets: Rc<Assets>,
}

impl Game {
    pub fn new(geng: &Geng, assets: &Rc<Assets>) -> Self {
        Self {
            geng: geng.clone(),
            assets: assets.clone(),
        }
    }
}

impl geng::State for Game {
    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        ugli::clear(framebuffer, Some(Rgba::BLUE), Some(1.0), None);
    }
}

fn main() {
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
