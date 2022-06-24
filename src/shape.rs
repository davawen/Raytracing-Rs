use glam::Vec3;

use crate::canvas::Drawable;

pub trait Shape: std::fmt::Debug + Drawable
{
    fn position(&self) -> Vec3;
    fn bounding_box(&self) -> Rect;
}

#[derive(Debug, Clone, Copy)]
pub struct Rect {
    pub min: Vec3,
    pub max: Vec3
}

#[derive(Debug)]
pub struct Sphere {
    pub pos: Vec3,
    pub radius: f32
}

#[derive(Debug)]
pub struct Ray {
    pub start: Vec3,
    pub dir: Vec3
}

impl Shape for Rect {
    fn position(&self) -> Vec3 {
        self.min
    }

    fn bounding_box(&self) -> Rect {
        *self
    }
}

impl Shape for Sphere {
    fn position(&self) -> Vec3 {
        self.pos
    }

    fn bounding_box(&self) -> Rect {
        Rect {
            min: self.pos - Vec3::splat(self.radius),
            max: self.pos + Vec3::splat(self.radius)
        }
    }
}
