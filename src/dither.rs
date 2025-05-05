use bevy::{
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef},
    sprite::{Material2d, Material2dPlugin},
};

pub fn plugin(app: &mut App) {
    app.add_plugins(Material2dPlugin::<DitherMaterial>::default());
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone, Default)]
pub struct DitherMaterial {
    #[uniform(0)]
    pub fill: f32,
    #[uniform(1)]
    pub dither_type: u32,
    #[uniform(2)]
    pub dither_scale: f32,
}

const SHADER_ASSET_PATH: &str = "shaders/dither.wgsl";
impl Material2d for DitherMaterial {
    fn fragment_shader() -> ShaderRef {
        SHADER_ASSET_PATH.into()
    }
}
