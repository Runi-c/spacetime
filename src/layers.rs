use bevy::{ecs::component::Component, render::view::RenderLayers};

trait LayersExt {
    const SPACE: RenderLayers;
    const FACTORY: RenderLayers;
}

impl LayersExt for RenderLayers {
    const SPACE: RenderLayers = RenderLayers::layer(0);
    const FACTORY: RenderLayers = RenderLayers::layer(1);
}

#[derive(Component)]
#[require(RenderLayers::SPACE)]
pub struct SpaceLayer;

#[derive(Component)]
#[require(RenderLayers::FACTORY)]
pub struct FactoryLayer;
