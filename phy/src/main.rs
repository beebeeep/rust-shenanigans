use std::time::{Duration, Instant};

use sdl2::{
    event::Event,
    gfx::primitives::DrawRenderer,
    keyboard::Keycode,
    pixels::{self, Color},
    render::Canvas,
    video::Window,
};

const SIZE_X: u32 = 800;
const SIZE_Y: u32 = 600;

struct Vec2 {
    x: f32,
    y: f32,
}
struct Ball {
    pos: Vec2,
    speed: Vec2,
    r: f32,
    color: pixels::Color,
}

struct Simulation {
    balls: Vec<Ball>,
    gravity: Vec2,
}

impl Ball {
    fn render(self: &Self, canvas: &mut Canvas<Window>) -> Result<(), String> {
        canvas.filled_circle(
            self.pos.x as i16,
            self.pos.y as i16,
            self.r as i16,
            self.color,
        )
    }
}

impl Simulation {
    fn run(self: &mut Self) {
        for b in &mut self.balls {
            b.pos.x += b.speed.x;
            b.pos.y += b.speed.y;
            if b.pos.x < 0.0 {
                b.pos.x = 0.0;
                b.speed.x = -b.speed.x;
            }
            if b.pos.y < 0.0 {
                b.pos.y = 0.0;
                b.speed.y = -b.speed.y;
            }
            if b.pos.x > SIZE_X as f32 {
                b.pos.x = SIZE_X as f32;
                b.speed.x = -b.speed.x;
            }
            if b.pos.y > SIZE_Y as f32 {
                b.pos.y = SIZE_Y as f32;
                b.speed.y = -b.speed.y;
            }

            b.speed.x += self.gravity.x;
            b.speed.y += self.gravity.y;
        }
    }

    fn render(self: &Self, canvas: &mut Canvas<Window>) -> Result<(), String> {
        for b in &self.balls {
            b.render(canvas)?;
        }
        Ok(())
    }
}

fn main() {
    let fps: Duration = Duration::from_millis(1000 / 60);
    let sdl_context = sdl2::init().unwrap();
    let video = sdl_context.video().unwrap();
    let window = video
        .window("phy", SIZE_X, SIZE_Y)
        .position_centered()
        .opengl()
        .build()
        .unwrap();
    let mut canvas = window.into_canvas().build().unwrap();
    let mut simulation = Simulation {
        gravity: Vec2 { x: 0.0, y: 1.0 },
        balls: vec![Ball {
            pos: Vec2 { x: 400.0, y: 300.0 },
            speed: Vec2 {
                x: 20.0 * (rand::random::<f32>() - 0.5),
                y: 20.0 * (rand::random::<f32>() - 0.5),
            },
            r: 20.0,
            color: Color::RGB(0x80, 0, 0),
        }],
    };
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::KeyDown {
                    keycode: Some(Keycode::Down),
                    ..
                } => {}
                _ => {}
            }
        }
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();

        let start = Instant::now();
        simulation.run();
        simulation.render(&mut canvas).unwrap();
        canvas.present();

        let elapsed = start - Instant::now();
        ::std::thread::sleep(fps - elapsed);
    }
}
