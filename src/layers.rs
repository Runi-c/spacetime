use bevy::render::view::RenderLayers;

pub struct Layers;
impl Layers {
    pub const SPACE: RenderLayers = RenderLayers::layer(0);
    pub const FACTORY: RenderLayers = RenderLayers::layer(1);
}
