use std::{cmp::Ordering, sync::LazyLock};

use azalea::{
    app::{App, Plugin},
    attack::AttackEvent,
    bot::LookAtEvent,
    ecs::prelude::*,
    entity::{
        dimensions::EntityDimensions,
        inventory::Inventory,
        metadata::AbstractMonster,
        Position,
    },
    inventory::{
        operations::{ClickOperation, SwapClick},
        ContainerClickEvent,
    },
    nearest_entity::EntityFinder,
    pathfinder::Pathfinder,
    physics::PhysicsSystems,
    prelude::*,
    registry::builtin::ItemKind,
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
                .before(PhysicsSystems),
        );
    }
}

impl AutoKillPlugin {
    /// # Panics
    /// Will panic if ?
    pub fn handle_auto_kill(
        mut query: Query<(Entity, &LocalSettings, &GameTicks, &Inventory, &Pathfinder)>,
        entities: EntityFinder<With<AbstractMonster>>,
        targets: Query<(&Position, Option<&EntityDimensions>)>,
        mut commands: Commands,
        mut look_at_events: MessageWriter<LookAtEvent>,
        mut attack_events: MessageWriter<AttackEvent>,
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

            let Ok((target_pos, target_entity_dimensions)) = targets.get(target) else {
                continue;
            };

            let mut position = **target_pos;
            if let Some(entity_dimensions) = target_entity_dimensions {
                position.y += f64::from(entity_dimensions.eye_height);
            }

            look_at_events.write(LookAtEvent { entity, position });

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
                    commands.trigger(ContainerClickEvent {
                        entity,
                        window_id: inventory.id,
                        operation: ClickOperation::Swap(SwapClick {
                            source_slot: u16::try_from(*slot).unwrap(),
                            target_slot: inventory.selected_hotbar_slot,
                        }),
                    });
                }
            }

            attack_events.write(AttackEvent { entity, target });
        }
    }
}

pub static WEAPON_ITEMS: LazyLock<HashMap<ItemKind, i32>> = LazyLock::new(|| {
    HashMap::from([
        (ItemKind::DiamondAxe, 9),
        (ItemKind::DiamondSword, 7),
        (ItemKind::GoldenAxe, 7),
        (ItemKind::GoldenSword, 4),
        (ItemKind::IronAxe, 9),
        (ItemKind::IronSword, 6),
        (ItemKind::NetheriteAxe, 10),
        (ItemKind::NetheriteSword, 8),
        (ItemKind::StoneAxe, 9),
        (ItemKind::StoneSword, 5),
        (ItemKind::WoodenAxe, 7),
        (ItemKind::WoodenSword, 4),
    ])
});
