use crate::vec2::*;
use crate::*;
use core::f32::consts::PI;
use sdl2::gfx::primitives::DrawRenderer;
use sdl2::pixels::Color;
use sdl2::render::Canvas;
use sdl2::ttf;
use sdl2::video::Window;

const BODY_DENSITY: f32 = 1.0;

pub(crate) struct Body {
    pub(crate) center: Vec2,
    pub(crate) points: Vec<Vec2>,
    pub(crate) color: Color,
    pub(crate) mass: f32,
}

impl Body {
    pub(crate) fn new(center: Vec2, size: f32, edges: usize) -> Self {
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

    pub(crate) fn render(&self, camera: &Vec2, font: &ttf::Font, canvas: &mut Canvas<Window>) {
        for i in 0..self.points.len() {
            let (p1, p2) = (
                (self.center + self.points[i]).to_screen(camera),
                (self.center + self.points[(i + 1) % self.points.len()]).to_screen(camera),
            );
            canvas
                .line(
                    p1.x as i16,
                    p1.y as i16,
                    p2.x as i16,
                    p2.y as i16,
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
