use azalea::{
    app::{App, Plugin, Update},
    ecs::prelude::*,
    entity::{metadata::Player, LocalEntity},
    inventory::{
        operations::{ClickOperation, SwapClick},
        ContainerClickEvent,
        Inventory,
        InventorySet,
        Menu,
    },
    registry::Item,
};

use crate::plugins::auto_eat::handle_auto_eat;

pub struct AutoTotemPlugin;

impl Plugin for AutoTotemPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            handle_auto_totem
                .after(handle_auto_eat)
                .before(InventorySet),
        );
    }
}

type QueryData<'a> = (Entity, &'a Inventory);
type QueryFilter = (With<Player>, With<LocalEntity>);

#[allow(clippy::needless_pass_by_value)]
pub fn handle_auto_totem(
    mut query: Query<QueryData, QueryFilter>,
    mut container_click_events: EventWriter<ContainerClickEvent>,
) {
    for (entity, inventory) in &mut query {
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
