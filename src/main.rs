mod global_cursor;

use avian2d::prelude::*;
use bevy::{
    input::mouse::{MouseButtonInput, MouseWheel},
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef, ShaderType},
};
use bevy_ecs_ldtk::{
    app::{LdtkEntityAppExt, LdtkIntCellAppExt},
    utils::grid_coords_to_translation,
};
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
            bevy_ecs_ldtk::LdtkPlugin,
            bevy::sprite::Material2dPlugin::<BlurMaterial>::default(),
            PhysicsPlugins::default(),
            global_cursor::GlobalCursorPlugin,
        ))
        .insert_resource(bevy_ecs_ldtk::LevelSelection::index(0))
        .insert_resource(bevy_ecs_ldtk::LdtkSettings {
            level_spawn_behavior: bevy_ecs_ldtk::LevelSpawnBehavior::UseZeroTranslation,
            ..Default::default()
        })
        .register_ldtk_int_cell_for_layer::<CollisionBundle>("collision", 1)
        .register_ldtk_entity::<PlayerBundle>("player")
        .insert_resource(Gravity(avian2d::math::Vector::NEG_Y * 9.81 * 100.0))
        .add_systems(Startup, setup)
        .add_systems(Update, quit_on_ctrl_q)
        .add_systems(Update, (process_player, process_new_level_geometry))
        .add_systems(Update, update_material_blur)
        .add_systems(Update, update_focus_depth)
        .add_systems(Update, update_collider_on_focus)
        .add_systems(Update, update_player_position)
        .add_systems(Update, log_cursor_clicks)
        .insert_resource(FocusDepth(0.0))
        .run();
}

#[derive(Default, Bundle, bevy_ecs_ldtk::LdtkEntity)]
struct PlayerBundle {
    player: PlayerCharacter,
    #[grid_coords]
    grid_coords: bevy_ecs_ldtk::GridCoords,
}

fn process_player(
    mut commands: Commands,
    new_players: Query<(Entity, &bevy_ecs_ldtk::GridCoords), Added<PlayerCharacter>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut color_materials: ResMut<Assets<ColorMaterial>>,
) {
    for (entity, grid_coords) in new_players.iter() {
        commands.entity(entity).insert((
            PlayerCharacter,
            Mesh2d(meshes.add(Rectangle::new(24.0, 24.0))),
            MeshMaterial2d(color_materials.add(ColorMaterial {
                color: Color::srgba(0.8, 0.3, 0.2, 1.0),
                ..Default::default()
            })),
            Transform::from_translation(
                bevy_ecs_ldtk::utils::grid_coords_to_translation(*grid_coords, IVec2::splat(8))
                    .extend(10.0),
            ),
            RigidBody::Dynamic,
            Collider::rectangle(24.0, 24.0),
        ));
    }
}

#[derive(Component, Default)]
struct LevelGeometry;

#[derive(Default, Bundle, bevy_ecs_ldtk::LdtkIntCell)]
struct CollisionBundle {
    geometry: LevelGeometry,
}

fn process_new_level_geometry(
    mut commands: Commands,
    new_level_geometry: Query<(Entity, &bevy_ecs_ldtk::GridCoords), Added<LevelGeometry>>,
) {
    for (entity, grid_coords) in new_level_geometry.iter() {
        let translation = grid_coords_to_translation(*grid_coords, IVec2::splat(4)).extend(0.0);
        commands.entity(entity).insert((
            Transform::from_translation(translation),
            RigidBody::Static,
            Collider::rectangle(4.0, 4.0),
        ));
    }
}

#[derive(Component, Default)]
struct BackGeometry;

#[derive(Default, Bundle, bevy_ecs_ldtk::LdtkIntCell)]
struct BackGeometryBundle {
    back_geometry: BackGeometry,
}

#[derive(Component, Default)]
struct MidGeometry;

#[derive(Default, Bundle, bevy_ecs_ldtk::LdtkIntCell)]
struct MidGeometryBundle {
    back_geometry: MidGeometry,
}

#[derive(Component, Default)]
struct FrontGeometry;

