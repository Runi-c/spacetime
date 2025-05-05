use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::mesh::{Indices, PrimitiveTopology},
};
use lyon_tessellation::{
    self,
    geom::euclid::{Point2D, UnknownUnit},
    geometry_builder::simple_builder,
    path::{builder::NoAttributes, Polygon},
    FillBuilder, FillOptions, FillTessellator, StrokeBuilder, StrokeOptions, StrokeTessellator,
    VertexBuffers,
};

pub trait MeshLyonExtensions {
    fn fill_with(func: impl FnOnce(&mut NoAttributes<FillBuilder<'_>>)) -> Self;
    fn fill_polygon(points: &[Vec2]) -> Self;
    fn stroke_with(
        func: impl FnOnce(&mut NoAttributes<StrokeBuilder<'_>>),
        options: &StrokeOptions,
    ) -> Self;
    fn stroke_polygon(points: &[Vec2], options: &StrokeOptions) -> Self;
}
impl MeshLyonExtensions for Mesh {
    fn fill_with(func: impl FnOnce(&mut NoAttributes<FillBuilder<'_>>)) -> Self {
        let mut geometry = VertexBuffers::new();
        let mut builder = simple_builder(&mut geometry);
        let mut tessellator = FillTessellator::new();
        let options = FillOptions::default();
        let mut builder: lyon_tessellation::path::builder::NoAttributes<
            lyon_tessellation::FillBuilder<'_>,
        > = tessellator.builder(&options, &mut builder);

        func(&mut builder);

        builder.build().unwrap();

        mesh_from_buffers(geometry)
    }
    fn fill_polygon(points: &[Vec2]) -> Self {
        Self::fill_with(|builder| {
            let points = points
                .iter()
                .map(|p| Point2D::new(p.x, p.y))
                .collect::<Vec<_>>();
            builder.add_polygon(Polygon {
                points: &points,
                closed: true,
            });
        })
    }

    fn stroke_with(
        func: impl FnOnce(&mut NoAttributes<StrokeBuilder<'_>>),
        options: &StrokeOptions,
    ) -> Self {
        let mut geometry = VertexBuffers::new();
        let mut builder = simple_builder(&mut geometry);
        let mut tessellator = StrokeTessellator::new();
        let mut builder = tessellator.builder(&options, &mut builder);

        func(&mut builder);

        builder.build().unwrap();

        mesh_from_buffers(geometry)
    }
    fn stroke_polygon(points: &[Vec2], options: &StrokeOptions) -> Self {
        Self::stroke_with(
            |builder| {
                let points = points
                    .iter()
                    .map(|p| Point2D::new(p.x, p.y))
                    .collect::<Vec<_>>();
                builder.add_polygon(Polygon {
                    points: &points,
                    closed: true,
                });
            },
            options,
        )
    }
}

fn mesh_from_buffers(buffers: VertexBuffers<Point2D<f32, UnknownUnit>, u16>) -> Mesh {
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    );
    let vertices = buffers
        .vertices
        .iter()
        .map(|v| vec3(v.x, v.y, 0.0))
        .collect::<Vec<_>>();
    let rect = buffers
        .vertices
        .into_iter()
        .fold(Rect::EMPTY, |rect, v| rect.union_point(vec2(v.x, v.y)));
    let center = rect.center();
    let size = rect.size();
    let scale = 1.0 / size.x.max(size.y);

    let uvs = vertices
        .iter()
        .map(|v| {
            let uv = vec2(v.x, v.y) - center;
            vec2(uv.x * scale + 0.5, uv.y * scale + 0.5)
        })
        .collect::<Vec<_>>();
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U16(buffers.indices.clone()));

    mesh
}
