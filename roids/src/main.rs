use core::f32::consts::PI;
use rand;
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

mod vec2;
use vec2::Vec2;

const SZ_W: i32 = 1024;
const SZ_H: i32 = 768;
const PLAYER_SPEED: f32 = 0.5;
const PLAYER_ROT_SPEED: f32 = 0.1;
const FPS: u64 = 60;
const BODY_DENSITY: f32 = 1.0;
const GRAVITY: f32 = 0.1;

const K_FWD: u32 = 1;
const K_BACK: u32 = 2;
const K_LEFT: u32 = 4;
const K_RIGHT: u32 = 8;
const K_TURNLEFT: u32 = 16;
const K_TURNRIGHT: u32 = 32;

struct Body {
    center: Vec2,
    points: Vec<Vec2>,
    color: Color,
    mass: f32,
}

impl Body {
    fn new(center: Vec2, size: f32, edges: usize) -> Self {
        let mut points = Vec::with_capacity(edges);
        let mut area = 0.0;
        let d = 2.0 * PI / edges as f32;
        for i in 0..edges {
            let r_a = (rand::random::<f32>() - 0.5) * 0.5;
            let r_r = size * (rand::random::<f32>() - 0.5) * 0.9;
            let (s, c) = f32::sin_cos(i as f32 * d + d * r_a);
            points.push(Vec2 {
                x: (r_r + size) * c,
                y: (r_r + size) * s,
            })
        }
        for i in 0..points.len() {
            let (p1, p2) = (points[i], points[(i + 1) % points.len()]);
            area += 0.5 * (p1.x * p2.y - p2.x * p1.y).abs();
        }

        Self {
            center,
            points,
            mass: BODY_DENSITY * area,
            color: Color::YELLOW,
        }
    }

    fn render(&self, font: &ttf::Font, canvas: &mut Canvas<Window>) {
        for i in 0..self.points.len() {
            let (p1, p2) = (
                self.center + self.points[i],
                self.center + self.points[(i + 1) % self.points.len()],
            );
            canvas
                .line(
                    p1.x as i16,
                    SZ_H as i16 - p1.y as i16,
                    p2.x as i16,
                    SZ_H as i16 - p2.y as i16,
                    self.color,
                )
                .unwrap();
        }
        display_text(
            &format!("{:.1}", self.mass),
            font,
            self.color,
            self.center,
            canvas,
        );
    }
}

struct Player {
    pos: Vec2,
    speed: Vec2,
    dir: f32,
}

struct Game {
    tick: u64,
    pressed_keys: u32,
    player: Player,
    bodies: Vec<Body>,
    trajectory: Vec<Vec2>, // VecDeque will be more optimal here...
    cursor: Vec2,
    paused: bool,
}

impl Game {
    fn new() -> Self {
        Self {
            paused: false,
            tick: 0,
            pressed_keys: 0,
            cursor: Vec2 { x: 0.0, y: 0.0 },
            bodies: vec![Body::new(Vec2 { x: 400., y: 400. }, 50.0, 8)],
            trajectory: vec![Vec2 { x: 0.0, y: 0.0 }; 100],
            player: Player {
                pos: Vec2 {
                    x: (SZ_W / 2) as f32,
                    y: (SZ_H / 2) as f32,
                },
                dir: 0.0,
                speed: Vec2 { x: 0.0, y: 0.0 },
            },
        }
    }

    fn progress(&mut self) {
        if self.paused {
            return;
        }
        self.tick += 1;

        // rotate player towards cursor
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
        let speed = if dd.abs() > 1.0 {
            PLAYER_ROT_SPEED
        } else {
            dd.abs() * PLAYER_ROT_SPEED
        };
        self.player.dir -= speed * dd.signum();

        (self.player.pos, self.player.speed) =
            self.progress_player(&self.player.pos, &self.player.speed);
        let (mut pos, mut speed) = (self.player.pos, self.player.speed);
        for i in 0..self.trajectory.len() {
            (pos, speed) = self.progress_player(&pos, &speed);
            self.trajectory[i] = pos;
        }
    }

    fn progress_player(&self, p: &Vec2, s: &Vec2) -> (Vec2, Vec2) {
        let (mut pos, mut speed) = (*p, *s);
        // thrust
        let (dy, dx) = self.player.dir.sin_cos();
        if self.pressed_keys & K_FWD != 0 {
            speed.x += PLAYER_SPEED * dx;
            speed.y += PLAYER_SPEED * dy;
        }
        if self.pressed_keys & K_BACK != 0 {
            speed.x -= PLAYER_SPEED * dx;
            speed.y -= PLAYER_SPEED * dy;
        }

        //gravity
        for body in &self.bodies {
            let gravity =
                (body.center - pos) * (GRAVITY * body.mass / pos.dist(body.center).powi(3)); // 3rd power because of normalization of (body - pos)
            speed = speed + gravity;
        }

        pos.x += speed.x;
        pos.y += speed.y;
        if pos.x < 0. {
            pos.x = SZ_W as f32
        }
        if pos.x > SZ_W as f32 {
            pos.x = 0.
        }
        if pos.y < 0. {
            pos.y = SZ_H as f32
        }
        if pos.y > SZ_H as f32 {
            pos.y = 0.
        }

        (pos, speed)
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

        for body in &self.bodies {
            body.render(font, canvas);
        }
        // trajectory
        for i in 0..self.trajectory.len() - 1 {
            let (p1, p2) = (self.trajectory[i], self.trajectory[i + 1]);
            if p1.dist(p2) > 10.0 {
                continue;
            }
            canvas
                .line(
                    p1.x as i16,
                    SZ_H as i16 - p1.y as i16,
                    p2.x as i16,
                    SZ_H as i16 - p2.y as i16,
                    Color::GRAY,
                )
                .unwrap();
        }
        // player
        let (dy, dx) = self.player.dir.sin_cos();
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

        // cursor
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

fn display_text(
    text: &str,
    font: &ttf::Font,
    color: Color,
    center: Vec2,
    canvas: &mut Canvas<Window>,
) -> Rect {
    let texture_creator = canvas.texture_creator();
    let surf = font.render(text).blended(color).unwrap();
    let texture = texture_creator.create_texture_from_surface(surf).unwrap();
    let TextureQuery { width, height, .. } = texture.query();
    let r = Rect::new(
        center.x as i32 - width as i32 / 2,
        SZ_H - center.y as i32 - height as i32 / 2,
        width,
        height,
    );
    canvas.copy(&texture, None, r).unwrap();
    return r;
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
