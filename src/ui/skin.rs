use macroquad::{color::Color, math::RectOffset, ui::{root_ui, Skin}};

pub const MENU_FONT_SIZE: u16 = 48;
pub const MENU_MARGIN: f32 = 16.;

pub async fn init() {
    let skin2 = {
        // let font = load_ttf_font("examples/ui_assets/MinimalPixel v2.ttf")
        //     .await
        //     .unwrap();
        // let label_style = root_ui()
        //     .style_builder()
        //     .with_font(&font)
        //     .unwrap()
        //     .text_color(Color::from_rgba(120, 120, 120, 255))
        //     .font_size(15)
        //     .build();

        let window_color = Color::from_hex(0x585858);

        let window_style = root_ui()
            .style_builder()
            // .background(
            //     Image::from_file_with_format(
            //         include_bytes!("../examples/ui_assets/window_background_2.png"),
            //         None,
            //     )
            //     .unwrap(),
            // )
            .color_inactive(window_color)
            .color_hovered(window_color)
            .color_selected(window_color)
            .color_clicked(window_color)
            .color(window_color)
            // .font_size(120)
            // .text_color(WHITE)
            // .background_margin(RectOffset::new(52.0, 52.0, 52.0, 52.0))
            .margin(RectOffset::new(16.0, 16.0, 16.0, 16.0))
            .build();

        let button_style = root_ui()
            .style_builder()
        // .background(
        //     Image::from_file_with_format(
        //         include_bytes!("../examples/ui_assets/button_background_2.png"),
        //         None,
        //     )
        //     .unwrap(),
        // )
        // .background_margin(RectOffset::new(8.0, 8.0, 8.0, 8.0))
        // .background_hovered(
        //     Image::from_file_with_format(
        //         include_bytes!("../examples/ui_assets/button_hovered_background_2.png"),
        //         None,
        //     )
        //     .unwrap(),
        // )
        // .background_clicked(
        //     Image::from_file_with_format(
        //         include_bytes!("../examples/ui_assets/button_clicked_background_2.png"),
        //         None,
        //     )
        //     .unwrap(),
        // )
        // .with_font(&font)
        // .unwrap()
        .margin(RectOffset::new(16., 16., 2., 2.))
        .color_clicked(Color::from_rgba(187, 187, 187, 255))
        .color_hovered(Color::from_rgba(170, 170, 170, 235))
        .text_color(Color::from_rgba(0, 0, 0, 255))
        // .text_color(Color::from_rgba(180, 180, 100, 255))
        .font_size(MENU_FONT_SIZE)
        .build();

        // let checkbox_style = root_ui()
        //     .style_builder()
        //     .background(
        //         Image::from_file_with_format(
        //             include_bytes!("../examples/ui_assets/checkbox_background.png"),
        //             None,
        //         )
        //         .unwrap(),
        //     )
        //     .background_hovered(
        //         Image::from_file_with_format(
        //             include_bytes!("../examples/ui_assets/checkbox_hovered_background.png"),
        //             None,
        //         )
        //         .unwrap(),
        //     )
        //     .background_clicked(
        //         Image::from_file_with_format(
        //             include_bytes!("../examples/ui_assets/checkbox_clicked_background.png"),
        //             None,
        //         )
        //         .unwrap(),
        //     )
        //     .build();

        // let editbox_style = root_ui()
        //     .style_builder()
        //     .background(
        //         Image::from_file_with_format(
        //             include_bytes!("../examples/ui_assets/editbox_background.png"),
        //             None,
        //         )
        //         .unwrap(),
        //     )
        //     .background_margin(RectOffset::new(2., 2., 2., 2.))
        //     .with_font(&font)
        //     .unwrap()
        //     .text_color(Color::from_rgba(120, 120, 120, 255))
        //     .font_size(25)
        //     .build();

        // let combobox_style = root_ui()
        //     .style_builder()
        //     .background(
        //         Image::from_file_with_format(
        //             include_bytes!("../examples/ui_assets/combobox_background.png"),
        //             None,
        //         )
        //         .unwrap(),
        //     )
        //     .background_margin(RectOffset::new(4., 25., 6., 6.))
        //     .with_font(&font)
        //     .unwrap()
        //     .text_color(Color::from_rgba(120, 120, 120, 255))
        //     .color(Color::from_rgba(210, 210, 210, 255))
        //     .font_size(25)
        //     .build();

        let margin = 16.;

        Skin {
            window_style,
            button_style,
            // label_style,
            // checkbox_style,
            // editbox_style,
            // combobox_style,

            margin,
            
            ..root_ui().default_skin()
        }
    };

    root_ui().push_skin(&skin2);
}
