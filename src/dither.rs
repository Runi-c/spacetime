use bevy::{
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef},
    sprite::Material2d,
};

// This struct defines the data that will be passed to your shader
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone, Default)]
pub struct DitherMaterial {
    #[uniform(0)]
    pub fill: f32,
    #[uniform(1)]
    pub dither_type: u32,
    #[uniform(2)]
    pub dither_scale: f32,
}

/// The Material trait is very configurable, but comes with sensible defaults for all methods.
/// You only need to implement functions for features that need non-default behavior. See the Material api docs for details!
const SHADER_ASSET_PATH: &str = "shaders/dither.wgsl";
impl Material2d for DitherMaterial {
    fn fragment_shader() -> ShaderRef {
        SHADER_ASSET_PATH.into()
    }
}
