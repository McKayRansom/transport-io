
use transport_io::consts::PKG_NAME;
use transport_io::context::Context;
use transport_io::map;
use transport_io::scene::gameplay::Gameplay;
use transport_io::scene::level_select::LevelSelect;
use transport_io::scene::main_menu::MainMenu;
use transport_io::scene::{EScene, GameOptions, Scene};

use miniquad::conf::Icon;
use macroquad::prelude::*;

fn window_conf() -> Conf {
    Conf {
        fullscreen: false,
        // high-dpi seems to change the zoom on webassembly??
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
                Box::new(GameOptions::Level(level).create().unwrap()),
            )
            .await,
        ),
        None => Box::new(MainMenu::new(&mut ctx).await),
    };

    loop {
        ctx.update();

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
