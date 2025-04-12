use core::{f32::consts::PI, fmt};
use sdl2::{
    event::Event,
    gfx::primitives::DrawRenderer,
    keyboard::Keycode,
    pixels::Color,
    rect::Rect,
    render::{Canvas, TextureQuery},
    ttf,
    video::Window,
};
use std::time::{Duration, Instant};

const SZ_W: i32 = 1024;
const SZ_H: i32 = 768;
const PLAYER_SPEED: f32 = 0.5;
const PLAYER_ROT_SPEED: f32 = 0.1;
const FPS: u64 = 60;

const K_FWD: u32 = 1;
const K_BACK: u32 = 2;
const K_LEFT: u32 = 4;
const K_RIGHT: u32 = 8;
const K_TURNLEFT: u32 = 16;
const K_TURNRIGHT: u32 = 32;

struct Point {
    x: f32,
    y: f32,
}

struct Player {
    pos: Point,
    speed: Point,
    dir: f32,
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x as i32, self.y as i32)
    }
}

struct Game {
    tick: u64,
    pressed_keys: u32,
    player: Player,
    cursor: Point,
    paused: bool,
}

impl Game {
    fn new() -> Self {
        Self {
            paused: false,
            tick: 0,
            pressed_keys: 0,
            cursor: Point { x: 0.0, y: 0.0 },
            player: Player {
                pos: Point {
                    x: (SZ_W / 2) as f32,
                    y: (SZ_H / 2) as f32,
                },
                dir: 0.0,
                speed: Point { x: 0.0, y: 0.0 },
            },
        }
    }

    fn progress(&mut self) {
        if self.paused {
            return;
        }
        self.tick += 1;

        let (dx, dy) = (self.player.dir.cos(), self.player.dir.sin());
        let dir_cur = f32::atan2(
            self.cursor.y - self.player.pos.y,
            self.cursor.x - self.player.pos.x,
        );

        let mut dd = self.player.dir - dir_cur;
        if dd > PI {
            dd -= PI * 2.0
        }
        if dd < -PI {
            dd += PI * 2.0
        }
        if dd < 0.0 {
            self.player.dir += dd.abs() * PLAYER_ROT_SPEED;
        }
        if dd > 0.0 {
            self.player.dir -= dd.abs() * PLAYER_ROT_SPEED;
        }

        if self.pressed_keys & K_FWD != 0 {
            self.player.speed.x += PLAYER_SPEED * dx;
            self.player.speed.y += PLAYER_SPEED * dy;
        }
        if self.pressed_keys & K_BACK != 0 {
            self.player.speed.x -= PLAYER_SPEED * dx;
            self.player.speed.y -= PLAYER_SPEED * dy;
        }

        self.player.pos.x += self.player.speed.x;
        self.player.pos.y += self.player.speed.y;
        if self.player.pos.x < 0. {
            self.player.pos.x = SZ_W as f32
        }
        if self.player.pos.x > SZ_W as f32 {
            self.player.pos.x = 0.
        }
        if self.player.pos.y < 0. {
            self.player.pos.y = SZ_H as f32
        }
        if self.player.pos.y > SZ_H as f32 {
            self.player.pos.y = 0.
        }
    }

    fn show_debug(&self, canvas: &mut Canvas<Window>, font: &ttf::Font) {
        let dir_cur = f32::atan2(
            self.cursor.y - self.player.pos.y,
            self.cursor.x - self.player.pos.x,
        );
        let texture_creator = canvas.texture_creator();
        let surf = font
            .render(&format!(
                "player {}@{:.0} cursor {}@{:.0}",
                self.player.pos,
                self.player.dir * 180.0 / PI,
                self.cursor,
                dir_cur * 180.0 / PI
            ))
            .blended(Color::GREEN)
            .unwrap();
        let line1_tex = texture_creator.create_texture_from_surface(surf).unwrap();
        let surf = font
            .render(&format!(
                "cursor {}@{:.0}",
                self.cursor,
                dir_cur * 180.0 / PI
            ))
            .blended(Color::GREEN)
            .unwrap();
        let line2_tex = texture_creator.create_texture_from_surface(surf).unwrap();
        let TextureQuery {
            width: w1,
            height: h1,
            ..
        } = line1_tex.query();
        let TextureQuery {
            width: w2,
            height: h2,
            ..
        } = line2_tex.query();
        canvas
            .copy(&line1_tex, None, Rect::new(0, 0, w1, h1))
            .unwrap();
        canvas
            .copy(&line2_tex, None, Rect::new(0, h1 as i32, w2, h2))
            .unwrap();
    }

