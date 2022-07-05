use glam::{Vec3, Vec2};

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

impl Rect {
    pub fn infinite() -> Self {
        Rect {
            min: Vec3::splat(f32::NEG_INFINITY),
            max: Vec3::splat(f32::INFINITY)
        }
    }

    pub fn order_components(mut self) -> Self {
        let this = self.clone();

        self.min = this.min.min(this.max);
        self.max = this.max.max(this.min);

        self
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

#[derive(Debug)]
pub struct Sphere<'a> {
    pub pos: Vec3,
    pub radius: f32,
    pub material: Material<'a>
}

impl Shape for Sphere<'_> {
    fn position(&self) -> Vec3 {
        self.pos
    }

    fn bounding_box(&self) -> Rect {
        Rect {
            min: self.pos - self.radius,
            max: self.pos + self.radius
        }.order_components()
    }
}

#[derive(Debug)]
pub struct Plane<'a> {
    pub pos: Vec3,
    pub normal: Vec3,
    pub material: Material<'a>
}

impl Shape for Plane<'_> {
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

#[derive(Debug, Clone, Copy, Default)]
pub struct Vertex {
    pub pos: Vec3,
    pub normal: Vec3,
    pub tex: Vec2
}

#[derive(Debug)]
pub struct Triangle<'a> {
    pub p0: Vertex,
    pub p1: Vertex,
    pub p2: Vertex,
    pub material: Material<'a>,

    pub normal: Vec3,
    pub edge1: Vec3,
    pub edge2: Vec3,
    pub edge3: Vec3
}

impl Shape for Triangle<'_> {
    fn position(&self) -> Vec3 {
        (self.p0.pos + self.p1.pos + self.p2.pos) / 3.0
    }

    fn bounding_box(&self) -> Rect {
        Rect {
            min: self.p0.pos.min(self.p1.pos.min(self.p2.pos)),
            max: self.p0.pos.max(self.p1.pos.max(self.p2.pos))
        }
    }
}

impl<'a> Triangle<'a> {
    pub fn new(p0: Vertex, p1: Vertex, p2: Vertex, material: Material<'a>) -> Self {
        Triangle { p0, p1, p2, material, normal: Vec3::ZERO, edge1: Vec3::ZERO, edge2: Vec3::ZERO, edge3: Vec3::ZERO}
            .precompute()
    }

    fn precompute(mut self) -> Self {
        self.edge1 = self.p1.pos - self.p0.pos;
        self.edge2 = self.p2.pos - self.p0.pos;
        self.edge3 = self.p2.pos - self.p1.pos;
        self.normal = -self.edge1.cross(self.edge2).normalize();

        self
    }

    pub fn barycentric_weigths(&self, p: Vec3) -> (f32, f32, f32) {
        // v1 - v2 -> -edge1
        // v1 - v3 -> -edge2
        // v2 - v3 -> -edge3
        // v3 - v2 -> edge3
        // v3 - v1 -> edge2

        let div = -self.edge3.y * (-self.edge2.x) + self.edge3.x * (-self.edge2.y);

        let w0 = ( -self.edge3.y * (p.x - self.p2.pos.x) + self.edge3.x * (p.y - self.p2.pos.y) ) / div;
        let w1 = ( self.edge2.y * (p.x - self.p2.pos.x) + -self.edge2.x * (p.y - self.p2.pos.y)) / div;
        let w2 = 1.0 - w0 - w1;

        (w0, w1, w2)
    }
}

#[derive(Debug)]
pub struct Ray {
    pub start: Vec3,
    pub dir: Vec3
}

impl Ray {
    /// Offset ray start slightly in its direction
    pub fn offset(mut self) -> Self {
        self.start += self.dir * 0.01;

        self
    }
}

