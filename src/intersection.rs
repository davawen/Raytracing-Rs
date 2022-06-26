use glam::Vec3;

use crate::shape::*;

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
        let inv = 1.0 / ray.dir;

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

impl Intersection<Ray> for Sphere {
    fn intersects(&self, ray: &Ray) -> bool {
        let to_center = self.pos - ray.start;
        let closest = to_center.project_onto(ray.dir);

        closest.distance_squared(self.pos) <= self.radius*self.radius
    }
}

#[derive(Debug, Clone)]
pub struct Inter<'a,T: ?Sized> {
    pub point: Vec3,
    pub normal: Vec3,
    pub shape: &'a T
}

pub trait Traceable
where Self: Shape {
    fn ray_intersection(&self, ray: &Ray) -> Option<Inter<dyn Traceable>>;
}

impl Traceable for Sphere {
    fn ray_intersection(&self, ray: &Ray) -> Option<Inter<dyn Traceable>> {
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

        Some(Inter {
            point,
            normal: (point - self.pos).normalize(),
            shape: self
        })
    }
}

impl Traceable for Plane {
    fn ray_intersection(&self, ray: &Ray) -> Option<Inter<dyn Traceable>> {
        let denom = self.normal.dot(ray.dir);
        if denom > f32::EPSILON {
            let dist = self.pos - ray.start;

            let t = dist.dot(self.normal) / denom;

            if t >= 0.0 {
                Some( Inter {
                    point: ray.start + ray.dir * t,
                    normal: self.normal,
                    shape: self
                } )
            }
            else {
                None
            }
        }
        else {
            None
        }
    }
}
