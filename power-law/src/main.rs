use rand::prelude::*;
use raylib::prelude::*;

const W: i32 = 1024;
const H: i32 = 768;

#[derive(Clone, PartialEq)]
enum State {
    None,
    Tree,
    Fire(i32),
}

struct Simulation {
    cells: Vec<State>,
    width: usize,
    growth_rate: f32,
    transmission_rate: f32,
    lightning_rate: f32,
}

impl Simulation {
    fn step(&mut self) {
        let mut rng = rand::rng();
        for i in 0..self.cells.len() {
            match self.cells[i] {
                State::None => {
                    if self.growth_rate > rng.random() {
                        self.cells[i] = State::Tree
                    }
                }
                State::Tree => {
                    if self.lightning_rate > rng.random() {
                        self.cells[i] = State::Fire(0)
                    }
                }
                State::Fire(age) => {
                    if age >= 2 {
                        self.cells[i] = State::None;
                        continue;
                    }
                    self.cells[i] = State::Fire(age + 1);
                    for idx in [
                        i + 1,                          // E
                        i.wrapping_sub(1),              // W
                        i + self.width,                 // S
                        i + self.width - 1,             // SE
                        i + self.width + 1,             // SW
                        i.wrapping_sub(self.width),     // N
                        i.wrapping_sub(self.width - 1), // NE
                        i.wrapping_sub(self.width + 1), // NW
                    ] {
                        if idx >= self.cells.len() {
                            continue;
                        }
                        if self.cells[idx] == State::Tree && self.transmission_rate > rng.random() {
                            self.cells[idx] = State::Fire(0);
                        }
                    }
                }
            }
        }
    }

    fn render(&self, dh: &mut RaylibDrawHandle) {
        for (idx, c) in self.cells.iter().enumerate() {
            let c = match c {
                State::None => Color::BLACK,
                State::Tree => Color::GREEN,
                State::Fire(0) => Color::YELLOW,
                State::Fire(1) => Color::ORANGE,
                State::Fire(2) => Color::RED,
                State::Fire(_) => Color::PURPLE,
            };
            let (scale_x, scale_y) = (
                W / self.width as i32,
                H / ((self.cells.len() / self.width) as i32),
            );
            let (x, y) = (
                scale_x * (idx % self.width) as i32,
                scale_y * (idx / self.width) as i32,
            );
            dh.draw_rectangle(x, y, scale_x, scale_y, c);
        }
        let txt = format!(
            "growth: {:.5}, lightning: {:e}, transmission: {:.5}",
            self.growth_rate, self.lightning_rate, self.transmission_rate,
        );
        let w = dh.measure_text(&txt, 24);
        dh.draw_rectangle(0, 0, w, 24, Color::BLACK);
        dh.draw_text(&txt, 0, 0, 24, Color::RED);
    }
}

fn main() {
    let mut rng = rand::rng();
    // let mut cells = vec![State::None; 1024 * 768];
    let mut cells = vec![State::None; 256 * 192];
    let mut simulation = Simulation {
        cells,
        width: 256,
        growth_rate: 1e-2,
        lightning_rate: 1e-5,
        transmission_rate: 0.5,
    };
    let (mut rl, thread) = raylib::init().size(W, H).title("power law").build();
    rl.set_target_fps(60);
    let mut paused = false;
    while !rl.window_should_close() {
        if rl.is_key_released(KeyboardKey::KEY_UP) {
            simulation.growth_rate *= 1.1;
        }
        if rl.is_key_released(KeyboardKey::KEY_DOWN) {
            simulation.growth_rate *= 0.9;
        }
        if rl.is_key_released(KeyboardKey::KEY_RIGHT) {
            simulation.lightning_rate *= 1.1;
        }
        if rl.is_key_released(KeyboardKey::KEY_LEFT) {
            simulation.lightning_rate *= 0.9;
        }
        if rl.is_key_released(KeyboardKey::KEY_COMMA) {
            simulation.transmission_rate *= 0.9;
        }
        if rl.is_key_released(KeyboardKey::KEY_PERIOD) {
            simulation.transmission_rate *= 1.1;
        }
        if rl.is_key_released(KeyboardKey::KEY_SPACE) {
            paused = !paused
        }

        if !paused {
            simulation.step();
        }
        let mut dh = rl.begin_drawing(&thread);
        simulation.render(&mut dh);
    }

    println!("Hello, world!");
}
