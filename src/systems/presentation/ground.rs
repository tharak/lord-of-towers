use crate::resources::{InfluenceField, VisualAssets};
use crate::types::faction_color;
use bevy::mesh::VertexAttributeValues;
use bevy::prelude::*;

pub(crate) fn ground_colors(
    field: Res<InfluenceField>,
    assets: Res<VisualAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut revision: Local<Option<u64>>,
) {
    if *revision == Some(field.revision) {
        return;
    }
    let Some(mut mesh) = meshes.get_mut(&assets.ground) else {
        return;
    };
    let Some(VertexAttributeValues::Float32x3(positions)) =
        mesh.attribute(Mesh::ATTRIBUTE_POSITION)
    else {
        return;
    };
    let colors: Vec<[f32; 4]> = positions
        .iter()
        .map(|p| {
            let influence = field.sample(Vec2::new(p[0], p[2]));
            let mut ordered: Vec<(usize, f32)> = influence.iter().copied().enumerate().collect();
            ordered.sort_by(|a, b| b.1.total_cmp(&a.1));
            let (winner, strength) = ordered[0];
            let blend = ((strength - ordered[1].1) / strength.max(0.01)).clamp(0.0, 1.0)
                * (strength * 0.5).clamp(0.0, 0.65);
            let c = faction_color(winner).to_linear();
            [
                0.07 + (c.red - 0.07) * blend,
                0.09 + (c.green - 0.09) * blend,
                0.11 + (c.blue - 0.11) * blend,
                1.0,
            ]
        })
        .collect();
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    *revision = Some(field.revision);
}
