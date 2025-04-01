use sdl2::{
    controller::MappingStatus,
    event::Event,
    gfx::primitives::DrawRenderer,
    keyboard::Keycode,
    pixels::{self, Color},
    rect::Rect,
    render::Canvas,
    sys::uint_least32_t,
    video::Window,
};
use std::time::{Duration, Instant};
const SZ_W: u32 = 320;
const SZ_H: u32 = 240;
const SCALE: u32 = 4;
const PLAYER_SPEED: f32 = 10.0;
const PLAYER_ROT_SPEED: i32 = 5;

enum KeyInput {
    Up,
    Down,
    Left,
    Right,
    TurnLeft,
    TurnRight,
    LookUp,
    LookDown,
}

struct Player {
    x: i32,
    y: i32,
    z: i32,
    dir_h: i32,
    dir_v: i32,
}

struct Game {
    tick: u64,
    player: Player,
    sin: Vec<f32>,
    cos: Vec<f32>,
}

impl Game {
    fn new() -> Self {
        Self {
            tick: 0,
            player: Player {
                x: 70,
                y: -110,
                z: 0,
                dir_h: 0,
                dir_v: 0,
            },
            sin: (0..360).map(|x| f32::sin(x as f32)).collect(),
            cos: (0..360).map(|x| f32::cos(x as f32)).collect(),
        }
    }

    fn progress(&mut self) {
        self.tick += 1;
    }

    fn process_input(&mut self, input: KeyInput) {
        let (dx, dy) = (
            self.sin[self.player.dir_h as usize],
            self.cos[self.player.dir_h as usize],
        );
        match input {
            KeyInput::Up => self.player.y += (PLAYER_SPEED * dy) as i32,
            KeyInput::Down => self.player.y -= (PLAYER_SPEED * dy) as i32,
            KeyInput::Left => self.player.x -= (PLAYER_SPEED * dx) as i32,
            KeyInput::Right => self.player.x += (PLAYER_SPEED * dx) as i32,
            KeyInput::TurnLeft => {
                self.player.dir_h = (self.player.dir_h + PLAYER_ROT_SPEED).rem_euclid(360)
            }
            KeyInput::TurnRight => {
                self.player.dir_h = (self.player.dir_h - PLAYER_ROT_SPEED).rem_euclid(360)
            }
            KeyInput::LookUp => {
                self.player.dir_v = (self.player.dir_v + PLAYER_ROT_SPEED).rem_euclid(360)
            }
            KeyInput::LookDown => {
                self.player.dir_v = (self.player.dir_v - PLAYER_ROT_SPEED).rem_euclid(360)
            }
        }
    }

    fn render(&self, canvas: &mut Canvas<Window>) {
        canvas.set_draw_color(Color::RED);
        canvas
            .fill_rect(Rect::new(
                (SCALE * (SZ_W / 2)) as i32,
                (SCALE * (self.tick % SZ_H as u64) as u32) as i32,
                SCALE,
                SCALE,
            ))
            .unwrap();
    }
}

fn main() {
    let fps: Duration = Duration::from_millis(1000 / 60);
    let sdl_context = sdl2::init().unwrap();
    let video = sdl_context.video().unwrap();
    let window = video
        .window("despair", SZ_W * SCALE, SZ_H * SCALE)
        .position_centered()
        .opengl()
        .build()
        .unwrap();
    let mut canvas = window.into_canvas().build().unwrap();
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut game = Game::new();

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::KeyDown {
                    keycode: Some(k), ..
                } => match k {
                    Keycode::W => game.process_input(KeyInput::Up),
                    Keycode::S => game.process_input(KeyInput::Down),
                    Keycode::A => game.process_input(KeyInput::Left),
                    Keycode::D => game.process_input(KeyInput::Right),
                    Keycode::H => game.process_input(KeyInput::TurnLeft),
                    Keycode::L => game.process_input(KeyInput::TurnRight),
                    Keycode::J => game.process_input(KeyInput::LookDown),
                    Keycode::K => game.process_input(KeyInput::LookUp),
                    _ => {}
                },
                _ => {}
            }
        }
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();

        let start = Instant::now();
        game.progress();
        game.render(&mut canvas);
        canvas.present();

        let elapsed = start - Instant::now();
        ::std::thread::sleep(fps - elapsed);
    }
}
