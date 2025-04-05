mod global_cursor;

use avian2d::prelude::*;
use bevy::{
    input::{
        common_conditions::{input_just_pressed, input_just_released},
        mouse::{AccumulatedMouseMotion, MouseButtonInput, MouseWheel},
    },
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef, ShaderType},
};
use bevy_aseprite_ultra::prelude::*;
use global_cursor::GlobalCursor;

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
            global_cursor::GlobalCursorPlugin,
            AsepriteUltraPlugin,
        ))
        .insert_resource(Gravity(avian2d::math::Vector::NEG_Y * 9.81 * 100.0))
        .add_systems(Startup, setup)
        .add_systems(Update, quit_on_ctrl_q)
        .add_systems(Update, update_material_blur)
        .add_systems(Update, update_focus_depth)
        .add_systems(Update, update_collider_on_focus)
        .add_systems(Update, update_player_position)
        .add_systems(Update, log_cursor_clicks)
        .add_systems(
            Update,
            (
                wheel_enable.run_if(input_just_pressed(MouseButton::Left)),
                wheel_disable.run_if(input_just_released(MouseButton::Left)),
                wheel_scroll_focus,
            ),
        )
        .insert_resource(FocusDepth(0.0))
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
    let back_collider = Collider::convex_hull(vec![
        Vec2::new(-210.34375, 45.656258),
        Vec2::new(-121.75392, 46.863293),
        Vec2::new(-123.56251, 6.4258),
        Vec2::new(-211.32423, 12.707018),
    ])
    .unwrap();

    let mid_collider = Collider::convex_hull(vec![
        Vec2::new(-90.46875, 6.0351415),
        Vec2::new(0.980453, 4.503922),
        Vec2::new(2.101593, -122.46483),
        Vec2::new(-95.58983, -122.21875),
    ])
    .unwrap();

    let front_collider = Collider::convex_hull(vec![
        Vec2::new(66.339874, -56.17968),
        Vec2::new(213.9531, -59.49609),
        Vec2::new(71.07811, -107.27344),
        Vec2::new(211.02737, -111.41797),
    ])
    .unwrap();

    commands.spawn((
        Camera2d,
        global_cursor::MainCamera,
        Transform::default().with_scale(Vec3::splat(0.33)),
    ));

    commands.spawn((
        Sprite {
            image: asset_server.load("world/simplified/level_0/_bg.png"),
            ..Default::default()
        },
        Transform::default().with_translation(Vec3::Z * -5.),
    ));

    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(424., 256.))),
        MeshMaterial2d(materials.add(BlurMaterial {
            settings: BlurSettings::default(),
            texture: asset_server.load("world/simplified/level_0/back.png"),
        })),
        Transform::default().with_translation(Vec3::ZERO),
        RigidBody::Static,
        back_collider,
    ));
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(424., 256.))),
        MeshMaterial2d(materials.add(BlurMaterial {
            settings: BlurSettings::default(),
            texture: asset_server.load("world/simplified/level_0/mid.png"),
        })),
        Transform::default().with_translation(Vec3::Z * 5.),
        RigidBody::Static,
        mid_collider,
    ));
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(424., 256.))),
        MeshMaterial2d(materials.add(BlurMaterial {
            settings: BlurSettings::default(),
            texture: asset_server.load("world/simplified/level_0/front.png"),
        })),
        Transform::default().with_translation(Vec3::Z * 10.),
        RigidBody::Static,
        front_collider,
    ));

    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(16.0, 16.0))),
        MeshMaterial2d(color_materials.add(ColorMaterial {
            color: Color::Srgba(Srgba {
                red: 1.0,
                green: 0.2,
                blue: 0.3,
                alpha: 1.0,
            }),
            ..default()
        })),
        Transform::default().with_translation(Vec3::new(-187.0, 68.0, 20.)),
        RigidBody::Dynamic,
        Collider::rectangle(16.0, 16.0),
        PlayerCharacter,
    ));

    commands.spawn((
        Sprite::default(),
        AseSpriteAnimation {
            aseprite: asset_server.load("wheel.aseprite"),
            animation: Animation::tag("scroll"),
        },
        ManualTick,
        Wheel,
        Transform::default()
            .with_translation(Vec3::new(200.0, 0.0, 100.0))
            .with_scale(Vec2::splat(2.0).extend(1.0)),
    ));
}

const MIN_FOCUS_DEPTH: f32 = 0.0;
const MAX_FOCUS_DEPTH: f32 = 10.0;
const FOCUS_COLLISION_THRESHOLD: f32 = 1.5;

#[derive(Component)]
struct Wheel;

#[derive(Component)]
struct WheelEnabled;

