use std::{rc::{Rc, Weak}, ops::Sub, cmp::Ordering};

use rand::{thread_rng, Rng};

use crate::{shape::*, intersection::{Inter, Intersection, Traceable}};
use crate::canvas::{ Canvas, Drawable, Pixel };

#[derive(Debug)]
pub struct Bvh<'a> {
    pub lhs: Option<Box<Bvh<'a>>>,
    pub rhs: Option<Box<Bvh<'a>>>,
    pub bound: Rect,
    pub shape: Option<&'a dyn Traceable>
}

impl<'a> Bvh<'a> {
    pub fn new(bound: Rect, shape: &'a dyn Traceable) -> Self {
        Bvh {
            lhs: None,
            rhs: None,
            bound,
            shape: Some(shape)
        }
    }

    fn from_child(lhs: Box<Bvh<'a>>, rhs: Box<Bvh<'a>>) -> Self {
        let mut bound = lhs.bound;
        if rhs.bound.min.x < bound.min.x { bound.min.x = rhs.bound.min.x }
        if rhs.bound.min.y < bound.min.y { bound.min.y = rhs.bound.min.y }
        if rhs.bound.max.x > bound.max.x { bound.max.x = rhs.bound.max.x }
        if rhs.bound.max.y > bound.max.y { bound.max.y = rhs.bound.max.y }
        
        Bvh {
            lhs: Some(lhs),
            rhs: Some(rhs),
            bound,
            shape: None
        }
    }

    pub fn construct<'b>(shapes: &'b mut [&'a dyn Traceable], dim: usize) -> Bvh<'a> {
        if shapes.is_empty() { panic!("Empty vector"); }
        else if shapes.len() == 1 {
            let shape = *shapes.first().unwrap();
            Bvh::new(shape.bounding_box(), shape)
        }
        else {
            shapes.sort_by(|a, b| {
                if dim % 2 == 0 {
                    a.position().x.partial_cmp(&b.position().x).unwrap()
                }
                else {
                    a.position().y.partial_cmp(&b.position().y).unwrap()
                }
            });

            let ( left, right ) = shapes.split_at_mut( shapes.len() / 2 );

            Bvh::from_child(
                Box::new(Bvh::construct(left, dim + 1)),
                Box::new(Bvh::construct(right, dim + 1))
            )
        }
    }

    pub fn intersects(&self, ray: &Ray) -> Option<Inter<dyn Traceable>> {
        if !self.bound.intersects(ray) { return None; }

        if let Some(shape) = self.shape {
            shape.ray_intersection(ray)
        }
        else {
            self.lhs.as_ref().unwrap().intersects(ray).or(
                self.rhs.as_ref().unwrap().intersects(ray)
            )
        }
    }
}

impl Drawable for Bvh<'_> {
    fn draw(&self, canvas: &mut Canvas, color: Pixel) {
        canvas.draw_outline(&self.bound, Pixel::RED);

        canvas.draw(&self.bound, color / 10);

        if let Some(lhs) = &self.lhs {
            canvas.draw(lhs.as_ref(), thread_rng().gen());
        }
        if let Some(rhs) = &self.rhs {
            canvas.draw(rhs.as_ref(), thread_rng().gen());
        }
    }
}