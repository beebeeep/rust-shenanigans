use crate::*;
use std::fmt;

#[derive(Clone, Copy)]
pub(crate) struct Vec2 {
    pub(crate) x: f32,
    pub(crate) y: f32,
}

impl std::ops::Add for Vec2 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl std::ops::Sub for Vec2 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl std::ops::Mul<f32> for Vec2 {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl Vec2 {
    pub(crate) fn dist(&self, p: Self) -> f32 {
        f32::sqrt((self.x - p.x).powi(2) + (self.y - p.y).powi(2))
    }

    pub(crate) fn to_screen(&self, camera: &Self) -> Self {
        Self {
            x: self.x - camera.x,
            y: SZ_H as f32 - (self.y - camera.y),
        }
    }
}

impl From<(f32, f32)> for Vec2 {
    fn from(p: (f32, f32)) -> Self {
        Self { x: p.0, y: p.1 }
    }
}

impl From<f32> for Vec2 {
    /// interpret v as angle in radians, return relevant unit vector
    fn from(v: f32) -> Self {
        Self {
            x: v.cos(),
            y: v.sin(),
        }
    }
}

impl fmt::Display for Vec2 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x as i32, self.y as i32)
    }
}
