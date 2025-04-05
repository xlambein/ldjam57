use avian2d::prelude::*;
use bevy::{
    input::mouse::MouseWheel,
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
        .add_plugins((
            bevy::sprite::Material2dPlugin::<BlurMaterial>::default(),
            PhysicsPlugins::default(),
        ))
        .insert_resource(Gravity(avian2d::math::Vector::NEG_Y * 9.81 * 100.0))
        .add_systems(Update, quit_on_ctrl_q)
        .add_systems(Startup, setup)
        .add_systems(Update, update_material_blur)
        .add_systems(Update, update_focus_depth)
        .insert_resource(FocusDepth(1.0))
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
    mut color_materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2d);
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(259.0, 194.0))),
        MeshMaterial2d(materials.add(BlurMaterial {
            blur_intensity: 0.0,
            texture: asset_server.load("crate.png"),
        })),
        Transform::default().with_translation(Vec3::new(-100.0, -100.0, 0.0)),
        RigidBody::Static,
        Collider::rectangle(259.0, 194.0),
    ));
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(259.0, 194.0))),
        MeshMaterial2d(materials.add(BlurMaterial {
            blur_intensity: 0.0,
            texture: asset_server.load("crate.png"),
        })),
        Transform::default().with_translation(Vec3::new(200.0, -50.0, 5.0)),
        RigidBody::Static,
        Collider::rectangle(259.0, 194.0),
    ));
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(50.0, 50.0))),
        MeshMaterial2d(color_materials.add(ColorMaterial {
            color: Color::Srgba(Srgba {
                red: 1.0,
                green: 0.2,
                blue: 0.3,
                alpha: 1.0,
            }),
            ..default()
        })),
        Transform::default().with_translation(Vec3::new(-100.0, 500.0, 10.0)),
        RigidBody::Dynamic,
        Collider::rectangle(50.0, 50.0),
    ));
}

const MIN_FOCUS_DEPTH: f32 = 0.0;
const MAX_FOCUS_DEPTH: f32 = 10.0;

#[derive(Resource)]
struct FocusDepth(f32);

fn update_material_blur(
    q: Query<(&MeshMaterial2d<BlurMaterial>, &GlobalTransform)>,
    mut materials: ResMut<Assets<BlurMaterial>>,
    focus_depth: Res<FocusDepth>,
) {
    for (handle, transform) in q.iter() {
        if let Some(material) = materials.get_mut(handle) {
            let depth = transform.translation().z;
            material.blur_intensity = (focus_depth.0 - depth).abs() * 10.0;
        }
    }
}

fn update_focus_depth(
    mut focus_depth: ResMut<FocusDepth>,
    mut mouse_wheel_events: EventReader<MouseWheel>,
) {
    for event in mouse_wheel_events.read() {
        if event.y > 0.0 {
            focus_depth.0 += 0.2;
        } else {
            focus_depth.0 -= 0.2;
        }
        focus_depth.0 = f32::min(MAX_FOCUS_DEPTH, f32::max(MIN_FOCUS_DEPTH, focus_depth.0));
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
