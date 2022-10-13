const SCREEN_SIZE: (u32, u32) = (100, 100);
const TILE_SIZE: (u32, u32) = (8, 8);

const BG_COLOR: (f32, f32, f32) = (0.0, 1.0, 0.0);

enum Terrain {
    Grass,
    Resource,
    Water,
}

struct MapCell {
    terrain: Terrain,
}

impl MapCell {
    pub fn color(&self) -> ggez::graphics::Color {
        match self.terrain {
            Terrain::Grass => ggez::graphics::Color::new(0.2, 0.4, 0.0, 1.0),
            Terrain::Resource => ggez::graphics::Color::new(0.5, 0.25, 0.0, 1.0),
            Terrain::Water => ggez::graphics::Color::new(0.3, 0.7, 1.0, 1.0),
        }
    }
}

struct GameState {
    map: Vec<MapCell>,
}

impl GameState {
    pub fn new() -> Self {
        use rand::prelude::*;
        Self {
            map: (0..SCREEN_SIZE.0 * SCREEN_SIZE.1)
                .map(|_| MapCell {
                    terrain: match rand::thread_rng().gen_range(0..3) {
                        0 => Terrain::Grass,
                        1 => Terrain::Resource,
                        2 => Terrain::Water,
                        _ => unreachable!(),
                    },
                })
                .collect(),
        }
    }
}

impl ggez::event::EventHandler for GameState {
    fn update(&mut self, ctx: &mut ggez::Context) -> Result<(), ggez::GameError> {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut ggez::Context) -> Result<(), ggez::GameError> {
        // First we create a canvas that renders to the frame, and clear it to a (sort of) green color
        let mut canvas = ggez::graphics::Canvas::from_frame(
            ctx,
            ggez::graphics::CanvasLoadOp::Clear([BG_COLOR.0, BG_COLOR.1, BG_COLOR.2, 1.0].into()),
        );

        for (index, cell) in self.map.iter().enumerate() {
            let pos = (index as u32 / SCREEN_SIZE.0, index as u32 % SCREEN_SIZE.1);
            let color = cell.color();
            canvas.draw(
                &ggez::graphics::Quad,
                ggez::graphics::DrawParam::new()
                    .dest_rect(ggez::graphics::Rect::new(
                        (pos.0 * TILE_SIZE.0) as f32,
                        (pos.1 * TILE_SIZE.1) as f32,
                        TILE_SIZE.0 as f32,
                        TILE_SIZE.1 as f32,
                    ))
                    .color([color.r, color.g, color.b, color.a]),
            );
        }

        // Finally, we "flush" the draw commands.
        // Since we rendered to the frame, we don't need to tell ggez to present anything else,
        // as ggez will automatically present the frame image unless told otherwise.
        canvas.finish(ctx)?;

        // We yield the current thread until the next update
        ggez::timer::yield_now();

        Ok(())
    }
}

fn main() -> ggez::GameResult {
    // Here we use a ContextBuilder to setup metadata about our game. First the title and author
    let (ctx, events_loop) = ggez::ContextBuilder::new("snake", "Gray Olson")
        // Next we set up the window. This title will be displayed in the title bar of the window.
        .window_setup(ggez::conf::WindowSetup::default().title("Snake!"))
        // Now we get to set the size of the window, which we use our SCREEN_SIZE constant from earlier to help with
        .window_mode(ggez::conf::WindowMode::default().dimensions(
            (SCREEN_SIZE.0 * TILE_SIZE.0) as f32,
            (SCREEN_SIZE.1 * TILE_SIZE.1) as f32,
        ))
        // And finally we attempt to build the context and create the window. If it fails, we panic with the message
        // "Failed to build ggez context"
        .build()?;

    // Next we create a new instance of our GameState struct, which implements EventHandler
    let state = GameState::new();
    // And finally we actually run our game, passing in our context and state.
    ggez::event::run(ctx, events_loop, state)
}
