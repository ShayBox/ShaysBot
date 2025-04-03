use std::{cmp::Ordering, sync::LazyLock};

use azalea::{
    app::{App, Plugin},
    attack::AttackEvent,
    ecs::prelude::*,
    entity::{metadata::AbstractMonster, EyeHeight, Position},
    inventory::{
        operations::{ClickOperation, SwapClick},
        ContainerClickEvent,
        Inventory,
    },
    nearest_entity::EntityFinder,
    pathfinder::Pathfinder,
    physics::PhysicsSet,
    prelude::*,
    registry::Item,
    world::MinecraftEntityId,
    LookAtEvent,
};

use crate::prelude::*;

/// Automatically swap and attack nearby monsters
pub struct AutoKillPlugin;

impl Plugin for AutoKillPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            GameTick,
            Self::handle_auto_kill
                .after(AutoLookPlugin::handle_auto_look)
                .before(GameTickPlugin::handle_game_ticks)
                .before(PhysicsSet),
        );
    }
}

impl AutoKillPlugin {
    /// # Panics
    /// Will panic if ?
    pub fn handle_auto_kill(
        mut query: Query<(Entity, &LocalSettings, &GameTicks, &Inventory, &Pathfinder)>,
        entities: EntityFinder<With<AbstractMonster>>,
        targets: Query<(&MinecraftEntityId, &Position, Option<&EyeHeight>)>,
        mut container_click_events: EventWriter<ContainerClickEvent>,
        mut look_at_events: EventWriter<LookAtEvent>,
        mut attack_events: EventWriter<AttackEvent>,
    ) {
        for (entity, local_settings, game_ticks, inventory, pathfinder) in &mut query {
            if !local_settings.auto_kill.enabled {
                continue;
            }

            if let Some(_goal) = &pathfinder.goal {
                continue;
            }

            let Some(target) = entities.nearest_to_entity(entity, 3.2) else {
                continue;
            };

            let Ok((target_id, target_pos, target_eye_height)) = targets.get(target) else {
                continue;
            };

            let mut position = **target_pos;
            if let Some(eye_height) = target_eye_height {
                position.y += f64::from(**eye_height);
            }

            look_at_events.send(LookAtEvent { entity, position });

            if game_ticks.0 % local_settings.auto_kill.delay_ticks != 0 {
                continue;
            }

            let held_kind = inventory.held_item().kind();
            if local_settings.auto_kill.auto_weapon && !WEAPON_ITEMS.contains_key(&held_kind) {
                let mut weapon_slots = Vec::new();

                for slot in inventory.inventory_menu.player_slots_range() {
                    let Some(item) = inventory.inventory_menu.slot(slot) else {
                        continue;
                    };

                    if let Some(damage) = WEAPON_ITEMS.get(&item.kind()) {
                        weapon_slots.push((slot, *damage));
                    }
                }

                weapon_slots.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(Ordering::Equal));

                if let Some((slot, _)) = weapon_slots.first() {
                    debug!(
                        "Swapping Weapon from {slot} to {}",
                        inventory.selected_hotbar_slot
                    );
                    container_click_events.send(ContainerClickEvent {
                        entity,
                        window_id: inventory.id,
                        operation: ClickOperation::Swap(SwapClick {
                            source_slot: u16::try_from(*slot).unwrap(),
                            target_slot: inventory.selected_hotbar_slot,
                        }),
                    });
                }
            }

            attack_events.send(AttackEvent {
                entity,
                target: *target_id,
            });
        }
    }
}

pub static WEAPON_ITEMS: LazyLock<HashMap<Item, i32>> = LazyLock::new(|| {
    HashMap::from([
        (Item::DiamondAxe, 9),
        (Item::DiamondSword, 7),
        (Item::GoldenAxe, 7),
        (Item::GoldenSword, 4),
        (Item::IronAxe, 9),
        (Item::IronSword, 6),
        (Item::NetheriteAxe, 10),
        (Item::NetheriteSword, 8),
        (Item::StoneAxe, 9),
        (Item::StoneSword, 5),
        (Item::WoodenAxe, 7),
        (Item::WoodenSword, 4),
    ])
});
