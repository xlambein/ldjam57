use bevy::{prelude::*, window::PrimaryWindow};

pub struct GlobalCursorPlugin;

impl Plugin for GlobalCursorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GlobalCursor>()
            .add_systems(Update, update_global_cursor);
    }
}

/// We will store the world position of the mouse cursor here.
#[derive(Resource, Default)]
pub struct GlobalCursor(Vec2);

impl GlobalCursor {
    pub fn position(&self) -> Vec2 {
        self.0
    }
}

/// Used to help identify our main camera
#[derive(Component)]
pub struct MainCamera;

fn update_global_cursor(
    mut mycoords: ResMut<GlobalCursor>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) {
    let (camera, camera_transform) = q_camera.single();
    let window = q_window.single();
    if let Some(world_position) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world_2d(camera_transform, cursor).ok())
    {
        mycoords.0 = world_position;
    }
}