fn wheel_enable(
    mut commands: Commands,
    q: Query<(Entity, &GlobalTransform), With<Wheel>>,
    cursor: Res<GlobalCursor>,
) {
    const SIZE: Vec2 = Vec2::new(88.0, 32.0);
    for (entity, transform) in q.iter() {
        let rect = Rect::from_center_size(
            transform.translation().truncate(),
            SIZE * transform.scale().truncate(),
        );
        if rect.contains(cursor.position()) {
            commands.entity(entity).insert(WheelEnabled);
        }
    }
}

fn wheel_disable(mut commands: Commands, q: Query<Entity, With<WheelEnabled>>) {
    for entity in q.iter() {
        commands.entity(entity).remove::<WheelEnabled>();
    }
}

fn wheel_scroll_focus(
    mut q: Query<(&mut AnimationState,), With<WheelEnabled>>,
    accumulated_mouse_motion: Res<AccumulatedMouseMotion>,
    mut focus_depth: ResMut<FocusDepth>,
) {
    if q.is_empty() {
        return;
    }
    focus_depth.increase(accumulated_mouse_motion.delta.x / 20.0);
    for (mut animation_state,) in q.iter_mut() {
        animation_state.current_frame = (focus_depth.0 * 3.0) as u16 % 3;
    }
}

#[derive(Resource)]
struct FocusDepth(f32);

impl FocusDepth {
    fn increase(&mut self, amount: f32) {
        self.0 += amount;
        self.0 = f32::min(MAX_FOCUS_DEPTH, f32::max(MIN_FOCUS_DEPTH, self.0));
    }
}

fn update_material_blur(
    q: Query<(&MeshMaterial2d<BlurMaterial>, &GlobalTransform)>,
    mut materials: ResMut<Assets<BlurMaterial>>,
    focus_depth: Res<FocusDepth>,
) {
    for (handle, transform) in q.iter() {
        if let Some(material) = materials.get_mut(handle) {
            let depth = transform.translation().z;
            material.settings.blur_intensity = (focus_depth.0 - depth).abs() * 1.5;
        }
    }
}

fn update_collider_on_focus(
    mut commands: Commands,
    q: Query<(Entity, &GlobalTransform, &MeshMaterial2d<BlurMaterial>)>,
    focus_depth: Res<FocusDepth>,
) {
    for (entity, transform, _) in q.iter() {
        let depth = transform.translation().z;
        if (focus_depth.0 - depth).abs() > FOCUS_COLLISION_THRESHOLD {
            commands.entity(entity).insert(ColliderDisabled);
        } else {
            commands.entity(entity).remove::<ColliderDisabled>();
        }
    }
}

fn update_focus_depth(
    mut focus_depth: ResMut<FocusDepth>,
    mut mouse_wheel_events: EventReader<MouseWheel>,
) {
    for event in mouse_wheel_events.read() {
        if event.y > 0.0 {
            focus_depth.increase(0.2);
        } else {
            focus_depth.increase(-0.2);
        }
    }
}

fn update_player_position(
    mut q: Query<(Entity, &PlayerCharacter, &mut Transform)>,
    window_q: Query<&Window>,
) {
    let window = window_q.single();
    for (_, _, mut transform) in q.iter_mut() {
        if transform.translation.y < (-window.resolution.height() / 2.0) {
            transform.translation.y *= -1.0;
        }
    }
}

fn log_cursor_clicks(
    mut mouse_button_events: EventReader<MouseButtonInput>,
    cursor: Res<GlobalCursor>,
) {
    for event in mouse_button_events.read() {
        if event.state.is_pressed() {
            eprintln!("{}", cursor.position());
        }
    }
}

#[derive(Component, Default)]
struct PlayerCharacter;

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct BlurMaterial {
    #[uniform(0)]
    settings: BlurSettings,
    #[texture(1)]
    #[sampler(2)]
    texture: Handle<Image>,
}

#[derive(ShaderType, Debug, Clone)]
struct BlurSettings {
    blur_intensity: f32,
    // WebGL2 structs must be 16 byte aligned.
    #[cfg(target_arch = "wasm32")]
    _webgl2_padding: Vec3,
}

impl Default for BlurSettings {
    fn default() -> Self {
        BlurSettings {
            blur_intensity: 0.0,
            // WebGL2 structs must be 16 byte aligned.
            #[cfg(target_arch = "wasm32")]
            _webgl2_padding: Vec3::ZERO,
        }
    }
}

impl bevy::sprite::Material2d for BlurMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/blur.wgsl".into()
    }

    fn alpha_mode(&self) -> bevy::sprite::AlphaMode2d {
        bevy::sprite::AlphaMode2d::Blend
    }
}
