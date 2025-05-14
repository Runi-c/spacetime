use bevy::{
    asset::weak_handle,
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef, ShaderType},
    sprite::{Material2d, Material2dPlugin},
};
use bitflags::bitflags;

pub const SOLID_WHITE: Handle<ColorMaterial> = weak_handle!("10056343-43eb-4015-8e71-7a8f0b082d02");
pub const SOLID_BLACK: Handle<ColorMaterial> = weak_handle!("03c8157f-caa4-4237-b037-599ed86834ba");

pub(super) fn plugin(app: &mut App) {
    let mut materials = app.world_mut().resource_mut::<Assets<ColorMaterial>>();
    materials.insert(SOLID_WHITE.id(), ColorMaterial::from(Color::WHITE));
    materials.insert(SOLID_BLACK.id(), ColorMaterial::from(Color::BLACK));
    app.add_plugins(Material2dPlugin::<DitherMaterial>::default());
}

bitflags! {
    #[repr(transparent)]
    #[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
    pub struct DitherFlags: u32 {
        const NONE = 0;
        const WORLDSPACE = 1 << 0;
        const BAYER = 1 << 1;
        const FBM = 1 << 2;
        const BLUENOISE = 1 << 3;
        const TRANSPARENT = 1 << 4;
    }
}

#[derive(ShaderType, Debug, Clone)]
pub struct Dither {
    pub offset: Vec2,
    pub fill: f32,
    pub flags: u32,
    pub scale: f32,
    pub padding: Vec3,
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Default, Clone)]
pub struct DitherMaterial {
    #[uniform(0)]
    pub settings: Dither,
}

impl From<Dither> for DitherMaterial {
    fn from(settings: Dither) -> Self {
        Self { settings }
    }
}

pub struct RockyDither {
    pub fill: f32,
    pub scale: f32,
}
impl From<RockyDither> for DitherMaterial {
    fn from(settings: RockyDither) -> Self {
        Self {
            settings: Dither {
                fill: settings.fill,
                scale: settings.scale,
                flags: DitherFlags::FBM.bits(),
                offset: vec2(rand::random(), rand::random()),
                ..default()
            },
        }
    }
}

pub struct GassyDither {
    pub fill: f32,
    pub scale: f32,
}
impl From<GassyDither> for DitherMaterial {
    fn from(settings: GassyDither) -> Self {
        Self {
            settings: Dither {
                fill: settings.fill,
                scale: settings.scale,
                ..default()
            },
        }
    }
}

pub struct MetalDither {
    pub fill: f32,
    pub scale: f32,
}
impl From<MetalDither> for DitherMaterial {
    fn from(settings: MetalDither) -> Self {
        Self {
            settings: Dither {
                fill: settings.fill,
                scale: settings.scale,
                flags: DitherFlags::BAYER.bits(),
                ..default()
            },
        }
    }
}

impl Default for Dither {
    fn default() -> Self {
        Self {
            offset: Vec2::ZERO,
            fill: 1.0,
            flags: 0,
            scale: 1.0,
            padding: Vec3::ZERO,
        }
    }
}

const SHADER_ASSET_PATH: &str = "shaders/dither.wgsl";
impl Material2d for DitherMaterial {
    fn fragment_shader() -> ShaderRef {
        SHADER_ASSET_PATH.into()
    }
}
