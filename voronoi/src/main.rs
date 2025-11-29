use std::f64::consts::PI;

use rand::prelude::*;
use raylib::prelude::*;
use voronoice::{BoundingBox, Point, VoronoiBuilder};

const W: i32 = 1024;
const H: i32 = 768;
const SPD: f32 = 5.0;

struct Shard {
    edges: Vec<Vector2>,
    center: Vector2,
    site: Vector2,
    speed: Vector2,
    color: Color,
}

impl Shard {
    fn render(&self, dh: &mut RaylibDrawHandle) {
        dh.draw_triangle_fan(&self.edges, self.color);
        for i in 0..self.edges.len() {
            // outline
            dh.draw_line_ex(
                self.edges[i],
                self.edges[(i + 1) % self.edges.len()],
                3.0,
                Color::DARKRED,
            );
        }
        // dh.draw_spline_basis(&self.edges, 2.0, Color::WHITE);

        /*
        dh.draw_circle(
            self.center.x as i32,
            self.center.y as i32,
            3.0,
            Color::WHITE,
        );
        dh.draw_circle(self.site.x as i32, self.site.y as i32, 3.0, Color::RED);
        dh.draw_line(
            self.site.x as i32,
            self.site.y as i32,
            self.center.x as i32,
            self.center.y as i32,
            Color::LIGHTYELLOW,
        );
        */
    }
}

fn update(shards: &mut Vec<Shard>, t: f64) {
    for s in &mut *shards {
        let m = f32::sin(3.0 * t.rem_euclid(2.0 * PI) as f32);
        s.center += s.speed * m;
        if s.center.x < 0.0 {
            s.center.x = W as f32
        }
        if s.center.y < 0.0 {
            s.center.y = H as f32
        }
        if s.center.x > W as f32 {
            s.center.x = 0.0
        }
        if s.center.y > H as f32 {
            s.center.y = 0.0
        }
    }

    let voronoi = VoronoiBuilder::default()
        .set_sites(
            shards
                .iter()
                .map(|v| Point {
                    x: v.center.x as f64,
                    y: v.center.y as f64,
                })
                .collect(),
        )
        .set_bounding_box(BoundingBox::new(
            Point {
                x: (W / 2) as f64,
                y: (H / 2) as f64,
            },
            W as f64,
            H as f64,
        ))
        .set_lloyd_relaxation_iterations(1)
        .build()
        .expect("building shards");

    assert_eq!(shards.len(), voronoi.iter_cells().count());
    for (i, cell) in voronoi.iter_cells().enumerate() {
        shards[i].center.x = cell.site_position().x as f32;
        shards[i].center.y = cell.site_position().y as f32;
        shards[i].edges.truncate(0);
        shards[i].edges.extend(
            cell.iter_vertices()
                .map(|v| Vector2::new(v.x as f32, v.y as f32)),
        );
    }
}

fn main() {
    let mut rng = rand::rng();
    let mut shards = Vec::with_capacity(200);
    for _ in 0..shards.capacity() {
        shards.push(Shard {
            center: Vector2::new(
                rng.random_range(0.0..W as f64) as f32,
                rng.random_range(0.0..H as f64) as f32,
            ),
            site: Vector2::zero(),
            speed: Vector2::new(
                rng.random_range(-1.0..1.0) * SPD,
                rng.random_range(-1.0..1.0) * SPD,
            ),
            color: if rng.random() {
                Color::ORANGERED
            } else {
                Color::DARKORANGE
            },
            edges: Vec::new(),
        });
    }

    let (mut rl, thread) = raylib::init().size(W, H).title("Arkanoid").build();
    rl.set_target_fps(60);
    // rl.gui_lock();
    // rl.disable_cursor();
    while !rl.window_should_close() {
        let t = rl.get_time();
        let mut dh = rl.begin_drawing(&thread);
        update(&mut shards, t);
        for s in &shards {
            s.render(&mut dh);
        }
    }
}
