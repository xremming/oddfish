use std::collections::HashMap;

use bevy::{input::mouse::MouseMotion, prelude::*, window::PrimaryWindow};

use mousetoria::map::{Neighbors, Terrain, Tile, TileMap, TILE_SIZE};

#[derive(Component)]
struct PrimaryCamera;

type QueryPrimaryCameraTransform<'world, 'state, 'transform> =
    Query<'world, 'state, &'transform mut Transform, (With<Camera2d>, With<PrimaryCamera>)>;

fn add_camera(mut commands: Commands) {
    commands.spawn((PrimaryCamera, Camera2dBundle { ..default() }));
}

fn add_tilemap(mut commands: Commands) {
    let mut map = TileMap::new(20, 30);
    map[(0, 0)] = Terrain::Mountain.as_display("mountain.png");

    commands.add(map);
}

fn update_neighbors(mut tiles_query: Query<(Entity, &Tile, &mut Neighbors)>) {
    let tiles = {
        let _build_tiles_span = info_span!("build_tiles").entered();

        let mut tiles = HashMap::new();
        for (entity, tile, _) in &mut tiles_query {
            tiles.insert((tile.x, tile.y), entity);
        }
        tiles
    };

    let _update_neighbors_span = info_span!("update_neighbors").entered();

    tiles_query
        .par_iter_mut()
        .for_each_mut(|(_, tile, mut neighbors)| {
            neighbors.update_neighbors((tile.x, tile.y), &tiles);
        });
}

fn debug_tiles(
    mut gizmos: Gizmos,
    tilemap_query: Query<(&Tile, &GlobalTransform)>,
    camera: Query<(&Camera, &GlobalTransform), With<PrimaryCamera>>,
    window: Query<&Window, With<PrimaryWindow>>,
) {
    let (camera, camera_transform) = camera.single();
    let window = window.single();

    let cursor_position = window
        .cursor_position()
        .and_then(|cursor_position| camera.viewport_to_world_2d(camera_transform, cursor_position));

    for (tile, transform) in &tilemap_query {
        let (scale, _, translation) = transform.to_scale_rotation_translation();
        const PADDING: f32 = 0.95;
        let size = TILE_SIZE * scale.truncate() * PADDING;
        let color = match cursor_position {
            Some(cursor_position) => {
                let tile_position = translation.truncate();
                let hitbox = Rect::from_center_size(tile_position, size);
                if hitbox.contains(cursor_position) {
                    Color::RED
                } else {
                    tile.terrain.debug_color()
                }
            }
            None => tile.terrain.debug_color(),
        };

        gizmos.rect_2d(translation.truncate(), 0.0, size - 4.0, color);
        // gizmos.circle_2d(translation.truncate(), radius, color);
    }
}

const CAMERA_SPEED: f32 = 100.0;

fn move_camera(
    time: Res<Time>,
    mut query: QueryPrimaryCameraTransform,
    input: Res<Input<KeyCode>>,
) {
    let mut input_vec = Vec2::ZERO;
    if input.pressed(KeyCode::W) {
        input_vec += Vec2::Y;
    }
    if input.pressed(KeyCode::S) {
        input_vec -= Vec2::Y;
    }
    if input.pressed(KeyCode::A) {
        input_vec -= Vec2::X;
    }
    if input.pressed(KeyCode::D) {
        input_vec += Vec2::X;
    }

    if input_vec == Vec2::ZERO {
        return;
    }

    let translation = input_vec.normalize().extend(0.0) * CAMERA_SPEED * time.delta_seconds();

    query.single_mut().translation += translation;
}

fn set_drag_state(
    mouse_button: Res<Input<MouseButton>>,
    mut drag_state: ResMut<NextState<DragState>>,
) {
    if mouse_button.just_pressed(MouseButton::Left) {
        drag_state.set(DragState::Dragging);
    }

    if mouse_button.just_released(MouseButton::Left) {
        drag_state.set(DragState::NotDragging);
    }
}

fn drag_camera(mut query: QueryPrimaryCameraTransform, mut mouse_motion: EventReader<MouseMotion>) {
    let mut camera = query.single_mut();

    for event in mouse_motion.iter() {
        let translation = {
            let mut v = event.delta.extend(0.0);
            v.x *= -1.0;
            v
        };

        camera.translation += translation;
    }
}

#[derive(States, Default, Debug, PartialEq, Eq, Hash, Clone)]
enum DragState {
    #[default]
    NotDragging,
    Dragging,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(Msaa::Sample8)
        .add_state::<DragState>()
        .add_systems(Startup, (add_camera, add_tilemap))
        .add_systems(
            Update,
            (
                set_drag_state,
                (
                    drag_camera.run_if(state_exists_and_equals(DragState::Dragging)),
                    move_camera,
                ),
                update_neighbors,
                debug_tiles,
            )
                .chain(),
        )
        .run();

    // simulation_handle.join().unwrap();
}
