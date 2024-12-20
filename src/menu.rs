use macroquad::{
    color::GRAY,
    math::vec2,
    text::draw_text,
    ui::{hash, root_ui, widgets},
    window::{screen_height, screen_width},
};

pub enum MenuSelect {
    Continue,
    NewGame,
    SaveGame,
    LoadGame,
}

pub fn draw() -> Option<MenuSelect> {
    let mut result: Option<MenuSelect> = None;

    let menu_width = 100.;
    let menu_height = 100.;

    let menu_item_height = 25.;
    let menu_item_pad = 5.;

    draw_text(
        "Transport IO",
        -300. + screen_width() / 2.,
        -menu_height + screen_height() / 2.,
        120.0,
        GRAY,
    );

    widgets::Window::new(
        hash!(),
        vec2(
            screen_width() / 2.0 - (menu_width / 2.),
            screen_height() / 2.0 - (menu_height / 2.),
        ),
        vec2(menu_width, menu_height),
    )
    .titlebar(false)
    .movable(false)
    .ui(&mut root_ui(), |ui| {
        let mut position = vec2(0., 0.);
        if ui.button(position, "Continue") {
            result = Some(MenuSelect::Continue)
        }
        position.y += menu_item_height + menu_item_pad;
        if ui.button(position, "New Game") {
            result = Some(MenuSelect::NewGame)
        }
        position.y += menu_item_height + menu_item_pad;
        if ui.button(position, "Save Game") {
            result = Some(MenuSelect::SaveGame)
        }
        position.y += menu_item_height + menu_item_pad;
        if ui.button(position, "Load Game") {
            result = Some(MenuSelect::LoadGame)
        }
    });

    result
}
