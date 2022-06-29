use glam::Vec3;

use crate::material::Material;

pub trait Shape: std::fmt::Debug
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
    pub radius: f32,
    pub material: Material
}

#[derive(Debug)]
pub struct Plane {
    pub pos: Vec3,
    pub normal: Vec3,
    pub material: Material
}

#[derive(Debug)]
pub struct Triangle {
    pub p1: Vec3,
    pub p2: Vec3,
    pub p3: Vec3,
    pub material: Material,

    pub normal: Vec3,
    pub edge1: Vec3,
    pub edge2: Vec3
}

#[derive(Debug)]
pub struct Ray {
    pub start: Vec3,
    pub dir: Vec3
}

impl Rect {
    pub fn infinite() -> Self {
        Rect {
            min: Vec3::splat(f32::NEG_INFINITY),
            max: Vec3::splat(f32::INFINITY)
        }
    }
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
            min: self.pos - self.radius,
            max: self.pos + self.radius
        }
    }
}

impl Shape for Plane {
    fn position(&self) -> Vec3 {
        self.pos
    }

    fn bounding_box(&self) -> Rect {
        if self.normal.y.abs() == 1.0 {
            Rect {
                min: Vec3::new( f32::NEG_INFINITY, self.pos.y, f32::NEG_INFINITY ),
                max: Vec3::new( f32::INFINITY, self.pos.y, f32::INFINITY )
            }
        }
        else{
            Rect::infinite()
        }
    }
}

impl Shape for Triangle {
    fn position(&self) -> Vec3 {
        (self.p1 + self.p2 + self.p3) / 3.0
    }

    fn bounding_box(&self) -> Rect {
        Rect {
            min: self.p1.min(self.p2.min(self.p3)),
            max: self.p1.max(self.p2.max(self.p3))
        }
    }
}

impl Triangle {
    pub fn new(p1: Vec3, p2: Vec3, p3: Vec3, material: Material) -> Self {
        Triangle { p1, p2, p3, material, normal: Vec3::ZERO, edge1: Vec3::ZERO, edge2: Vec3::ZERO}
            .precompute()
    }

    fn precompute(mut self) -> Self {
        self.edge1 = self.p2 - self.p1;
        self.edge2 = self.p3 - self.p1;
        self.normal = -self.edge1.cross(self.edge2).normalize();

        self
    }
}
