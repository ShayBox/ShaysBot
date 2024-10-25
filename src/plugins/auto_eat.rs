use azalea::{
    app::{App, Plugin, Update},
    chunks::handle_chunk_batch_finished_event,
    ecs::prelude::*,
    entity::{clamp_look_direction, metadata::Player, LocalEntity, LookDirection},
    inventory::{
        handle_container_click_event,
        operations::{ClickOperation, SwapClick},
        ContainerClickEvent,
        Inventory,
        InventorySet,
    },
    packet_handling::game::{handle_send_packet_event, SendPacketEvent},
    protocol::packets::game::{
        serverbound_interact_packet::InteractionHand,
        serverbound_use_item_packet::ServerboundUseItemPacket,
        ServerboundGamePacket,
    },
    registry::Item,
    Hunger,
};

use crate::plugins::auto_totem::handle_auto_totem;

pub struct AutoEatPlugin;

impl Plugin for AutoEatPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            handle_auto_eat
                .after(clamp_look_direction)
                .before(handle_chunk_batch_finished_event)
                .before(handle_container_click_event)
                .before(handle_send_packet_event)
                .before(handle_auto_totem)
                .before(InventorySet),
        );
    }
}

type QueryData<'a> = (Entity, &'a Hunger, &'a Inventory, &'a LookDirection);
type QueryFilter = (With<Player>, With<LocalEntity>);

#[allow(clippy::needless_pass_by_value)]
pub fn handle_auto_eat(
    mut query: Query<QueryData, QueryFilter>,
    mut packet_events: EventWriter<SendPacketEvent>,
    mut container_click_events: EventWriter<ContainerClickEvent>,
) {
    for (entity, hunger, inventory, direction) in &mut query {
        if hunger.food >= 18 {
            continue;
        }

        if !FOOD_ITEMS.contains(&inventory.held_item().kind()) {
            inventory
                .inventory_menu
                .slots()
                .into_iter()
                .enumerate()
                .filter_map(|(index, slot)| {
                    if FOOD_ITEMS.contains(&slot.kind()) {
                        Some(ContainerClickEvent {
                            entity,
                            window_id: inventory.id,
                            operation: ClickOperation::Swap(SwapClick {
                                source_slot: u16::try_from(index).ok()?,
                                target_slot: inventory.selected_hotbar_slot,
                            }),
                        })
                    } else {
                        None
                    }
                })
                .for_each(|event| {
                    container_click_events.send(event);
                });
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
