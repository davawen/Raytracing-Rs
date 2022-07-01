use glam::Vec3;

pub trait Reflect {
    /// Reflects a vector along a normal
    fn reflect(self, normal: Self) -> Self;
}

impl Reflect for Vec3 {
    fn reflect(self, normal: Self) -> Self {
        self - 2.0*self.dot(normal)*normal
    }
}
