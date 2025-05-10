use bevy::{prelude::*, window::PrimaryWindow};

use crate::{
    factory::{
        grid::Direction,
        pipe::{pipe_bundle, InvalidateNetworks},
    },
    layers::FactoryLayer,
    materials::DitherMaterial,
    scheduling::Sets,
    sounds::Sounds,
    z_order::ZOrder,
    SCREEN_SIZE,
};

use super::{
    camera::{CursorPosition, FactoryCamera},
    grid::{Grid, TileCoords, TILE_SIZE},
    machines::{ammo_factory, hull_fixer, pipe_switch, rocket_factory},
    pipe::{Pipe, PipeFlowMaterial},
};

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, setup_shop.in_set(Sets::Spawn))
        .add_systems(Update, shop_item_drag.in_set(Sets::Input))
        .add_observer(observe_shop_items)
        .add_observer(shop_layout);
}

#[derive(Resource)]
pub struct Shop(pub Entity);

#[derive(Component)]
pub enum ShopItem {
    AmmoFactory,
    PipeSwitch,
    HullFixer,
    RocketFactory,
}

#[derive(Resource)]
pub struct PickedUpItem(pub Entity);

fn setup_shop(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<DitherMaterial>>,
    flow_material: Res<PipeFlowMaterial>,
) {
    let shop = commands
        .spawn((
            Name::new("Shop"),
            FactoryLayer,
            Transform::from_xyz(0.0, -SCREEN_SIZE.y / 2.0 + 100.0, 0.0),
            ZOrder::SHOP,
            Visibility::Visible,
            Pickable::IGNORE,
            children![
                ammo_factory(&mut materials, flow_material.0.clone()),
                pipe_switch(&mut meshes, flow_material.0.clone()),
                hull_fixer(&mut meshes, &mut materials, flow_material.0.clone()),
                rocket_factory(&mut meshes, &mut materials, flow_material.0.clone()),
            ],
        ))
        .id();
    commands.insert_resource(Shop(shop));
    commands.trigger(InvalidateShopLayout);
}

fn observe_shop_items(trigger: Trigger<OnAdd, ShopItem>, mut commands: Commands) {
    commands
        .entity(trigger.target())
        .observe(shop_item_drag_start)
        .observe(shop_item_drag_end);
}

fn shop_item_drag_start(
    trigger: Trigger<Pointer<DragStart>>,
    mut commands: Commands,
    sounds: Res<Sounds>,
) {
    let target = trigger.target();
    commands.insert_resource(PickedUpItem(target));
    commands.entity(target).remove::<ChildOf>();
    commands.spawn((
        Name::new("Pick Up Machine Sound"),
        AudioPlayer::new(sounds.pickup_machine.clone()),
        PlaybackSettings::DESPAWN,
    ));
}

fn shop_item_drag_end(
    trigger: Trigger<Pointer<DragEnd>>,
    mut commands: Commands,
    mut grid: ResMut<Grid>,
    pipes: Query<&Pipe>,
    shop_items: Query<(&ShopItem, Option<&TileCoords>)>,
    cursor_pos: CursorPosition,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<DitherMaterial>>,
    flow_material: Res<PipeFlowMaterial>,
    shop: Res<Shop>,
    mut invalidate: EventWriter<InvalidateNetworks>,
    sounds: Res<Sounds>,
) {
    let target = trigger.target();
    let (shop_item, coords) = shop_items.get(target).unwrap();
    commands.remove_resource::<PickedUpItem>();
    if let Some(tile_pos) = cursor_pos.tile_pos() {
        info!("Dropped building on position: {:?}", tile_pos);
        if grid.get_tile(tile_pos).is_some() && grid.get_building(tile_pos).is_none() {
            let spawned = match shop_item {
                ShopItem::AmmoFactory => commands
                    .spawn(ammo_factory(&mut materials, flow_material.0.clone()))
                    .id(),
                ShopItem::PipeSwitch => commands
                    .spawn(pipe_switch(&mut meshes, flow_material.0.clone()))
                    .id(),
                ShopItem::HullFixer => commands
                    .spawn(hull_fixer(
                        &mut meshes,
                        &mut materials,
                        flow_material.0.clone(),
                    ))
                    .id(),
                ShopItem::RocketFactory => commands
                    .spawn(rocket_factory(
                        &mut meshes,
                        &mut materials,
                        flow_material.0.clone(),
                    ))
                    .id(),
            };
            grid.get_building_mut(tile_pos).unwrap().replace(spawned);
            commands
                .entity(spawned)
                .insert((TileCoords(tile_pos), ChildOf(grid.entity)));
            for dir in Direction::iter() {
                if let Some(pipe) = grid
                    .get_building(tile_pos + dir.as_ivec2())
                    .filter(|e| pipes.contains(*e))
                {
                    commands.entity(pipe).despawn();
                    let pipe = commands
                        .spawn((pipe_bundle(tile_pos + dir.as_ivec2()), ChildOf(grid.entity)))
                        .id();
                    grid.get_building_mut(tile_pos + dir.as_ivec2())
                        .unwrap()
                        .replace(pipe);
                }
            }
        }
    }
    if coords.is_none() {
        // return to shop
        commands.entity(target).insert(ChildOf(shop.0));
        commands.trigger(InvalidateShopLayout);
    } else {
        // thrown away built item
        commands.entity(target).despawn();
        grid.get_building_mut(coords.unwrap().0).unwrap().take();
    }
    invalidate.write(InvalidateNetworks);
    commands.spawn((
        Name::new("Place Machine Sound"),
        AudioPlayer::new(sounds.place_machine.clone()),
        PlaybackSettings::DESPAWN,
    ));
}

fn shop_item_drag(
    mut shop_items: Query<&mut Transform, With<ShopItem>>,
    picked_up_item: Option<Res<PickedUpItem>>,
    window: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform), With<FactoryCamera>>,
) {
    if let Some(picked_up_item) = picked_up_item {
        let window = window.single().unwrap();
        let (camera, camera_transform) = camera.single().unwrap();
        let screen_pos = window.cursor_position().unwrap_or_default();
        if let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, screen_pos) {
            if let Ok(mut transform) = shop_items.get_mut(picked_up_item.0) {
                transform.translation.x = world_pos.x;
                transform.translation.y = world_pos.y;
            } else {
                warn!("Shop item not found: {:?}", picked_up_item.0);
            }
        }
    }
}

#[derive(Event)]
struct InvalidateShopLayout;
fn shop_layout(
    _trigger: Trigger<InvalidateShopLayout>,
    mut shop_items: Query<&mut Transform, (With<ShopItem>, Without<TileCoords>)>,
) {
    let num = shop_items.iter().count() as f32;
    let width = (TILE_SIZE * 1.5) * num;
    for (i, mut transform) in shop_items.iter_mut().enumerate() {
        transform.translation.x = -width / 2.0 + TILE_SIZE * (i as f32 + 0.5) * 1.5;
        transform.translation.y = 0.0;
    }
}
