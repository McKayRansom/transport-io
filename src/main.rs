mod hash_map_id;
mod map;
mod tileset;
mod ui;
mod scene;
mod context;
mod save;
mod consts;

use consts::PKG_NAME;
use context::Context;
use scene::level_select::LevelSelect;
use scene::{EScene, Scene};
use scene::gameplay::Gameplay;
use scene::main_menu::MainMenu;

use macroquad::prelude::*;
fn window_conf() -> Conf {
    Conf {
        fullscreen: false,
        high_dpi: true,
        // icon: Some(Icon {
        //     small: include_bytes!("../icons/16x16.rgba").to_owned(),
        //     medium: include_bytes!("../icons/32x32.rgba").to_owned(),
        //     big: include_bytes!("../icons/64x64.rgba").to_owned(),
        // }),
        platform: miniquad::conf::Platform {
            linux_backend: miniquad::conf::LinuxBackend::WaylandWithX11Fallback,
            ..Default::default()
        },
        window_height: 720,
        window_resizable: true,
        window_title: String::from(PKG_NAME),
        window_width: 1280,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {

    let mut ctx = Context {
        ..Context::default().await
    };

    let mut current_scene: Box<dyn Scene> = Box::new(
        MainMenu::new(&mut ctx).await
        // Gameplay::new().await
    );

    loop {

        current_scene.update(&mut ctx);

        let color: Color = Color::from_hex(0x2b313f);
        clear_background(color);

        current_scene.draw(&mut ctx);

        if ctx.request_quit {
            break;
        }

        if let Some(escene) = ctx.switch_scene_to.clone() {
            current_scene = match escene {
                EScene::MainMenu => Box::new(MainMenu::new(&mut ctx).await),
                EScene::Gameplay(options) => Box::new(Gameplay::new(options).await),
                EScene::LevelSelect => Box::new(LevelSelect::new(&mut ctx)),
            };
            ctx.switch_scene_to = None;
        }

        next_frame().await;
    }
}