    fn render(&self, canvas: &mut Canvas<Window>, font: &ttf::Font) {
        canvas.set_draw_color(Color::RED);
        let (dx, dy) = (self.player.dir.cos(), self.player.dir.sin());
        canvas
            .line(
                self.player.pos.x as i16,
                SZ_H as i16 - self.player.pos.y as i16,
                self.player.pos.x as i16 + (dx * 30.0) as i16,
                SZ_H as i16 - (self.player.pos.y as i16 + (dy * 30.0) as i16),
                Color::GREEN,
            )
            .unwrap();
        canvas
            .filled_circle(
                self.player.pos.x as i16,
                SZ_H as i16 - self.player.pos.y as i16,
                5,
                Color::RED,
            )
            .unwrap();

        canvas
            .circle(
                self.cursor.x as i16,
                SZ_H as i16 - self.cursor.y as i16,
                3,
                Color::GREY,
            )
            .unwrap();

        self.show_debug(canvas, font);
    }

    fn key_down(&mut self, k: Keycode) {
        match k {
            Keycode::W => self.pressed_keys |= K_FWD,
            Keycode::S => self.pressed_keys |= K_BACK,
            Keycode::A => self.pressed_keys |= K_LEFT,
            Keycode::D => self.pressed_keys |= K_RIGHT,
            Keycode::H => self.pressed_keys |= K_TURNLEFT,
            Keycode::L => self.pressed_keys |= K_TURNRIGHT,
            Keycode::P => self.paused = !self.paused,
            Keycode::SPACE => {
                self.player.speed.x = 0.;
                self.player.speed.y = 0.;
            }
            _ => {}
        }
    }

    fn set_cursor(&mut self, x: i32, y: i32) {
        self.cursor.x = x as f32;
        self.cursor.y = (SZ_H - y) as f32; // convert from screen coords
    }

    fn key_up(&mut self, k: Keycode) {
        match k {
            Keycode::W => self.pressed_keys &= !K_FWD,
            Keycode::S => self.pressed_keys &= !K_BACK,
            Keycode::A => self.pressed_keys &= !K_LEFT,
            Keycode::D => self.pressed_keys &= !K_RIGHT,
            Keycode::H => self.pressed_keys &= !K_TURNLEFT,
            Keycode::L => self.pressed_keys &= !K_TURNRIGHT,
            _ => {}
        }
    }
}

fn main() {
    let fps: Duration = Duration::from_millis(1000 / FPS);
    let sdl_context = sdl2::init().unwrap();
    let video = sdl_context.video().unwrap();
    sdl_context.mouse().show_cursor(false);
    let window = video
        .window("roids", SZ_W as u32, SZ_H as u32)
        .position_centered()
        .input_grabbed()
        .vulkan()
        .build()
        .unwrap();
    let mut canvas = window.into_canvas().build().unwrap();
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump().unwrap();
    let ttf_context = ttf::init().unwrap();
    let mut font = ttf_context.load_font("font.ttf", 12).unwrap();
    font.set_style(ttf::FontStyle::BOLD);

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
                Event::MouseButtonDown {
                    timestamp,
                    window_id,
                    which,
                    mouse_btn,
                    clicks,
                    x,
                    y,
                } => {
                    println!(
                        "btn down {timestamp} {window_id} {which} {mouse_btn:?}  {clicks} {x} {y}"
                    );
                }
                Event::MouseButtonUp {
                    timestamp,
                    window_id,
                    which,
                    mouse_btn,
                    clicks,
                    x,
                    y,
                } => {
                    println!(
                        "btn up {timestamp} {window_id} {which} {mouse_btn:?}  {clicks} {x} {y}"
                    );
                }
                Event::MouseMotion { x, y, .. } => game.set_cursor(x, y),
                _ => {}
            }
        }
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();

        let start = Instant::now();
        game.progress();
        game.render(&mut canvas, &font);
        canvas.present();

        let elapsed = start - Instant::now();
        ::std::thread::sleep(fps - elapsed);
    }
}
