//! A small snake game done after watching
//! <https://www.youtube.com/watch?v=HCwMb0KslX8>
//! to showcase ggez and how it relates/differs from piston.
//!
//! Note that this example is meant to highlight the general
//! structure of a ggez game. Some of the details may need to
//! be changed to scale the game. For example, if we needed to
//! draw hundreds or thousands of shapes, a `SpriteBatch` is going
//! to offer far better performance than the direct draw calls
//! that this example uses.
//!
//! Author: @termhn
//! Original repo: <https://github.com/termhn/ggez_snake>


mod grid;
mod path;
use ggez::event::MouseButton;
use ggez::input::keyboard::KeyCode;
use path::PathGrid;
use grid::GridPosition;
use grid::GRID_SIZE;
mod station;
mod vehicle;

use ggez::{
    event, graphics,
    input::keyboard::KeyInput,
    Context, GameResult,
};
use station::Station;
use vehicle::Vehicle;

use std::env;



// Next we define how large we want our actual window to be by multiplying
// the components of our grid size by its corresponding pixel size.
const SCREEN_SIZE: (f32, f32) = (
    GRID_SIZE.0 as f32 * grid::GRID_CELL_SIZE.0 as f32,
    GRID_SIZE.1 as f32 * grid::GRID_CELL_SIZE.1 as f32,
);

// Here we're defining how often we want our game to update. This will be
// important later so that we don't have our snake fly across the screen because
// it's moving a full tile every frame.
const DESIRED_FPS: u32 = 2;

const HELP_TEXT: &'static str = 
"Transport IO v0.0
Q: Quit
A: Add vehicle
S: Build station
D: Delete Road
F: Build Road
";


enum BuildMode {
    None,
    Vehicle,
    Station,
    Road,
    Delete,
}

/// Now we have the heart of our game, the `GameState`. This struct
/// will implement ggez's `EventHandler` trait and will therefore drive
/// everything else that happens in our game.
struct GameState {
    path_grid: PathGrid,
    snakes: Vec<vehicle::Vehicle>,
    stations: Vec<station::Station>,
    mouse_down: bool,
    build_mode: BuildMode,
}

impl GameState {

    pub fn new() -> Self {

        let path_grid = PathGrid::new();

        GameState {
            path_grid: path_grid,
            snakes: Vec::new(),
            stations: Vec::new(),
            mouse_down: false,
            build_mode: BuildMode::None,
        }
    }

    pub fn load_level(&mut self) {

        // let snake_pos = (GRID_SIZE.0 / 4, GRID_SIZE.1 / 2).into();
        // let snake_pos2: GridPosition = (GRID_SIZE.0 / 2, GRID_SIZE.1 / 2).into();

        // let new_snake2  = snake::Snake::new(snake_pos2, &mut self.path_grid);

        // self.snakes.push(new_snake2);

        let station_pos = (10, 10).into();
        let station_pos2 = (20, 15).into();

        let mut pos: GridPosition = station_pos;
        for i in 10..21 {
            pos.x = i;
            self.path_grid.add_allowed(pos);
        }

        for i in 10..16 {
            pos.y = i;
            self.path_grid.add_allowed(pos);
        }

        let new_station = station::Station::new(station_pos);
        let new_station2 = station::Station::new(station_pos2);

        let new_snake = vehicle::Vehicle::new(station_pos2, &mut self.path_grid);
        self.snakes.push(new_snake);

        self.stations.push(new_station);
        self.stations.push(new_station2);
    }
}

/// Now we implement `EventHandler` for `GameState`. This provides an interface
/// that ggez will call automatically when different events happen.
impl event::EventHandler<ggez::GameError> for GameState {
    /// Update will happen on every frame before it is drawn. This is where we update
    /// our game state to react to whatever is happening in the game world.
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        // Rely on ggez's built-in timer for deciding when to update the game, and how many times.
        // If the update is early, there will be no cycles, otherwises, the logic will run once for each
        // frame fitting in the time since the last update.
        while ctx.time.check_update_time(DESIRED_FPS) {

            for s in self.snakes.iter_mut() {
                s.update(&self.stations, &mut self.path_grid);
            }
        }

