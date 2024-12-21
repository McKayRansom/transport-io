
use macroquad::{texture::{load_texture, FilterMode, Texture2D}, ui::Ui};


pub struct Grades {
    texture_a: Texture2D,
    texture_b: Texture2D,
    texture_c: Texture2D,
    texture_f: Texture2D,
}

impl Grades {

    pub async fn new() -> Self {
        let grades = Grades {
            texture_a: load_texture("resources/grades_a.png").await.unwrap(),
            texture_b: load_texture("resources/grades_b.png").await.unwrap(),
            texture_c: load_texture("resources/grades_c.png").await.unwrap(),
            texture_f: load_texture("resources/grades_f.png").await.unwrap(),
        };

        grades.texture_a.set_filter(FilterMode::Nearest);
        grades.texture_b.set_filter(FilterMode::Nearest);
        grades.texture_c.set_filter(FilterMode::Nearest);
        grades.texture_f.set_filter(FilterMode::Nearest);

        grades
    }

    pub fn get_texture(&self, percent: f32) -> Texture2D {
        if percent >= 0.9 {
            self.texture_a.clone()
        } else if percent >= 0.8 {
            self.texture_b.clone()
        } else if percent >= 0.7 {
            self.texture_c.clone()
        } else {
            self.texture_f.clone()
        }
    }

    pub fn draw(&self, ui: &mut Ui, percent: f32) {
        ui.texture(self.get_texture(percent), 32., 32.);
    }
}