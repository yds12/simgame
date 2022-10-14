const SCREEN_SIZE: (usize, usize) = (307, 161);
const TILE_SIZE: (usize, usize) = (4, 4);

const BG_COLOR: (f32, f32, f32) = (1.0, 0.0, 1.0);

const SMOOTHING: usize = 30;

const fn get_coord(index: usize) -> (usize, usize) {
    (index % SCREEN_SIZE.0, index / SCREEN_SIZE.0)
}

const fn get_index(x: usize, y: usize) -> usize {
    y * SCREEN_SIZE.0 + x
}

fn prob(p: f64) -> bool {
    use rand::prelude::*;
    let val = rand::thread_rng().gen_range(0..1000) as f64 / 1000.0;

    val < p
}

fn rand_index(size: usize) -> usize {
    use rand::prelude::*;
    rand::thread_rng().gen_range(0..size)
}

fn get_cross_neighbor_coords(x: usize, y: usize) -> Vec<(usize, usize)> {
    let mut neighs = Vec::new();

    if x > 0 {
        neighs.push((x - 1, y));
    }
    if y > 0 {
        neighs.push((x, y - 1));
    }
    if x < SCREEN_SIZE.0 - 1 {
        neighs.push((x + 1, y));
    }
    if y < SCREEN_SIZE.1 - 1 {
        neighs.push((x, y + 1));
    }

    neighs
}

