use bevy::{ecs::system::SystemParam, prelude::*};
use rand::Rng;

use super::camera::SpaceCamera;

#[derive(SystemParam)]
pub struct ScreenBounds<'w, 's> {
    camera: Query<'w, 's, &'static Projection, With<SpaceCamera>>,
}

impl<'w, 's> ScreenBounds<'w, 's> {
    pub fn bounds(&self) -> Rect {
        if let Ok(projection) = self.camera.single() {
            if let Projection::Orthographic(orthographic) = projection {
                orthographic.area
            } else {
                panic!("Camera projection is not orthographic")
            }
        } else {
            panic!("No camera found")
        }
    }

    pub fn random_inside(&self) -> Vec2 {
        let bounds = self.bounds();
        Vec2::new(
            rand::thread_rng().gen_range(bounds.min.x..=bounds.max.x),
            rand::thread_rng().gen_range(bounds.min.y..=bounds.max.y),
        )
    }

    pub fn random_outside(&self) -> Vec2 {
        let bounds = self.bounds();
        match rand::thread_rng().gen_range(0..=3) {
            0 => Vec2::new(
                rand::thread_rng().gen_range(bounds.min.x..=bounds.max.x),
                bounds.min.y - 1.0,
            ),
            1 => Vec2::new(
                rand::thread_rng().gen_range(bounds.min.x..=bounds.max.x),
                bounds.max.y + 1.0,
            ),
            2 => Vec2::new(
                bounds.min.x - 1.0,
                rand::thread_rng().gen_range(bounds.min.y..=bounds.max.y),
            ),
            _ => Vec2::new(
                bounds.max.x + 1.0,
                rand::thread_rng().gen_range(bounds.min.y..=bounds.max.y),
            ),
        }
    }
}
