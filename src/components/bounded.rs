use amethyst::ecs::{Component, DenseVecStorage};
use amethyst::core::math::{Vector3, Point2};

use ncollide3d::shape::Cuboid;

#[derive(Component)]
#[storage(DenseVecStorage)]
pub struct Bounded {
    pub bounds: Cuboid<f32>,
}

impl Bounded {
    pub fn new(width: f32, height: f32) -> Bounded {
        Bounded{bounds: Cuboid::new(Vector3::new(width/2., height/2., 0.0))}
    }
}
