use bevy::{
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef},
};

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "LDJam 57 - Depths".into(),
                        name: Some("ldjam57".into()),
                        // Tells Wasm to resize the window according to the available canvas
                        fit_canvas_to_parent: true,
                        ..default()
                    }),
                    ..default()
                })
                // Disable smoothing for better pixel art
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugins((bevy::sprite::Material2dPlugin::<BlurMaterial>::default(),))
        .add_systems(Update, quit_on_ctrl_q)
        .add_systems(Startup, setup)
        .add_systems(Update, update_material_blur)
        .run();
}

fn quit_on_ctrl_q(keys: Res<ButtonInput<KeyCode>>, mut exit: EventWriter<AppExit>) {
    if keys.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight])
        && keys.just_pressed(KeyCode::KeyQ)
    {
        exit.send(AppExit::Success);
    }
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<BlurMaterial>>,
) {
    commands.spawn(Camera2d);
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(1600.0, 1062.0))),
        MeshMaterial2d(materials.add(BlurMaterial {
            blur_intensity: 0.0,
            texture: asset_server.load("computer.png"),
        })),
        Transform::default().with_translation(Vec3::new(-100.0, -50.0, 0.0)),
    ));
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(1600.0, 1062.0))),
        MeshMaterial2d(materials.add(BlurMaterial {
            blur_intensity: 0.0,
            texture: asset_server.load("computer.png"),
        })),
        Transform::default().with_translation(Vec3::new(200.0, 50.0, 1.0)),
    ));
}

fn update_material_blur(
    q: Query<&MeshMaterial2d<BlurMaterial>>,
    mut materials: ResMut<Assets<BlurMaterial>>,
    time: Res<Time>,
) {
    for handle in q.iter() {
        if let Some(material) = materials.get_mut(handle) {
            material.blur_intensity = time.elapsed_secs().sin().abs();
        }
    }
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct BlurMaterial {
    #[uniform(0)]
    blur_intensity: f32,
    #[texture(1)]
    #[sampler(2)]
    texture: Handle<Image>,
}

impl bevy::sprite::Material2d for BlurMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/blur.wgsl".into()
    }

    fn alpha_mode(&self) -> bevy::sprite::AlphaMode2d {
        bevy::sprite::AlphaMode2d::Blend
    }
}
