mod consts;
mod context;
mod hash_map_id;
mod map;
mod save;
mod scene;
mod tileset;
mod ui;

use consts::PKG_NAME;
use context::Context;
use miniquad::conf::Icon;
use scene::gameplay::Gameplay;
use scene::level_select::LevelSelect;
use scene::main_menu::MainMenu;
use scene::{EScene, Scene};

use macroquad::prelude::*;
fn window_conf() -> Conf {
    Conf {
        fullscreen: false,
        high_dpi: true,
        icon: Some(Icon {
            small: include_bytes!("../icons/16x16.rgba").to_owned(),
            medium: include_bytes!("../icons/32x32.rgba").to_owned(),
            big: include_bytes!("../icons/64x64.rgba").to_owned(),
        }),
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

    let mut current_scene: Box<dyn Scene> = match map::levels::TEST_LEVEL {
        Some(level) => Box::new(
            Gameplay::new(
                &mut ctx,
                Box::new(scene::GameOptions::Level(level).create().unwrap()),
            )
            .await,
        ),
        None => Box::new(MainMenu::new(&mut ctx).await),
    };

    loop {
        current_scene.update(&mut ctx);

        clear_background(BLACK);

        current_scene.draw(&mut ctx);

        if ctx.request_quit {
            break;
        }

        if let Some(escene) = ctx.switch_scene_to.take() {
            current_scene = match escene {
                EScene::MainMenu => Box::new(MainMenu::new(&mut ctx).await),
                EScene::Gameplay(map) => Box::new(Gameplay::new(&mut ctx, map).await),
                EScene::LevelSelect => Box::new(LevelSelect::new(&mut ctx)),
            };
        }

        next_frame().await;
    }
}
