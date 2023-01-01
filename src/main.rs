use bevy::math::Vec4Swizzles;

use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use rand::Rng;


struct TileIndex;
impl TileIndex {
    const UNKNOWN: u32 = 0;
    const BOMB: u32 = 9;
    const EMPTY: u32 = 10;
    const FLAG: u32 = 11;
}


#[derive(Component, Clone, Copy)]
struct TileState {
    opened: bool,
    flagged: bool,
    is_bomb: bool,
}

fn restart_sys(mut commands: Commands, tile_q: Query<Entity, With<TilemapId>>, keyboard_input: Res<Input<KeyCode>>){
    if keyboard_input.just_pressed(KeyCode::R) {
        let mut rng = rand::thread_rng();
        for entity in tile_q.iter() {
        // add the components TileValue and TileOpened to the tile storage
            commands.entity(entity).insert(TileState {
                opened: false,
                flagged: false,
                is_bomb: rng.gen_bool(1. / 5.),
            }).insert(TileTextureIndex(0));
        }
    }
}

fn startup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    query: Query<(Entity, &TilemapId)>,
) {
    commands.spawn(Camera2dBundle::default());
    let texture_handle = asset_server.load("numbers.png");

    let map_size = TilemapSize { x: 20, y: 20 };
    let mut tile_storage = TileStorage::empty(map_size);
    let tilemap_entity = commands.spawn_empty().id();
    let tilemap_id = TilemapId(tilemap_entity);

    fill_tilemap(
        TileTextureIndex(TileIndex::UNKNOWN),
        map_size,
        tilemap_id,
        &mut commands,
        &mut tile_storage,
    );

    // add the components TileValue and TileOpened to the tile storage
	let mut rng = rand::thread_rng();
    for entity in tile_storage.iter() {
        let entity = entity.unwrap();
        commands.entity(entity).insert(TileState {
            opened: false,
            flagged: false,
            is_bomb: rng.gen_bool(1. / 5.),
        });
    }

    let tile_size = TilemapTileSize { x: 32.0, y: 32.0 };
    let grid_size: TilemapGridSize = tile_size.into();
    let map_type = TilemapType::Square;
    commands.entity(tilemap_entity).insert(TilemapBundle {
        grid_size,
        size: map_size,
        storage: tile_storage,
        texture: TilemapTexture::Single(texture_handle),
        tile_size,
        map_type,
        transform: get_tilemap_center_transform(&map_size, &grid_size, &map_type, 0.0),
        ..Default::default()
    });
}

#[derive(Resource, Default)]
struct CursorPos(Vec2);

fn mb2_sys(
    mut commands: Commands,
    mouse_input: Res<Input<MouseButton>>,
    windows: Res<Windows>,
    tilemap_q: Query<(
        &TilemapSize,
        &TilemapGridSize,
        &TilemapType,
        &TileStorage,
        &Transform,
    )>,
    mut tilestate_q: Query<&mut TileState>,
    camera_q: Query<(&GlobalTransform, &Camera)>,
) {
    if !mouse_input.just_pressed(MouseButton::Right) {
        return;
    };
    let win = windows.primary();
    let pos = match win.cursor_position() {
        Some(pos) => pos,
        None => return,
    };
    let (camera_transform, cam) = camera_q.single();
    let p3 = cam.viewport_to_world(camera_transform, pos).unwrap().origin;
    let (map_size, grid_size, map_type, tile_storage, transform) = tilemap_q.single();

    let pos = (transform.compute_matrix().inverse() * p3.extend(1.0)).xy();

    let tile_pos = match TilePos::from_world_pos(&pos, map_size, grid_size, map_type) {
        Some(tile_pos) => tile_pos,
        None => return,
    };
    let entity = tile_storage.get(&tile_pos).unwrap(); 
    let mut tile_state = tilestate_q.get_component_mut::<TileState>(entity).unwrap();
    if tile_state.opened {
        return;
    }
    tile_state.flagged = !tile_state.flagged;
    if tile_state.flagged {
        commands.entity(entity).insert(TileTextureIndex(TileIndex::FLAG));
    } else {
        commands.entity(entity).insert(TileTextureIndex(TileIndex::UNKNOWN));
    }
}

