use crate::{body::Body, vec2::Vec2, *};
use core::f32::consts::PI;
use sdl2::{
    gfx::primitives::DrawRenderer,
    keyboard::Keycode,
    pixels::Color,
    rect::Rect,
    render::{Canvas, TextureQuery},
    ttf,
    video::Window,
};

struct Player {
    pos: Vec2,
    speed: Vec2,
    dir: f32,
}

pub struct Game {
    tick: u64,
    pressed_keys: u32,
    player: Player,
    camera: Vec2, // position of bottom-left corner of viewport
    bodies: Vec<Body>,
    trajectory: Vec<Vec2>, // VecDeque will be more optimal here...
    cursor: Vec2,
    paused: bool,
}

impl Game {
    pub fn new() -> Self {
        Self {
            paused: false,
            tick: 0,
            pressed_keys: 0,
            cursor: Vec2 { x: 0.0, y: 0.0 },
            camera: Vec2 { x: 0.0, y: 0.0 },
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

    pub fn progress(&mut self) {
        if self.paused {
            return;
        }
        self.tick += 1;
        self.camera.x += 1.0;

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

        // calculate new player position
        (self.player.pos, self.player.speed) =
            self.progress_player(&self.player.pos, &self.player.speed);
        let (mut pos, mut speed) = (self.player.pos, self.player.speed);

        // calculate future trajectory
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

    pub fn render(&self, canvas: &mut Canvas<Window>, font: &ttf::Font) {
        canvas.set_draw_color(Color::RED);

        for body in &self.bodies {
            body.render(&self.camera, font, canvas);
        }
        // trajectory
        for i in 0..self.trajectory.len() - 1 {
            let (p1, p2) = (
                self.trajectory[i].to_screen(&self.camera),
                self.trajectory[i + 1].to_screen(&self.camera),
            );
            if p1.dist(p2) > 10.0 {
                continue;
            }
            canvas
                .line(
                    p1.x as i16,
                    p1.y as i16,
                    p2.x as i16,
                    p2.y as i16,
                    Color::GRAY,
                )
                .unwrap();
        }
        // player
        let d = Vec2::from(self.player.dir);
        let (p1, p2) = (
            self.player.pos.to_screen(&self.camera),
            (self.player.pos + d * 30.0).to_screen(&self.camera),
        );
        canvas
            .line(
                p1.x as i16,
                p1.y as i16,
                p2.x as i16,
                p2.y as i16,
                Color::GREEN,
            )
            .unwrap();
        canvas
            .filled_circle(p1.x as i16, p1.y as i16, 5, Color::RED)
            .unwrap();

        let c = self.cursor.to_screen(&self.camera);
        // cursor
        canvas
            .circle(c.x as i16, c.y as i16, 3, Color::GREY)
            .unwrap();

        self.show_debug(canvas, font);
    }

    pub fn key_down(&mut self, k: Keycode) {
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

    pub fn set_cursor(&mut self, x: i32, y: i32) {
        self.cursor.x = x as f32;
        self.cursor.y = (SZ_H - y) as f32; // convert from screen coords
    }

    pub fn key_up(&mut self, k: Keycode) {
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
