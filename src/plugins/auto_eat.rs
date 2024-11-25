use azalea::{
    app::{App, Plugin},
    ecs::prelude::*,
    entity::{metadata::Player, LocalEntity, LookDirection},
    inventory::{
        operations::{ClickOperation, SwapClick},
        ContainerClickEvent,
        Inventory,
        InventorySet,
    },
    mining::continue_mining_block,
    packet_handling::game::{handle_send_packet_event, SendPacketEvent},
    physics::PhysicsSet,
    prelude::*,
    protocol::packets::game::{
        serverbound_interact_packet::InteractionHand,
        serverbound_use_item_packet::ServerboundUseItemPacket,
        ServerboundGamePacket,
    },
    registry::Item,
    Hunger,
};

use crate::{handle_game_tick, GameTicks};

/// Automatically eat any food item when below 18 hunger
pub struct AutoEatPlugin;

impl Plugin for AutoEatPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            GameTick,
            handle_auto_eat
                .after(handle_game_tick)
                .before(handle_send_packet_event)
                .before(continue_mining_block)
                .before(InventorySet)
                .before(PhysicsSet),
        );
    }
}

type QueryData<'a> = (
    Entity,
    &'a GameTicks,
    &'a Hunger,
    &'a Inventory,
    &'a LookDirection,
);
type QueryFilter = (With<Player>, With<LocalEntity>);

pub fn handle_auto_eat(
    mut query: Query<QueryData, QueryFilter>,
    mut packet_events: EventWriter<SendPacketEvent>,
    mut container_click_events: EventWriter<ContainerClickEvent>,
) {
    const EAT_DELAY: u128 = 32 + 2; /* Extra Delay: Fix de-sync */

    for (entity, game_ticks, hunger, inventory, direction) in &mut query {
        if hunger.food >= 18 {
            continue;
        }

        if game_ticks.0 % EAT_DELAY != 0 {
            continue;
        }

        if !FOOD_ITEMS.contains(&inventory.held_item().kind()) {
            /* Try to find food in the inventory and swap to it */
            let mut slots = inventory
                .inventory_menu
                .slots()
                .into_iter()
                .enumerate()
                .filter_map(|(index, slot)| {
                    if FOOD_ITEMS.contains(&slot.kind()) {
                        Some(u16::try_from(index).ok()?)
                    } else {
                        None
                    }
                });

            if let Some(slot) = slots.next() {
                container_click_events.send(ContainerClickEvent {
                    entity,
                    window_id: inventory.id,
                    operation: ClickOperation::Swap(SwapClick {
                        source_slot: slot,
                        target_slot: inventory.selected_hotbar_slot,
                    }),
                });
            } else {
                info!("[AutoEat] Missing Food!");
                continue;
            }
        }

        let packet = ServerboundGamePacket::UseItem(ServerboundUseItemPacket {
            hand:     InteractionHand::MainHand,
            pitch:    direction.x_rot,
            yaw:      direction.y_rot,
            sequence: 0,
        });

        packet_events.send(SendPacketEvent { entity, packet });
    }
}

pub const FOOD_ITEMS: [Item; 32] = [
    Item::Apple,
    Item::BakedPotato,
    Item::Beef,
    Item::Beetroot,
    Item::BeetrootSoup,
    Item::Bread,
    Item::Carrot,
    Item::Chicken,
    Item::Cod,
    Item::CookedBeef,
    Item::CookedCod,
    Item::CookedMutton,
    Item::CookedPorkchop,
    Item::CookedSalmon,
    Item::Cookie,
    Item::DriedKelp,
    Item::EnchantedGoldenApple,
    Item::GlowBerries,
    Item::GoldenApple,
    Item::GoldenCarrot,
    Item::HoneyBottle,
    Item::MelonSlice,
    Item::MushroomStem,
    Item::Mutton,
    Item::Porkchop,
    Item::Potato,
    Item::PumpkinPie,
    Item::Rabbit,
    Item::RabbitStew,
    Item::Salmon,
    Item::SweetBerries,
    Item::TropicalFish,
];
