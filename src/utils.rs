use bevy::prelude::*;
use parry2d::na::Point2;

pub fn parry2d_vec2(v: Point2<f32>) -> Vec2 {
    Vec2::new(v.x, v.y)
}
