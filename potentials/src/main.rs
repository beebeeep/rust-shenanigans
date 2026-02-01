use std::time::{Duration, Instant};

use raylib::prelude::*;

const W: i32 = 1024;
const H: i32 = 768;
const G: f32 = 10.0;

struct Body {
    mass: f32,
    pos: Vector2,
    speed: Vector2,
    r: f32,
}

struct Field {
    field: Vec<Vector2>,
    last_recalculation: Instant,
}

impl Body {
    fn render(&self, dh: &mut RaylibDrawHandle) {
        dh.draw_circle(self.pos.x as i32, self.pos.y as i32, self.r, Color::WHITE);
    }
}

impl Field {
    fn update(&mut self, bodies: &mut Vec<Body>, frame_time: f32) -> Option<(f32, f32)> {
        let (mut min, mut max) = (f32::MAX, f32::MIN);
        for body in bodies.iter_mut() {
            let (x, y) = (body.pos.x as usize, body.pos.y as usize);
            if x < W as usize && y < H as usize {
                let f = self.field[x + y * (W as usize)];
                body.speed += f * frame_time;
            }
            body.pos += body.speed * frame_time;
        }

        if self.last_recalculation.elapsed().as_millis() < 1000 {
            return None;
        }
        for x in 0..W {
            for y in 0..H {
                let f = &mut self.field[(x + y * W) as usize];
                f.x = 0.0;
                f.y = 0.0;
                for body in bodies.iter() {
                    let p = Vector2 {
                        x: x as f32,
                        y: y as f32,
                    } - body.pos;
                    let d = p.length();
                    if d < body.r {
                        // do not calculate potential inside bodies
                        continue;
                    }
                    let g = p * G * body.mass / d.powi(3);
                    let l = p.length();
                    if l < min {
                        min = l
                    }
                    if l > max {
                        max = l
                    }
                    *f = *f - g;
                }
            }
        }
        self.last_recalculation = Instant::now();
        Some((min, max))
    }
}

fn main() {
    let mut bodies = vec![
        Body {
            mass: 500.0,
            pos: Vector2 { x: 350.0, y: 350.0 },
            speed: Vector2 { x: 0.0, y: 0.0 },
            r: 20.0,
        },
        Body {
            mass: 1.0,
            pos: Vector2 { x: 400.0, y: 400.0 },
            speed: Vector2 { x: -10.0, y: 0.0 },
            r: 2.0,
        },
    ];
    let mut field = Field {
        field: vec![Vector2 { x: 0.0, y: 0.0 }; (W * H) as usize],
        last_recalculation: Instant::now().checked_sub(Duration::from_secs(10)).unwrap(),
    };

    let (mut rl, thread) = raylib::init().size(W, H).title("Potentials").build();
    rl.set_target_fps(60);

    let (mut lmin, mut lmax) = (1.0, 1.0);
    while !rl.window_should_close() {
        let mut dh = rl.begin_drawing(&thread);

        if let Some((min, max)) = field.update(&mut bodies, dh.get_frame_time()) {
            (lmin, lmax) = (min.log2(), max.log2());
        }

        let t = Instant::now();
        for x in 0..W {
            for y in 0..H {
                //   println!(
                //     "{x} {y} -> {:?}",
                //     field[(x + y * W) as usize].length() * scale
                // );
                let r = 256.0 * (field.field[(x + y * W) as usize].length() - lmin) / (lmax - lmin);
                dh.draw_pixel(
                    x,
                    y,
                    Color {
                        r: r as u8,
                        g: 0,
                        b: 0,
                        a: 0xff,
                    },
                );
            }
        }
        for b in &bodies {
            b.render(&mut dh);
        }
        println!("{}", t.elapsed().as_millis());
    }
}
