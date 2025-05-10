use bevy::{ecs::component::Component, render::view::RenderLayers};

trait LayersExt {
    const SPACE: RenderLayers;
    const FACTORY: RenderLayers;
}

impl LayersExt for RenderLayers {
    const SPACE: RenderLayers = RenderLayers::layer(0);
    const FACTORY: RenderLayers = RenderLayers::layer(1);
}

#[derive(Component, Clone)]
#[require(RenderLayers::SPACE)]
pub struct SpaceLayer;

#[derive(Component, Clone)]
#[require(RenderLayers::FACTORY)]
pub struct FactoryLayer;
