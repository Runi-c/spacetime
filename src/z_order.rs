use bevy::prelude::*;

use crate::scheduling::Sets;

pub fn plugin(app: &mut App) {
    app.add_systems(Update, apply_z_order.in_set(Sets::PostUpdate));
}

#[derive(Component, Clone)]
#[require(Transform)]
pub struct ZOrder(pub f32);
impl ZOrder {
    pub const BACKGROUND: Self = Self(0.0);
    pub const ASTEROID: Self = Self(5.0);
    pub const ENEMY: Self = Self(6.0);
    pub const PICKUP: Self = Self(7.0);
    pub const BULLET: Self = Self(8.0);
    pub const SHIP: Self = Self(10.0);

    pub const TILE: Self = Self(5.0);
    pub const PIPE: Self = Self(7.0);
    pub const MACHINE: Self = Self(10.0);
    pub const SHOP: Self = Self(20.0);
}

fn apply_z_order(mut query: Query<(&ZOrder, &mut Transform), Changed<ZOrder>>) {
    for (z_order, mut transform) in query.iter_mut() {
        transform.translation.z = z_order.0;
    }
}
