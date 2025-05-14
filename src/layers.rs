use bevy::{ecs::component::Component, render::view::RenderLayers};

#[derive(Component, Clone)]
#[require(RenderLayers::layer(0))]
pub struct SpaceLayer;

#[derive(Component, Clone)]
#[require(RenderLayers::layer(1))]
pub struct FactoryLayer;

#[derive(Component, Clone)]
#[require(RenderLayers::layer(2))]
pub struct UILayer;