        Ok(())
    }

    /// draw is where we should actually render the game's current state.
    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        // First we create a canvas that renders to the frame, and clear it to a black
        let mut canvas =
            graphics::Canvas::from_frame(ctx, graphics::Color::from([0.0, 0.0, 0.0, 1.0]));

        for i in 0..GRID_SIZE.0 {
            for j in 0..GRID_SIZE.1 {
                let pos = GridPosition {x: i, y: j};
                if self.path_grid.is_allowed(pos) {
                    canvas.draw(
                        &graphics::Quad,
                        graphics::DrawParam::new()
                            .dest_rect(pos.into())
                            .color([0.3, 0.3, 0.3, 0.5]),
                    ); 
                }
            }
        }

        // Then we tell the snake and the food to draw themselves
        for s in self.snakes.iter() {
            s.draw(&mut canvas);
        }
        for s in self.stations.iter() {
            s.draw(&mut canvas);
        }

        let offset: f32 = 10.0;
        let dest_point = ggez::glam::Vec2::new(offset, offset);
        canvas.draw(
            graphics::Text::new(HELP_TEXT)
                .set_font("LiberationMono")
                .set_scale(32.),
            dest_point,
        );

        // Finally, we "flush" the draw commands.
        // Since we rendered to the frame, we don't need to tell ggez to present anything else,
        // as ggez will automatically present the frame image unless told otherwise.
        canvas.finish(ctx)?;

        // We yield the current thread until the next update
        ggez::timer::yield_now();
        // And return success.
        Ok(())
    }

    /// `key_down_event` gets fired when a key gets pressed.
    fn key_down_event(&mut self, ctx: &mut Context, input: KeyInput, repeat: bool) -> GameResult {
        if repeat {
            return Ok(());
        }
        // Here we attempt to convert the Keycode into a Direction using the helper
        // we defined earlier.
        if let Some(keycode) = input.keycode {
            match keycode {
                KeyCode::Q => {
                    ctx.request_quit();
                }
                KeyCode::A => {
                    self.build_mode = BuildMode::Vehicle;
                }
                KeyCode::S => {
                    self.build_mode = BuildMode::Station;
                }
                KeyCode::D => {
                    self.build_mode = BuildMode::Delete;
                }
                KeyCode::F => {
                    self.build_mode = BuildMode::Road;
                }
                _ => {

                }
            }
        }

        Ok(())
    }

    fn mouse_button_down_event(
        &mut self,
        _ctx: &mut Context,
        button: MouseButton,
        x: f32,
        y: f32,
    ) -> GameResult {
        self.mouse_down = true;
        let pos = GridPosition::from_screen(x, y);
        println!("Mouse button pressed: {button:?}, pos: {pos:?} x: {x}, y: {y}");
        match self.build_mode {

            BuildMode::Vehicle => {
                if self.path_grid.is_allowed(pos) {
                    self.snakes.push(Vehicle::new(pos, &mut self.path_grid))
                }
            }
            BuildMode::Station => {
                if !self.path_grid.is_allowed(pos) {
                    self.path_grid.add_allowed(pos);
                }
                self.stations.push(Station::new(pos))

            }
            BuildMode::Road => {
                if !self.path_grid.is_allowed(pos) {
                    self.path_grid.add_allowed(pos);
                }
            }
            BuildMode::Delete => {
                if self.path_grid.is_allowed(pos) {
                    self.path_grid.remove_allowed(pos);
                }
            }
            _ => {

            }
        }
        Ok(())
    }


    fn mouse_button_up_event(
        &mut self,
        _ctx: &mut Context,
        button: MouseButton,
        x: f32,
        y: f32,
    ) -> GameResult {
        self.mouse_down = false;
        println!("Mouse button released: {button:?}, x: {x}, y: {y}");
        Ok(())
    }

    fn mouse_motion_event(
        &mut self,
        _ctx: &mut Context,
        x: f32,
        y: f32,
        xrel: f32,
        yrel: f32,
    ) -> GameResult {
        if self.mouse_down {
            // Mouse coordinates are PHYSICAL coordinates, but here we want logical coordinates.

            // If you simply use the initial coordinate system, then physical and logical
            // coordinates are identical.
            // self.pos_x = x;
            // self.pos_y = y;

            // If you change your screen coordinate system you need to calculate the
            // logical coordinates like this:
            /*
            let screen_rect = graphics::screen_coordinates(_ctx);
            let size = graphics::window(_ctx).inner_size();
            self.pos_x = (x / (size.width  as f32)) * screen_rect.w + screen_rect.x;
            self.pos_y = (y / (size.height as f32)) * screen_rect.h + screen_rect.y;
            */

            let pos = GridPosition::from_screen(x, y);
            match self.build_mode {

                // BuildMode::Vehicle => {
                    // if self.path_grid.is_allowed(pos) {
                        // self.snakes.push(Vehicle::new(pos, &mut self.path_grid))
                    // }
                // }
                // BuildMode::Station => {
                    // if !self.path_grid.is_allowed(pos) {
                    // }
                    // self.stations.push(Station::new(pos))

                // }
                BuildMode::Road => {
                    if !self.path_grid.is_allowed(pos) {
                        self.path_grid.add_allowed(pos);
                    }
                }
                BuildMode::Delete => {
                    if self.path_grid.is_allowed(pos) {
                        self.path_grid.remove_allowed(pos);
                    }
                }
                _ => {

                }
            }
        }
        println!("Mouse motion, x: {x}, y: {y}, relative x: {xrel}, relative y: {yrel}");
        Ok(())
    }

}

fn main() -> GameResult {

    let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = std::path::PathBuf::from(manifest_dir);
        path.push("resources");
        path
    } else {
        std::path::PathBuf::from("./resources")
    };

    // Here we use a ContextBuilder to setup metadata about our game. First the title and author
    let (mut ctx, events_loop) = ggez::ContextBuilder::new("snake", "Gray Olson")
        // Next we set up the window. This title will be displayed in the title bar of the window.
        .window_setup(ggez::conf::WindowSetup::default().title("Snake!"))
        // Now we get to set the size of the window, which we use our SCREEN_SIZE constant from earlier to help with
        .window_mode(ggez::conf::WindowMode::default().dimensions(SCREEN_SIZE.0, SCREEN_SIZE.1))
        // And finally we attempt to build the context and create the window. If it fails, we panic with the message
        // "Failed to build ggez context"
        .add_resource_path(resource_dir)
        .build()?;

    ctx.gfx.add_font(
        "LiberationMono",
        graphics::FontData::from_path(&ctx.fs, "/LiberationMono-Regular.ttf")?,
    );

    // Next we create a new instance of our GameState struct, which implements EventHandler
    let mut state = GameState::new();

    // state.key_manager.add_handler(KeyHandler {key: KeyCode::Q, func: game_quit, help: "Q: Quit the game"});

    state.load_level();
    // And finally we actually run our game, passing in our context and state.
    event::run(ctx, events_loop, state)
}