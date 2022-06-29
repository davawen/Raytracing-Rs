use std::ops::Mul;

use crate::canvas::Pixel;
use derive_more::{ Add, AddAssign, Mul, MulAssign, Sub, SubAssign, Div, DivAssign };
use glam::Vec3;
use lerp::Lerp;
use num::Float;

#[derive(Debug, Clone, Copy, Add, AddAssign, Mul, MulAssign, Sub, SubAssign, Div, DivAssign)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32
}

impl Color {
    pub const WHITE: Color = Color::splat(1.0);
    pub const GRAY: Color = Color::splat(0.5);
    pub const BLACK: Color = Color::splat(0.0);
    pub const RED: Color = Color::new(1.0, 0.0, 0.0);
    pub const GREEN: Color = Color::new(0.0, 1.0, 0.0);
    pub const BLUE: Color = Color::new(0.0, 0.0, 1.0);
    pub const YELLOW: Color = Color::new(1.0, 1.0, 0.0);
    pub const PINK: Color = Color::new(1.0, 0.0, 1.0);
    pub const CYAN: Color = Color::new(0.0, 1.0, 1.0);

    pub const fn new(r: f32, g: f32, b: f32) -> Self {
        Color {
            r, g, b
        }
    }

    pub fn from_u8(r: u8, g: u8, b: u8) -> Self {
        Color {
            r: (r as f32) / 255.0,
            g: (g as f32) / 255.0,
            b: (b as f32) / 255.0
        }
    }

    pub const fn splat(c: f32) -> Self {
        Color::new(c, c, c)
    }

    pub fn splat_u8(c: u8) -> Self {
        Color::from_u8(c, c, c)
    }
}

impl From<Pixel> for Color {
    fn from(pixel: Pixel) -> Self {
        Color::from_u8(pixel.0, pixel.1, pixel.2)
    }
}

impl From<Vec3> for Color {
    fn from(vec: Vec3) -> Self {
        Color::new(vec.x, vec.y, vec.z)
    }
}

impl Into<Vec3> for Color {
    fn into(self) -> Vec3 {
        Vec3::new(self.r, self.g, self.b)
    }
}

impl Mul<Color> for Color {
    type Output = Color;

    fn mul(self, rhs: Color) -> Self::Output {
        Color {
            r: self.r * rhs.r,
            g: self.g * rhs.g,
            b: self.b * rhs.b
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Material {
    pub color: Color,
    pub reflectivity: f32
}

impl Default for Material {
    fn default() -> Self {
        Material { color: Color::WHITE, reflectivity: 0.0 }
    }
}
