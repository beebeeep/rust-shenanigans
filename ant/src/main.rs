use sdl2::{self, event::Event, keyboard::Keycode, pixels::Color};
use std::{thread, time::Duration};
const WIDTH: usize = 800;
const HEIGHT: usize = 600;

#[derive(Clone)]
pub struct Dir(pub i32, pub i32);

pub static DIRECTIONS: &'static [Dir] = &[
    Dir(1, 0), // E
    // Dir(1, -1),  // NE
    Dir(0, -1), // N
    // Dir(-1, -1), // NW
    Dir(-1, 0), // W
    // Dir(-1, 1),  // SW
    Dir(0, 1), // S
               // Dir(1, 1),   // SE
];

struct Ant {
    x: usize,
    y: usize,
    d: usize,
}
impl Ant {
    fn step(&mut self) {
        (self.x, self.y) = (
            (self.x as i32 + DIRECTIONS[self.d].0).rem_euclid(WIDTH as i32) as usize,
            (self.y as i32 + DIRECTIONS[self.d].1).rem_euclid(HEIGHT as i32) as usize,
        );
    }
}

struct Sim {
    field: [[Color; HEIGHT]; WIDTH],
    ant: Ant,
}

impl Sim {
    fn new() -> Self {
        Self {
            field: [[Color::BLACK; HEIGHT]; WIDTH],
            ant: Ant {
                d: 0,
                x: WIDTH / 2,
                y: HEIGHT / 2,
            },
        }
    }

    fn tick(&mut self) {
        let f = &mut self.field[self.ant.x][self.ant.y];
        match *f {
            Color::WHITE => {
                *f = Color::BLACK;
                self.ant.d = (self.ant.d + 1) % DIRECTIONS.len();
                self.ant.step();
            }
            Color::BLACK => {
                *f = Color::WHITE;
                self.ant.d = (self.ant.d as i32 - 1).rem_euclid(DIRECTIONS.len() as i32) as usize;
                self.ant.step();
            }
            _ => {}
        }
    }
}

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let mut events = sdl_context.event_pump().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("wyrmas", WIDTH as u32, HEIGHT as u32)
        .position_centered()
        .build()
        .unwrap();
    let mut canvas = window.into_canvas().build().unwrap();
    canvas.clear();
    canvas.present();
    let mut sim = Sim::new();
    'run: loop {
        for event in events.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'run,
                _ => {}
            }
        }
        sim.tick();
        for x in 0..WIDTH as i32 {
            for y in 0..HEIGHT as i32 {
                canvas.set_draw_color(sim.field[x as usize][y as usize]);
                canvas.draw_point((x, y)).unwrap();
            }
        }
        canvas.present();
        // thread::sleep(Duration::from_millis(1));
    }
}
