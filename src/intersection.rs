use std::f32::consts::PI;

use glam::{Vec3, Vec2};
use num::Zero;

use crate::{shape::*, material::Material};

pub trait Intersection<T> where
    T: ?Sized,
{
    fn intersects(&self, other: &T) -> bool;
}

impl Intersection<Vec3> for Rect {
    fn intersects(&self, other: &Vec3) -> bool {
        other.x >= self.min.x &&
        other.y >= self.min.y &&
        other.z >= self.min.z &&
        other.x <= self.max.x &&
        other.y <= self.max.y &&
        other.z <= self.max.z
    }
}

impl Intersection<Rect> for Rect {
    fn intersects(&self, other: &Rect) -> bool {
        self.intersects(&other.min) ||
        self.intersects(&other.max)
    }
}

impl Intersection<Ray> for Rect {
    fn intersects(&self, ray: &Ray) -> bool {
        let inv = ray.dir.recip();

        let tx1 = (self.min.x - ray.start.x) * inv.x;
        let tx2 = (self.max.x - ray.start.x) * inv.x;

        let mut tmin = tx1.min(tx2);
        let mut tmax = tx1.max(tx2);

        let ty1 = (self.min.y - ray.start.y) * inv.y;
        let ty2 = (self.max.y - ray.start.y) * inv.y;

        tmin = tmin.max(ty1.min(ty2));
        tmax = tmax.min(ty1.max(ty2));
        
        let tz1 = (self.min.z - ray.start.z) * inv.z;
        let tz2 = (self.max.z - ray.start.z) * inv.z;

        tmin = tmin.max(tz1.min(tz2));
        tmax = tmax.min(tz1.max(tz2));

        tmax >= tmin.max(0.0)
    }
}

impl Intersection<Ray> for Sphere<'_> {
    fn intersects(&self, ray: &Ray) -> bool {
        let to_center = self.pos - ray.start;
        let closest = to_center.project_onto(ray.dir);

        closest.distance_squared(self.pos) <= self.radius*self.radius
    }
}

impl Intersection<Sphere<'_>> for Sphere<'_> {
    fn intersects(&self, other: &Sphere) -> bool {
        self.pos.distance_squared(other.pos) <= (self.radius+other.radius)*(self.radius+other.radius)
    }
}

#[derive(Debug, Clone)]
pub struct Inter<T> {
    pub point: Vec3,
    pub normal: Vec3,
    pub front: bool,
    pub shape: T
}

pub trait Traceable
where Self: Shape + std::marker::Sync {
    fn material(&self) -> &Material;
    fn ray_intersection(&self, ray: &Ray) -> Option<Inter<&dyn Traceable>>;

    /// Returns a texture coordinate according to a point on itself
    fn sample(&self, _p: Vec3) -> Vec2 {
        Vec2::ZERO
    }
}

impl<'a> Traceable for Sphere<'a> {
    fn material(&self) -> &Material {
        &self.material
    }

    fn ray_intersection(&self, ray: &Ray) -> Option<Inter<&dyn Traceable>> {
        let to_center = self.pos - ray.start;

        // Calculate coefficients a, b, c from quadratic equation

        // let a = ray.dir.dot(ray.dir); // Assume ray direction is normalised
        let b = to_center.dot(ray.dir);
        let c = to_center.dot(to_center) - self.radius*self.radius;
        let discriminant = b*b - c;
        
        if discriminant < 0.0 { return None }

        let discr_sqrt = discriminant.sqrt();
        let mut t = b - discr_sqrt;

        if t < 0.0 {
            t = b + discr_sqrt;
            if t < 0.0 { return None }
        }

        let point = ray.start + ray.dir * t;

        let dist = (point-self.pos).normalize();

        let sgn = self.radius.signum();

        // Make normal point inwards when ray start is inside sphere
        // Multiplying by the sign inverse's the comparison ( negative radius = inside-out sphere )
        let ( front, normal ) = if self.pos.distance_squared(ray.start)*sgn <= self.radius*self.radius*sgn {
            ( false, -dist )
        }
        else {
            ( true, dist )
        };

        Some(Inter {
            point,
            normal,
            front,
            shape: self
        })
    }

    fn sample(&self, p: Vec3) -> Vec2 {
        let dist = p - self.pos;

        let u = (-dist.x.atan2(dist.z) / (2.0*PI) + 0.5 + 0.3) % 1.0;
        let v = dist.y / self.radius / 2.0 + 0.5;

        Vec2::new(u, v)
    }
}

impl Traceable for Plane<'_> {
    fn material(&self) -> &Material {
        &self.material
    }

    fn ray_intersection(&self, ray: &Ray) -> Option<Inter<&dyn Traceable>> {
        let denom = self.normal.dot(ray.dir);

        if denom.is_zero() { return None }

        let dist = self.pos - ray.start;

        let t = self.normal.dot(dist) / denom;

        if t >= 0.0 {
            Some( Inter {
                point: ray.start + ray.dir * t,
                normal: self.normal,
                front: false,
                shape: self
            } )
        }
        else {
            None
        }
    }

    fn sample(&self, p: Vec3) -> Vec2 {
        Vec2::new( p.x, p.z )
    }
}

impl<'a> Traceable for Triangle<'a> {
    fn material(&self) -> &Material {
        &self.material
    }

    fn ray_intersection(&self, ray: &Ray) -> Option<Inter<&dyn Traceable>> {
        let h = ray.dir.cross(self.edge2);
        let a = self.edge1.dot(h);

        if a.is_zero() { return None } // Ray parallel to triangle

        let f = a.recip();
        let s = ray.start - self.p0.pos;
        let u = f * s.dot(h);

        if !(0.0..=1.0).contains(&u) { return None }

        let q = s.cross(self.edge1);
        let v = f * ray.dir.dot(q);

        if v < 0.0 || u + v > 1.0 { return None }

        // At this stage we can compute t to find out where the intersection point is on the line.
        let t = f * self.edge2.dot(q);

        let normal = if ray.dir.dot(self.normal) < 0.0 { self.normal } else { -self.normal };

        if t > 0.0 {
            Some(Inter {
                point: ray.start + ray.dir * t,
                normal,
                front: true,
                shape: self
            })
        }
        else {
            None
        }
    }

    fn sample(&self, p: Vec3) -> Vec2 {
        let (w0, w1, w2) = self.barycentric_weigths(p);

        let out = w0*self.p0.tex + w1*self.p1.tex + w2*self.p2.tex;

        Vec2::new( out.x, out.y )
    }
}
