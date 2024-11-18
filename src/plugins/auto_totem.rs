use azalea::{
    app::{App, Plugin},
    ecs::prelude::*,
    entity::{metadata::Player, LocalEntity},
    inventory::{
        handle_container_click_event,
        operations::{ClickOperation, SwapClick},
        ContainerClickEvent,
        Inventory,
        Menu,
    },
    prelude::*,
    registry::Item,
};

use crate::{plugins::prelude::*, BoundedCounter};

/// Automatically equip totems in the offhand slot
pub struct AutoTotemPlugin;

impl Plugin for AutoTotemPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            GameTick,
            handle_auto_totem
                .before(handle_container_click_event)
                .after(handle_auto_eat),
        );
    }
}

#[derive(Component, Default)]
pub struct AutoTotemCounter(BoundedCounter<u8>);

type InitQueryData = Entity;
type InitQueryFilter = (With<Player>, With<LocalEntity>, Without<AutoTotemCounter>);

type RunQueryData<'a> = (Entity, &'a Inventory, &'a mut AutoTotemCounter);
type RunQueryFilter = (With<Player>, With<LocalEntity>, With<AutoTotemCounter>);

pub fn handle_auto_totem(
    mut init_query: Query<InitQueryData, InitQueryFilter>,
    mut commands: Commands,

    mut run_query: Query<RunQueryData, RunQueryFilter>,
    mut container_click_events: EventWriter<ContainerClickEvent>,
) {
    for entity in &mut init_query {
        commands.entity(entity).insert(AutoTotemCounter::default());
    }

    for (entity, inventory, mut counter) in &mut run_query {
        let Some(ticks) = counter.0.next() else {
            return;
        };

        if ticks % 2 == 0 {
            continue;
        }

        let Menu::Player(player) = &inventory.inventory_menu else {
            continue;
        };

        if player.offhand.kind() == Item::TotemOfUndying {
            continue;
        }

        let Some(index) = inventory.menu().slots()[8..]
            .iter()
            .position(|slot| slot.kind() == Item::TotemOfUndying)
            .and_then(|index| u16::try_from(index + 8).ok())
        else {
            continue;
        };

        container_click_events.send(ContainerClickEvent {
            entity,
            window_id: inventory.id,
            operation: ClickOperation::Swap(SwapClick {
                source_slot: index,
                target_slot: 40,
            }),
        });
    }
}
