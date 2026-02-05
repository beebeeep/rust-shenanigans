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
    cells: Vec<Vec<State>>,
    width: usize,
    height: usize,
    growth_rate: f32,
    transmission_rate: f32,
    lightning_rate: f32,
}

impl Simulation {
    fn step(&mut self) {
        let mut rng = rand::rng();
        let mut changes = Vec::with_capacity(128);
        for x in 0..self.width {
            for y in 0..self.height {
                match self.cells[x][y] {
                    State::None => {
                        if self.growth_rate > rng.random() {
                            changes.push(((x, y), State::Tree));
                        }
                    }
                    State::Tree => {
                        if self.lightning_rate > rng.random() {
                            changes.push(((x, y), State::Fire(0)));
                        }
                    }
                    State::Fire(age) => {
                        if age >= 2 {
                            changes.push(((x, y), State::None));
                            continue;
                        }
                        changes.push(((x, y), State::Fire(age + 1)));
                        for dx in -1..=1 {
                            for dy in -1..=1 {
                                let (x, y) = (x as i32 + dx, y as i32 + dy);
                                if (dx == 0 && dy == 0)
                                    || x < 0
                                    || y < 0
                                    || x >= self.width as i32
                                    || y >= self.height as i32
                                {
                                    continue;
                                }
                                let (x, y) = (x as usize, y as usize);
                                if self.cells[x][y] == State::Tree
                                    && self.transmission_rate > rng.random()
                                {
                                    changes.push(((x, y), State::Fire(0)));
                                }
                            }
                        }
                    }
                }
            }
        }
        for ((x, y), s) in changes {
            self.cells[x][y] = s
        }
    }

    fn render(&self, dh: &mut RaylibDrawHandle) {
        for x in 0..self.width as i32 {
            for y in 0..self.height as i32 {
                let c = match self.cells[x as usize][y as usize] {
                    State::None => Color::BLACK,
                    State::Tree => Color::GREEN,
                    State::Fire(0) => Color::YELLOW,
                    State::Fire(1) => Color::ORANGE,
                    State::Fire(2) => Color::RED,
                    State::Fire(_) => Color::PURPLE,
                };
                let (scale_x, scale_y) = (W / self.width as i32, H / (self.height) as i32);
                dh.draw_rectangle(x * scale_x, y * scale_y, scale_x, scale_y, c);
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
}

fn main() {
    // let mut cells = vec![State::None; 1024 * 768];
    // let cells = vec![State::None; 256 * 192];
    let cells = vec![vec![State::None; 192]; 256];
    let mut simulation = Simulation {
        cells,
        width: 256,
        height: 192,
        growth_rate: 1e-2,
        lightning_rate: 1e-5,
        transmission_rate: 1.0,
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
        if rl.is_key_released(KeyboardKey::KEY_P) {
            paused = !paused
        }
        if rl.is_key_released(KeyboardKey::KEY_SPACE) {
            simulation.step();
        }

        if !paused {
            simulation.step();
        }
        let mut dh = rl.begin_drawing(&thread);
        simulation.render(&mut dh);
    }

    println!("Hello, world!");
}
