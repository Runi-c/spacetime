use bevy::prelude::*;

use crate::{
    layers::FactoryLayer, materials::DitherMaterial, scheduling::Sets, sounds::Sounds,
    z_order::ZOrder, SCREEN_SIZE,
};

use super::{
    camera::CursorPosition,
    grid::{Grid, TileCoords, TILE_SIZE},
    machines::{ammo_factory, hull_fixer, pipe_switch, rocket_factory},
    pipe::PipeFlowMaterial,
    pipe_network::InvalidateNetworks,
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, shop_spawn.in_set(Sets::Spawn))
        .add_systems(Update, shop_item_drag.in_set(Sets::Input))
        .add_observer(shop_item_observers)
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

#[derive(Component)]
pub struct ShopOrder(pub usize);

#[derive(Resource)]
pub struct PickedUpItem(pub Entity);

fn shop_spawn(
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
            children![
                (
                    ShopOrder(0),
                    ammo_factory(&mut materials, flow_material.0.clone()),
                ),
                (
                    ShopOrder(1),
                    pipe_switch(&mut meshes, flow_material.0.clone()),
                ),
                (
                    ShopOrder(2),
                    hull_fixer(&mut meshes, &mut materials, flow_material.0.clone()),
                ),
                (
                    ShopOrder(3),
                    rocket_factory(&mut meshes, &mut materials, flow_material.0.clone()),
                )
            ],
        ))
        .id();
    commands.insert_resource(Shop(shop));
    commands.trigger(InvalidateShopLayout);
}

fn shop_item_observers(trigger: Trigger<OnAdd, ShopItem>, mut commands: Commands) {
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
    mut invalidate: EventWriter<InvalidateNetworks>,
    cursor_pos: CursorPosition,
    shop_items: Query<(&ShopItem, Option<&TileCoords>)>,
    mut grid: ResMut<Grid>,
    shop: Res<Shop>,
    sounds: Res<Sounds>,
    flow_material: Res<PipeFlowMaterial>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<DitherMaterial>>,
) -> Result {
    let target = trigger.target();
    let (shop_item, coords) = shop_items.get(target)?;
    commands.remove_resource::<PickedUpItem>();
    if let Some(tile_pos) = cursor_pos.tile() {
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
            grid.insert_building(tile_pos, spawned);
            commands
                .entity(spawned)
                .insert((TileCoords(tile_pos), ChildOf(grid.entity)));
        }
    }
    if let Some(coords) = coords {
        // thrown away built item
        commands.entity(target).despawn();
        grid.remove_building(coords.0);
    } else {
        // return to shop
        commands.entity(target).insert(ChildOf(shop.0));
        commands.trigger(InvalidateShopLayout);
    }
    invalidate.write(InvalidateNetworks);
    commands.spawn((
        Name::new("Place Machine Sound"),
        AudioPlayer::new(sounds.place_machine.clone()),
        PlaybackSettings::DESPAWN,
    ));

    Ok(())
}

fn shop_item_drag(
    mut shop_items: Query<&mut Transform, With<ShopItem>>,
    picked_up_item: Option<Res<PickedUpItem>>,
    cursor_pos: CursorPosition,
) {
    if let Some(picked_up_item) = picked_up_item {
        if let Some(world_pos) = cursor_pos.world() {
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
    mut shop_items: Query<(&mut Transform, &ShopOrder)>,
) {
    let num = shop_items.iter().count() as f32;
    let width = (TILE_SIZE * 1.5) * num;
    for (mut transform, index) in shop_items.iter_mut() {
        transform.translation.x = (-width / 2.0 + TILE_SIZE * (index.0 as f32 + 0.5) * 1.5).round(); // rounded to help with correct dithering
        transform.translation.y = 0.0;
    }
}
