use crate::components::SpectatorCamera;
use crate::resources::CameraGesture;
use bevy::prelude::*;

pub(crate) fn camera_controls(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut gesture: ResMut<CameraGesture>,
    mut cameras: Query<(&mut Transform, &mut Projection), With<SpectatorCamera>>,
) {
    for (mut transform, mut projection) in &mut cameras {
        transform.translation += Vec3::new(gesture.pan.x, 0.0, gesture.pan.y);
        let x = (keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight)) as i32
            - (keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft)) as i32;
        let z = (keys.pressed(KeyCode::KeyS) || keys.pressed(KeyCode::ArrowDown)) as i32
            - (keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::ArrowUp)) as i32;
        transform.translation +=
            Vec3::new(x as f32, 0.0, z as f32) * time.delta_secs().min(0.1) * 40.0;
        if let Projection::Orthographic(ref mut orthographic) = *projection {
            let zoom = (keys.pressed(KeyCode::Minus) || keys.pressed(KeyCode::NumpadSubtract))
                as i32
                - (keys.pressed(KeyCode::Equal) || keys.pressed(KeyCode::NumpadAdd)) as i32;
            orthographic.scale = (orthographic.scale * gesture.zoom
                + zoom as f32 * time.delta_secs().min(0.1))
            .clamp(0.3, 2.0);
        }
    }
    gesture.pan = Vec2::ZERO;
    gesture.zoom = 1.0;
}