fn get_neighbor_coords(x: usize, y: usize) -> Vec<(usize, usize)> {
    let mut neighs = get_cross_neighbor_coords(x, y);

    if x > 0 && y > 0 {
        neighs.push((x - 1, y - 1));
    }
    if x > 0 && y < SCREEN_SIZE.1 - 1 {
        neighs.push((x - 1, y + 1));
    }
    if x < SCREEN_SIZE.0 - 1 && y > 0 {
        neighs.push((x + 1, y - 1));
    }
    if x < SCREEN_SIZE.0 - 1 && y < SCREEN_SIZE.1 - 1 {
        neighs.push((x + 1, y + 1));
    }

    neighs
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Person;

#[derive(Clone, Copy, Debug, PartialEq)]
enum Terrain {
    Land,
    Resource,
    Water,
}

struct MapCell {
    terrain: Terrain,
    pop: Vec<Person>,
}

impl From<Terrain> for MapCell {
    fn from(terrain: Terrain) -> Self {
        Self {
            terrain,
            pop: Vec::new(),
        }
    }
}

impl MapCell {
    pub fn color(&self) -> ggez::graphics::Color {
        if self.pop.len() == 0 {
            match self.terrain {
                Terrain::Resource => ggez::graphics::Color::new(0.4, 0.8, 0.0, 1.0),
                Terrain::Land => ggez::graphics::Color::new(0.7, 0.55, 0.0, 1.0),
                Terrain::Water => ggez::graphics::Color::new(0.3, 0.7, 1.0, 1.0),
            }
        } else {
            let interp = match self.pop.len() {
                pop if (pop as f64).log(10.0) < 10.0 => (pop as f64).log(10.0) as f32 * 0.1,
                _ => 1.0
            };

            ggez::graphics::Color::new(1.0 - interp, 0.0, interp, 1.0)
        }
    }
}

struct GameState {
    map: Vec<MapCell>,
}

impl GameState {
    pub fn new() -> Self {
        use rand::prelude::*;
        let mut state = Self {
            map: (0..SCREEN_SIZE.0 * SCREEN_SIZE.1)
                .map(|_| {
                    match rand::thread_rng().gen_range(0..3) {
                        0 => Terrain::Land,
                        1 => Terrain::Resource,
                        2 => Terrain::Water,
                        _ => unreachable!(),
                    }
                    .into()
                })
                .collect(),
        };

        for _ in 0..SMOOTHING {
            state.smoothen();
        }
        state.populate();

        state
    }

    fn populate(&mut self) {
        use rand::prelude::*;
        let mut rng = rand::thread_rng();

        for cell in &mut self.map {
            if cell.terrain == Terrain::Resource && prob(0.005) {
                cell.pop = (0..rng.gen_range(0..10000)).map(|_| Person).collect();
            }
            else if cell.terrain == Terrain::Land && prob(0.001) {
                cell.pop = (0..rng.gen_range(0..1000)).map(|_| Person).collect();
            }
        }
    }

    fn smoothen(&mut self) {
        use rand::prelude::*;

        let mut commons = Vec::new();
        let len = self.map.len();

        for index in 0..len {
            let common_vec = self.most_common_neighbor(index);
            let ind = rand::thread_rng().gen_range(0..common_vec.len());
            commons.push(common_vec[ind]);
        }

        for index in 0..len {
            self.map[index].terrain = commons[index];
        }
    }

    fn most_common_neighbor(&self, index: usize) -> Vec<Terrain> {
        let neighs = self.get_neighbors(index);
        let mut counts = (0, 0, 0);

        for neigh in neighs {
            match neigh {
                Terrain::Land => counts.0 += 1,
                Terrain::Resource => counts.1 += 1,
                Terrain::Water => counts.2 += 1,
            }
        }

        // TODO: improve this
        match counts {
            (g, r, w) if w == g && w == r => {
                vec![Terrain::Water, Terrain::Resource, Terrain::Land]
            }
            (g, r, w) if w > g && w == r => vec![Terrain::Water, Terrain::Resource],
            (g, r, w) if w > r && w == g => vec![Terrain::Water, Terrain::Land],
            (g, r, w) if w > g && w > r => vec![Terrain::Water],
            (g, r, _) if r == g => vec![Terrain::Land, Terrain::Resource],
            (g, r, _) if r > g => vec![Terrain::Resource],
            _ => vec![Terrain::Land],
        }
    }

    fn get_neighbors(&self, index: usize) -> Vec<Terrain> {
        let mut terrs = Vec::new();
        let pos = get_coord(index);
        let neigh_coords = get_neighbor_coords(pos.0, pos.1);

        for (x, y) in neigh_coords {
            terrs.push(self.map[get_index(x, y)].terrain);
        }

        terrs
    }
}

impl ggez::event::EventHandler for GameState {
    fn update(&mut self, ctx: &mut ggez::Context) -> Result<(), ggez::GameError> {
        for index in 0..self.map.len() {
            let pos = get_coord(index);
            let neighs = get_cross_neighbor_coords(pos.0, pos.1);
            let this_pop = self.map[index].pop.len();
            let this_terr = self.map[index].terrain;

            if this_pop > 0 {
                if (this_terr == Terrain::Resource && prob(0.009)) ||
                   (this_terr == Terrain::Land && prob(0.003)) { // grow
                    let amount = (this_pop as f64 * 0.01) as usize;

                    for _ in 0..amount {
                        self.map[index].pop.push(Person);
                    }
                }
                if prob(0.005) { // migrate
                    let amount = (this_pop as f64 * 0.01) as usize;
                    let neigh_pos = neighs[rand_index(neighs.len())];
                    let neigh_ix = get_index(neigh_pos.0, neigh_pos.1);

                    if self.map[neigh_ix].terrain != Terrain::Water {
                        for _ in 0..amount {
                            self.map[index].pop.pop();
                            self.map[neigh_ix].pop.push(Person);
                        }
                    }
                }
                if (this_terr == Terrain::Resource && prob(0.008)) ||
                   (this_terr == Terrain::Land && prob(0.004)) { // shrink
                    let amount = match this_pop {
                        p if p > 10 => (this_pop as f64 * 0.01) as usize,
                        p => p
                    };

                    for _ in 0..amount {
                        self.map[index].pop.pop();
                    }
                }
            }
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut ggez::Context) -> Result<(), ggez::GameError> {
        use ggez::graphics::Drawable; // for Mesh::draw

        let mut canvas = ggez::graphics::Canvas::from_frame(
            ctx,
            ggez::graphics::CanvasLoadOp::Clear([BG_COLOR.0, BG_COLOR.1, BG_COLOR.2, 1.0].into()),
        );

        let mut mesh = ggez::graphics::MeshBuilder::new();

        for (index, cell) in self.map.iter().enumerate() {
            let pos = get_coord(index);
            let color = cell.color();

            mesh.rectangle(
                ggez::graphics::DrawMode::fill(),
                ggez::graphics::Rect::new(
                    (pos.0 * TILE_SIZE.0) as f32,
                    (pos.1 * TILE_SIZE.1) as f32,
                    TILE_SIZE.0 as f32,
                    TILE_SIZE.1 as f32,
                ),
                color,
            )?;
        }

        if ggez::timer::ticks(ctx) % 60 == 0 {
            println!("FPS: {}", ggez::timer::fps(ctx));
        }

        let mesh = ggez::graphics::Mesh::from_data(ctx, mesh.build());
        canvas.draw(&mesh, ggez::graphics::DrawParam::new());

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
    let (ctx, events_loop) = ggez::ContextBuilder::new("SIM", "yds12")
        // Next we set up the window. This title will be displayed in the title bar of the window.
        .window_setup(ggez::conf::WindowSetup::default().title("SIM"))
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
