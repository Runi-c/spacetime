use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::mesh::{Indices, PrimitiveTopology},
};
use lyon_tessellation::{
    self,
    geom::euclid::{Point2D, UnknownUnit},
    geometry_builder::simple_builder,
    path::Polygon,
    FillOptions, FillTessellator, StrokeOptions, StrokeTessellator, VertexBuffers,
};

pub fn fill_polygon(points: &[Vec2]) -> Mesh {
    let mut geometry = VertexBuffers::new();
    let mut builder = simple_builder(&mut geometry);
    let mut tessellator = FillTessellator::new();
    let options = FillOptions::default();
    let mut builder = tessellator.builder(&options, &mut builder);

    let points = points
        .iter()
        .map(|p| Point2D::new(p.x, p.y))
        .collect::<Vec<_>>();
    builder.add_polygon(Polygon {
        points: &points,
        closed: true,
    });
    builder.build().unwrap();

    mesh_from_buffers(geometry)
}

pub fn stroke_polygon(points: &[Vec2], width: f32) -> Mesh {
    let mut geometry = VertexBuffers::new();
    let mut builder = simple_builder(&mut geometry);
    let mut tessellator = StrokeTessellator::new();
    let options = StrokeOptions::default().with_line_width(width);
    let mut builder = tessellator.builder(&options, &mut builder);

    let points = points
        .iter()
        .map(|p| Point2D::new(p.x, p.y))
        .collect::<Vec<_>>();
    builder.add_polygon(Polygon {
        points: &points,
        closed: true,
    });
    builder.build().unwrap();

    mesh_from_buffers(geometry)
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
