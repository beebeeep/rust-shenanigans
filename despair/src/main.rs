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
const PLAYER_SPEED: f32 = 4.0;
const PLAYER_ROT_SPEED: i32 = 1;
const FPS: u64 = 20;

const K_UP: u32 = 1;
const K_DOWN: u32 = 2;
const K_LEFT: u32 = 4;
const K_RIGHT: u32 = 8;
const K_TURNLEFT: u32 = 16;
const K_TURNRIGHT: u32 = 32;
const K_LOOKUP: u32 = 64;
const K_LOOKDOWN: u32 = 128;

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

#[derive(Debug)]
struct Point {
    x: f32,
    y: f32,
    z: f32,
}

impl Point {
    fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    fn to_screen(&self, fov: i32) -> Self {
        Self {
            x: fov as f32 * self.x / self.y + (SZ_W as f32 / 2.0),
            y: fov as f32 * self.z / self.y + (SZ_H as f32 / 2.0),
            z: 0.0,
        }
    }

    fn to_world(&self, p: &Player) -> Self {
        Self {
            x: (self.x as f32 * COS[p.dir_h as usize] - self.y as f32 * SIN[p.dir_h as usize]),
            y: (self.x as f32 * SIN[p.dir_h as usize] + self.y as f32 * COS[p.dir_h as usize]),
            z: 0.0 - p.pos.z + (p.dir_v as f32 * self.y) / 32.0,
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
    pressed_keys: u32,
    player: Player,
}

impl Game {
    fn new() -> Self {
        Self {
            tick: 0,
            pressed_keys: 0,
            player: Player {
                pos: Point::new(70.0, -110.0, 0.0),
                dir_h: 0,
                dir_v: 0,
            },
        }
    }

    fn progress(&mut self) {
        self.tick += 1;

        let (dx, dy) = (
            PLAYER_SPEED * SIN[self.player.dir_h as usize],
            PLAYER_SPEED * COS[self.player.dir_h as usize],
        );
        if self.pressed_keys & K_UP != 0 {
            self.player.pos.x += dx;
            self.player.pos.y += dy;
        }
        if self.pressed_keys & K_DOWN != 0 {
            self.player.pos.x -= dx;
            self.player.pos.y -= dy;
        }
        if self.pressed_keys & K_LEFT != 0 {
            self.player.pos.x -= dy;
            self.player.pos.x -= dx;
        }
        if self.pressed_keys & K_RIGHT != 0 {
            println!("right");
            self.player.pos.x += dy;
            self.player.pos.y += dx;
        }
        if self.pressed_keys & K_TURNLEFT != 0 {
            self.player.dir_h = (self.player.dir_h + PLAYER_ROT_SPEED).rem_euclid(360);
        }
        if self.pressed_keys & K_TURNRIGHT != 0 {
            self.player.dir_h = (self.player.dir_h - PLAYER_ROT_SPEED).rem_euclid(360);
        }
        if self.pressed_keys & K_LOOKUP != 0 {
            self.player.dir_v = (self.player.dir_v + PLAYER_ROT_SPEED).rem_euclid(360);
        }
        if self.pressed_keys & K_LOOKDOWN != 0 {
            self.player.dir_v = (self.player.dir_v - PLAYER_ROT_SPEED).rem_euclid(360);
        }
        println!("pl {} {:?}", self.pressed_keys, self.player.pos);
    }

    fn render(&self, canvas: &mut Canvas<Window>) {
        canvas.set_draw_color(Color::RED);
        let p1 = Point::new(40.0 - self.player.pos.x, 10.0 - self.player.pos.y, 0.);
        let p2 = Point::new(40.0 - self.player.pos.x, 290.0 - self.player.pos.y, 0.0);
        let wp1 = p1.to_world(&self.player);
        let wp2 = p2.to_world(&self.player);
        let sp1 = wp1.to_screen(200);
        let sp2 = wp2.to_screen(200);
        if sp1.x > 0.0 && sp1.x < SZ_W as f32 && sp1.y > 0.0 && sp1.y < SZ_H as f32 {
            canvas
                .fill_rect(Rect::new(
                    SCALE * sp1.x as i32,
                    SCALE * sp1.y as i32,
                    SCALE as u32,
                    SCALE as u32,
                ))
                .unwrap();
        }
        // println!("world: {wp1:?}, {wp2:?}");
        // println!("screen: {sp1:?}, {sp2:?}");
        if sp2.x > 0.0 && sp2.x < SZ_W as f32 && sp2.y > 0.0 && sp2.y < SZ_H as f32 {
            canvas
                .fill_rect(Rect::new(
                    SCALE * sp2.x as i32,
                    SCALE * sp2.y as i32,
                    SCALE as u32,
                    SCALE as u32,
                ))
                .unwrap();
        }
    }

    fn key_down(&mut self, k: Keycode) {
        match k {
            Keycode::W => self.pressed_keys |= K_UP,
            Keycode::S => self.pressed_keys |= K_DOWN,
            Keycode::A => self.pressed_keys |= K_LEFT,
            Keycode::D => self.pressed_keys |= K_RIGHT,
            Keycode::H => self.pressed_keys |= K_TURNLEFT,
            Keycode::L => self.pressed_keys |= K_TURNRIGHT,
            Keycode::J => self.pressed_keys |= K_LOOKDOWN,
            Keycode::K => self.pressed_keys |= K_LOOKUP,
            _ => {}
        }
    }

    fn key_up(&mut self, k: Keycode) {
        match k {
            Keycode::W => self.pressed_keys &= !K_UP,
            Keycode::S => self.pressed_keys &= !K_DOWN,
            Keycode::A => self.pressed_keys &= !K_LEFT,
            Keycode::D => self.pressed_keys &= !K_RIGHT,
            Keycode::H => self.pressed_keys &= !K_TURNLEFT,
            Keycode::L => self.pressed_keys &= !K_TURNRIGHT,
            Keycode::J => self.pressed_keys &= !K_LOOKDOWN,
            Keycode::K => self.pressed_keys &= !K_LOOKUP,
            _ => {}
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
                Event::KeyDown {
                    keycode: Some(k), ..
                } => game.key_down(k),
                Event::KeyUp {
                    keycode: Some(k), ..
                } => game.key_up(k),
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