#[derive(Default, Bundle, bevy_ecs_ldtk::LdtkIntCell)]
struct FrontGeometryBundle {
    back_geometry: FrontGeometry,
}

fn process_new_z_geometry(
    mut commands: Commands,
    new_back_geometry: Query<(Entity, &bevy_ecs_ldtk::GridCoords), Added<BackGeometry>>,
    new_mid_geometry: Query<(Entity, &bevy_ecs_ldtk::GridCoords), Added<MidGeometry>>,
    new_front_geometry: Query<(Entity, &bevy_ecs_ldtk::GridCoords), Added<FrontGeometry>>,
    mut meshes: ResMut<Assets<Mesh>>,
    ldtk_assets: Res<Assets<LdtkAsset>>,
) {
    for (entity, grid_coords) in new_back_geometry.iter() {
        commands.entity(entity).insert((
            Transform::from_translation(
                bevy_ecs_ldtk::utils::grid_coords_to_translation(*grid_coords, IVec2::splat(8))
                    .extend(0.),
            ),
            Mesh2d(meshes.add(Rectangle::new(8.0, 8.0))),
            // MeshMaterial2d(materials.add(BlurMaterial {
            //     settings: BlurSettings::default(),
            //     texture: asset_server.load("crate.png"),
            // }))
        ));
    }
    for (entity, grid_coords) in new_mid_geometry.iter() {}
    for (entity, grid_coords) in new_front_geometry.iter() {}
}

fn quit_on_ctrl_q(keys: Res<ButtonInput<KeyCode>>, mut exit: EventWriter<AppExit>) {
    if keys.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight])
        && keys.just_pressed(KeyCode::KeyQ)
    {
        exit.send(AppExit::Success);
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((Camera2d, global_cursor::MainCamera));
    // commands.spawn((
    //     Mesh2d(meshes.add(Rectangle::new(259.0, 194.0))),
    //     MeshMaterial2d(materials.add(BlurMaterial {
    //         settings: BlurSettings::default(),
    //         texture: asset_server.load("crate.png"),
    //     })),
    //     Transform::default().with_translation(Vec3::new(-100.0, -100.0, 0.0)),
    //     RigidBody::Static,
    //     crate_collider.clone(),
    // ));
    // commands.spawn((
    //     Mesh2d(meshes.add(Rectangle::new(259.0, 194.0))),
    //     MeshMaterial2d(materials.add(BlurMaterial {
    //         settings: BlurSettings::default(),
    //         texture: asset_server.load("crate.png"),
    //     })),
    //     Transform::default().with_translation(Vec3::new(-50.0, -200.0, 5.0)),
    //     RigidBody::Static,
    //     crate_collider,
    // ));
    // commands.spawn((
    //     Mesh2d(meshes.add(Rectangle::new(50.0, 50.0))),
    //     MeshMaterial2d(color_materials.add(ColorMaterial {
    //         color: Color::Srgba(Srgba {
    //             red: 1.0,
    //             green: 0.2,
    //             blue: 0.3,
    //             alpha: 1.0,
    //         }),
    //         ..default()
    //     })),
    //     Transform::default().with_translation(Vec3::new(-100.0, 500.0, 10.0)),
    //     RigidBody::Dynamic,
    //     Collider::rectangle(50.0, 50.0),
    //     PlayerCharacter,
    // ));
    commands.spawn(bevy_ecs_ldtk::LdtkWorldBundle {
        ldtk_handle: asset_server.load("world.ldtk").into(),
        ..Default::default()
    });
}

const MIN_FOCUS_DEPTH: f32 = 0.0;
const MAX_FOCUS_DEPTH: f32 = 10.0;
const FOCUS_COLLISION_THRESHOLD: f32 = 1.5;

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
            material.settings.blur_intensity = (focus_depth.0 - depth).abs() * 10.0;
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
            focus_depth.0 += 0.2;
        } else {
            focus_depth.0 -= 0.2;
        }
        focus_depth.0 = f32::min(MAX_FOCUS_DEPTH, f32::max(MIN_FOCUS_DEPTH, focus_depth.0));
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
