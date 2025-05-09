use azalea::{
    app::{App, Plugin},
    ecs::prelude::*,
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

use crate::prelude::*;

/// Automatically equip totems of undying to avoid dying
pub struct AutoTotemPlugin;

impl Plugin for AutoTotemPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            GameTick,
            Self::handle_auto_totem
                .after(AutoEatPlugin::handle_auto_eat)
                .after(GameTickPlugin::handle_game_ticks)
                .before(handle_container_click_event),
        );
    }
}

impl AutoTotemPlugin {
    pub fn handle_auto_totem(
        mut query: Query<(Entity, &Inventory, &GameTicks)>,
        mut container_click_events: EventWriter<ContainerClickEvent>,
    ) {
        for (entity, inventory, game_ticks) in &mut query {
            if game_ticks.0 % 2 == 0 {
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

            container_click_events.write(ContainerClickEvent {
                entity,
                window_id: inventory.id,
                operation: ClickOperation::Swap(SwapClick {
                    source_slot: index,
                    target_slot: 40,
                }),
            });
        }
    }
}
