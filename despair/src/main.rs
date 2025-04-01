use lazy_static::lazy_static;
use sdl2::{
    controller::MappingStatus,
    event::Event,
    gfx::primitives::DrawRenderer,
    keyboard::Keycode,
    pixels::{self, Color},
    rect::Rect,
    render::Canvas,
    video::Window,
};
use std::f32;
use std::time::{Duration, Instant};
const SZ_W: i32 = 320;
const SZ_H: i32 = 240;
const SCALE: i32 = 4;
const PLAYER_SPEED: f32 = 10.0;
const PLAYER_ROT_SPEED: i32 = 1;
const FPS: u64 = 20;

lazy_static! {
    static ref COS: Vec<f32> = {
        (0..360)
            .map(|x| f32::cos((x as f32 / 180.0) * f32::consts::PI))
            .collect()
    };
    static ref SIN: Vec<f32> = {
        (0..360)
            .map(|x| f32::sin((x as f32 / 180.0) * f32::consts::PI))
            .collect()
    };
}

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

#[derive(Debug)]
struct Point {
    x: i32,
    y: i32,
    z: i32,
}

impl Point {
    fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    fn to_screen(&self, fov: i32) -> Self {
        Self {
            x: fov * self.x / self.y + (SZ_W / 2),
            y: fov * self.z / self.y + (SZ_H / 2),
            z: 0,
        }
    }

    fn to_world(&self, p: &Player) -> Self {
        Self {
            x: (self.x as f32 * COS[p.dir_h as usize] - self.y as f32 * SIN[p.dir_h as usize])
                as i32,
            y: (self.x as f32 * SIN[p.dir_h as usize] + self.y as f32 * COS[p.dir_h as usize])
                as i32,
            z: 0 - p.pos.z + (p.dir_v * self.y) / 32,
        }
    }
}

struct Player {
    pos: Point,
    dir_h: i32,
    dir_v: i32,
}

struct Game {
    tick: u64,
    player: Player,
}

impl Game {
    fn new() -> Self {
        Self {
            tick: 0,
            player: Player {
                pos: Point::new(70, -110, 0),
                dir_h: 0,
                dir_v: 0,
            },
        }
    }

    fn progress(&mut self) {
        self.tick += 1;
    }

    fn process_input(&mut self, input: KeyInput) {
        let (dx, dy) = (
            PLAYER_SPEED * SIN[self.player.dir_h as usize],
            PLAYER_SPEED * COS[self.player.dir_h as usize],
        );
        match input {
            KeyInput::Up => {
                self.player.pos.x += dx as i32;
                self.player.pos.y += dy as i32;
            }
            KeyInput::Down => {
                self.player.pos.x -= dx as i32;
                self.player.pos.y -= dy as i32;
            }
            KeyInput::Left => {
                self.player.pos.x -= dy as i32;
                self.player.pos.y -= dx as i32;
            }
            KeyInput::Right => {
                self.player.pos.x += dy as i32;
                self.player.pos.y += dx as i32;
            }
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
        println!(
            "player pos: {:?}, rot: hor {} vert {} {dx} {dy}",
            self.player.pos, self.player.dir_h, self.player.dir_v
        );
    }

    fn render(&self, canvas: &mut Canvas<Window>) {
        canvas.set_draw_color(Color::RED);
        let p1 = Point::new(40 - self.player.pos.x, 10 - self.player.pos.y, 0);
        let p2 = Point::new(40 - self.player.pos.x, 290 - self.player.pos.y, 0);
        let wp1 = p1.to_world(&self.player);
        let wp2 = p2.to_world(&self.player);
        let sp1 = wp1.to_screen(200);
        let sp2 = wp2.to_screen(200);
        if sp1.x > 0 && sp1.x < SZ_W && sp1.y > 0 && sp1.y < SZ_H {
            canvas
                .fill_rect(Rect::new(
                    SCALE * sp1.x,
                    SCALE * sp1.y,
                    SCALE as u32,
                    SCALE as u32,
                ))
                .unwrap();
        }
        // println!("world: {wp1:?}, {wp2:?}");
        // println!("screen: {sp1:?}, {sp2:?}");
        if sp2.x > 0 && sp2.x < SZ_W && sp2.y > 0 && sp2.y < SZ_H {
            canvas
                .fill_rect(Rect::new(
                    SCALE * sp2.x,
                    SCALE * sp2.y,
                    SCALE as u32,
                    SCALE as u32,
                ))
                .unwrap();
        }
    }
}

fn main() {
    let fps: Duration = Duration::from_millis(1000 / FPS);
    let sdl_context = sdl2::init().unwrap();
    let video = sdl_context.video().unwrap();
    let window = video
        .window("despair", (SZ_W * SCALE) as u32, (SZ_H * SCALE) as u32)
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
                Event::KeyUp {
                    keycode: Some(k), ..
                } => todo!(),
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
