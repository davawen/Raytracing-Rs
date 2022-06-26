use glam::Vec3;

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
    pub radius: f32
}

#[derive(Debug)]
pub struct Plane {
    pub pos: Vec3,
    pub normal: Vec3
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

impl Shape for Plane {
    fn position(&self) -> Vec3 {
        self.pos
    }

    fn bounding_box(&self) -> Rect {
        if self.normal.y.abs() == 1.0 {
            Rect {
                min: Vec3::new( f32::NEG_INFINITY, self.pos.y, f32::NEG_INFINITY ),
                max: Vec3::new( f32::INFINITY, 0.0, f32::INFINITY )
            }
        }
        else{
            Rect {
                min: Vec3::splat(f32::NEG_INFINITY),
                max: Vec3::splat(f32::INFINITY)
            }
        }
    }
}
