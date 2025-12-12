use std::{cmp::Ordering, collections::HashMap, sync::LazyLock};

use azalea::{
    app::{App, Plugin},
    ecs::prelude::*,
    entity::{inventory::Inventory, metadata::Player, LocalEntity, LookDirection},
    inventory::{
        operations::{ClickOperation, SwapClick},
        ContainerClickEvent,
        InventorySystems,
    },
    local_player::Hunger,
    mining::continue_mining_block,
    packet::game::{handle_outgoing_packets_observer, SendGamePacketEvent},
    physics::PhysicsSystems,
    prelude::*,
    protocol::packets::game::{
        s_interact::InteractionHand,
        ServerboundGamePacket,
        ServerboundUseItem,
    },
    registry::builtin::ItemKind,
};

use crate::prelude::*;

/// Automatically eat food to avoid starving to death
pub struct AutoEatPlugin;

impl Plugin for AutoEatPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            GameTick,
            Self::handle_auto_eat
                .after(GameTickPlugin::handle_game_ticks)
                .before(handle_outgoing_packets_observer)
                .before(continue_mining_block)
                .before(InventorySystems)
                .before(PhysicsSystems),
        );
    }
}

type QueryData<'a> = (
    Entity,
    &'a GameTicks,
    &'a Hunger,
    &'a Inventory,
    &'a LookDirection,
    &'a LocalSettings,
);
type QueryFilter = (With<Player>, With<LocalEntity>);

impl AutoEatPlugin {
    /// # Panics
    /// Will panic when the slot larger than u16 (impossible?)
    pub fn handle_auto_eat(mut query: Query<QueryData, QueryFilter>, mut commands: Commands) {
        for (entity, game_ticks, hunger, inventory, direction, local_settings) in &mut query {
            if !local_settings.auto_eat.enabled {
                continue;
            }

            if hunger.food >= 18 {
                continue;
            }

            if game_ticks.0 % local_settings.auto_eat.delay_ticks != 0 {
                continue;
            }

            if !FOOD_ITEMS.contains_key(&inventory.held_item().kind()) {
                let mut food_slots = Vec::new();

                for slot in inventory.inventory_menu.player_slots_range() {
                    let Some(item) = inventory.inventory_menu.slot(slot) else {
                        continue;
                    };

                    if let Some((nutrition, saturation)) = FOOD_ITEMS.get(&item.kind()) {
                        food_slots.push((slot, *nutrition, *saturation));
                    }
                }

                food_slots.sort_by(|a, b| {
                    b.2.partial_cmp(&a.2)
                        .unwrap_or(Ordering::Equal)
                        .then_with(|| b.1.cmp(&a.1))
                });

                if let Some((slot, _, _)) = food_slots.first() {
                    debug!(
                        "Swapping Food from {slot} to {}",
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

            let packet = ServerboundGamePacket::UseItem(ServerboundUseItem {
                hand:  InteractionHand::MainHand,
                x_rot: direction.x_rot(),
                y_rot: direction.y_rot(),
                seq:   0,
            });

            commands.trigger(SendGamePacketEvent {
                sent_by: entity,
                packet,
            });
        }
    }
}

pub static FOOD_ITEMS: LazyLock<HashMap<ItemKind, (i32, f32)>> = LazyLock::new(|| {
    HashMap::from([
        (ItemKind::Apple, (4, 2.4)),
        (ItemKind::BakedPotato, (5, 6.0)),
        (ItemKind::Beef, (3, 1.8)),
        (ItemKind::Beetroot, (1, 1.2)),
        (ItemKind::BeetrootSoup, (6, 7.2)),
        (ItemKind::Bread, (5, 6.0)),
        (ItemKind::Carrot, (3, 3.6)),
        (ItemKind::Chicken, (2, 1.2)),
        (ItemKind::Cod, (2, 0.4)),
        (ItemKind::CookedBeef, (8, 12.8)),
        (ItemKind::CookedChicken, (6, 7.2)),
        (ItemKind::CookedCod, (5, 6.0)),
        (ItemKind::CookedMutton, (6, 9.6)),
        (ItemKind::CookedPorkchop, (8, 12.8)),
        (ItemKind::CookedRabbit, (5, 6.0)),
        (ItemKind::CookedSalmon, (6, 9.6)),
        (ItemKind::Cookie, (2, 0.4)),
        (ItemKind::DriedKelp, (1, 0.6)),
        (ItemKind::EnchantedGoldenApple, (4, 9.6)),
        (ItemKind::GlowBerries, (2, 0.4)),
        (ItemKind::GoldenApple, (4, 9.6)),
        (ItemKind::GoldenCarrot, (6, 14.4)),
        (ItemKind::HoneyBottle, (6, 1.2)),
        (ItemKind::MelonSlice, (2, 1.2)),
        (ItemKind::MushroomStew, (6, 7.2)),
        (ItemKind::Mutton, (2, 1.2)),
        (ItemKind::Porkchop, (3, 1.8)),
        (ItemKind::Potato, (1, 0.6)),
        (ItemKind::PumpkinPie, (8, 4.8)),
        (ItemKind::Rabbit, (3, 1.8)),
        (ItemKind::RabbitStew, (10, 12.0)),
        (ItemKind::Salmon, (2, 0.4)),
        (ItemKind::SweetBerries, (2, 0.4)),
        (ItemKind::TropicalFish, (1, 0.2)),
    ])
});
