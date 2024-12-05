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
use path::PathGrid;
use grid::GridPosition;
use grid::GRID_SIZE;
mod food;
mod snake;
use oorandom::Rand32;

use ggez::{
    event, graphics,
    input::keyboard::KeyInput,
    Context, GameResult,
};



// Next we define how large we want our actual window to be by multiplying
// the components of our grid size by its corresponding pixel size.
const SCREEN_SIZE: (f32, f32) = (
    GRID_SIZE.0 as f32 * grid::GRID_CELL_SIZE.0 as f32,
    GRID_SIZE.1 as f32 * grid::GRID_CELL_SIZE.1 as f32,
);

// Here we're defining how often we want our game to update. This will be
// important later so that we don't have our snake fly across the screen because
// it's moving a full tile every frame.
const DESIRED_FPS: u32 = 8;



/// Now we have the heart of our game, the `GameState`. This struct
/// will implement ggez's `EventHandler` trait and will therefore drive
/// everything else that happens in our game.
struct GameState {
    path_grid: PathGrid,
    snakes: Vec<snake::Snake>,
    food: food::Station,
    gameover: bool,
    rng: Rand32,
}

impl GameState {

    pub fn new() -> Self {

        // And we seed our RNG with the system RNG.
        let mut seed: [u8; 8] = [0; 8];
        getrandom::getrandom(&mut seed[..]).expect("Could not create RNG seed");
        let mut rng = Rand32::new(u64::from_ne_bytes(seed));
        // Then we choose a random place to put our piece of food using the helper we made
        // earlier.
        let food_pos = GridPosition::random(&mut rng, GRID_SIZE.0, GRID_SIZE.1);

        let path_grid = PathGrid::new();

        GameState {
            path_grid: path_grid,

            snakes: Vec::new(),
            food: food::Station::new(food_pos),
            gameover: false,
            rng,
        }
    }

    pub fn load_level(&mut self) {

        let snake_pos = (GRID_SIZE.0 / 4, GRID_SIZE.1 / 2).into();
        let snake_pos2: GridPosition = (GRID_SIZE.0 / 2, GRID_SIZE.1 / 2).into();

        let new_snake = snake::Snake::new(snake_pos, &mut self.path_grid);

        let new_snake2  = snake::Snake::new(snake_pos2, &mut self.path_grid);

        self.snakes.push(new_snake);
        self.snakes.push(new_snake2);
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
            // We check to see if the game is over. If not, we'll update. If so, we'll just do nothing.
            if !self.gameover {
                // Here we do the actual updating of our game world. First we tell the snake to update itself,
                // passing in a reference to our piece of food.
                for s in self.snakes.iter_mut() {
                    s.update(&self.food, &mut self.path_grid);
                    // Next we check if the snake ate anything as it updated.
                    if let Some(ate) = s.ate {
                        // If it did, we want to know what it ate.
                        match ate {
                            // If it ate a piece of food, we randomly select a new position for our piece of food
                            // and move it to this new position.
                            snake::Ate::Food => {
                                let new_food_pos =
                                    GridPosition::random(&mut self.rng, GRID_SIZE.0, GRID_SIZE.1);
                                self.food.pos = new_food_pos;
                            }
                            // If it ate itself, we set our gameover state to true.
                            snake::Ate::Itself => {
                                self.gameover = true;
                            }
                        }
                    }
                }
                
            }
        }

        Ok(())
    }

    /// draw is where we should actually render the game's current state.
    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        // First we create a canvas that renders to the frame, and clear it to a black
        let mut canvas =
            graphics::Canvas::from_frame(ctx, graphics::Color::from([0.0, 0.0, 0.0, 1.0]));

        // Then we tell the snake and the food to draw themselves
        for s in self.snakes.iter() {
            s.draw(&mut canvas);
        }
        self.food.draw(&mut canvas);

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
    fn key_down_event(&mut self, _ctx: &mut Context, _input: KeyInput, _repeat: bool) -> GameResult {
        // Here we attempt to convert the Keycode into a Direction using the helper
        // we defined earlier.
        // if let Some(dir) = input.keycode.and_then(Direction::from_keycode) {
        //     self.snake.keydir(dir);
        // }
        Ok(())
    }
}

fn main() -> GameResult {
    // Here we use a ContextBuilder to setup metadata about our game. First the title and author
    let (ctx, events_loop) = ggez::ContextBuilder::new("snake", "Gray Olson")
        // Next we set up the window. This title will be displayed in the title bar of the window.
        .window_setup(ggez::conf::WindowSetup::default().title("Snake!"))
        // Now we get to set the size of the window, which we use our SCREEN_SIZE constant from earlier to help with
        .window_mode(ggez::conf::WindowMode::default().dimensions(SCREEN_SIZE.0, SCREEN_SIZE.1))
        // And finally we attempt to build the context and create the window. If it fails, we panic with the message
        // "Failed to build ggez context"
        .build()?;

    // Next we create a new instance of our GameState struct, which implements EventHandler
    let mut state = GameState::new();

    state.load_level();
    // And finally we actually run our game, passing in our context and state.
    event::run(ctx, events_loop, state)
}