fn mb1_sys(
    mut commands: Commands,
    windows: Res<Windows>,
    mouse_input: Res<Input<MouseButton>>,
    tilemap_q: Query<(
        &TilemapSize,
        &TilemapGridSize,
        &TilemapType,
        &TileStorage,
        &Transform,
    )>,
    tilestate_q: Query<&TileState>,
    camera_q: Query<(&GlobalTransform, &Camera)>,
) {
    if !mouse_input.just_released(MouseButton::Left) {
        return;
    }
    let win = windows.primary();
    let pos = match win.cursor_position() {
        Some(pos) => pos,
        None => return,
    };
    let (camera_transform, cam) = camera_q.single();
    let p3 = cam.viewport_to_world(camera_transform, pos).unwrap().origin;
    let (map_size, grid_size, map_type, tile_storage, transform) = tilemap_q.single();

    let pos = (transform.compute_matrix().inverse() * p3.extend(1.0)).xy();

    let tile_pos = match TilePos::from_world_pos(&pos, map_size, grid_size, map_type) {
        Some(tile_pos) => tile_pos,
        None => return,
    };
    let entity = tile_storage.get(&tile_pos).unwrap();
	match tilestate_q.get_component::<TileState>(entity).unwrap() {
        TileState { is_bomb: true, .. } => {
            println!("Game Over");
        },
        TileState { opened: true, .. } => {
            return;
        },
        TileState { flagged: true, .. } => {
            println!("Flagged");
            return;
        },
        _ => {},
    }

    let mut queue: Vec<TilePos> = vec![tile_pos];
    let mut index = 0;
    loop {
        let tile_pos = *match queue.get(index) {
            Some(tile_pos) => tile_pos,
            None => break,
        };
        index += 1;
        // check the number of neighboring mines
        let mut mine_count = 0;
        let mut openable = vec![];
        for neighbor in [
            (-1, -1),
            (-1, 0),
            (-1, 1),
            (0, -1),
            (0, 1),
            (1, -1),
            (1, 0),
            (1, 1),
        ]
        .into_iter()
        .filter_map(move |(x, y)| {
            // return Some(TilePos) if the neighbor is within the bounds of tile_storage.size
            let tx = tile_pos.x as i32 + x;
            let ty = tile_pos.y as i32 + y;
            if tx >= 0
                && tx < tile_storage.size.x as i32
                && ty >= 0
                && ty < tile_storage.size.y as i32
            {
                Some(TilePos {
                    x: tx as u32,
                    y: ty as u32,
                })
            } else {
                None
            }
        }) {
            if let Some(entity) = tile_storage.get(&neighbor) {
                if let Ok(TileState { is_bomb, opened, .. }) = tilestate_q.get_component(entity) {
					if *is_bomb {
						mine_count += 1;
					} else if !opened {
						openable.push(neighbor);
					}
				}
            }
        }

        let entity = tile_storage.get(&tile_pos).unwrap();
        let state = tilestate_q.get_component::<TileState>(entity).unwrap();
        commands
            .entity(entity)
			.insert(TileState {
				opened: true,
				..*state
			})
            .insert(TileTextureIndex(match (mine_count, state.is_bomb) {
				(_, true) => TileIndex::BOMB,
				(0, _) => TileIndex::EMPTY,
				(n, _) => n,
			}));
        if mine_count == 0 {
            for open in openable {
                // wish i could use a hashset
                if queue.contains(&open) {
                    continue;
                }
                queue.push(open);
            }
        }
    }
}

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    window: WindowDescriptor {
                        title: String::from("Minesweeper"),
                        ..Default::default()
                    },
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_system(mb1_sys)
        .add_system(mb2_sys)
        .add_system(restart_sys)
        .init_resource::<CursorPos>()
        .add_plugin(TilemapPlugin)
        .add_startup_system(startup)
        .run();
}
