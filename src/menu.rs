use macroquad::{
    color::{BLACK, WHITE},
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

    let x =-300. + screen_width() / 2.;
    let y = -menu_height + screen_height() / 2.;
    let font_size = 120.0;

    let shadow_y = y + 5.;
    let shadow_x = x + 5.;

    draw_text(
        "Transport IO",
        shadow_x, shadow_y, font_size,
        BLACK,
    );

    draw_text(
        "Transport IO",
        x, y, font_size,
        WHITE,
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
