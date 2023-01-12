use std::sync::{atomic::Ordering, Arc};

use anyhow::Result;
use azalea::{Client, Event};
use azalea_protocol::packets::game::{
    clientbound_player_combat_kill_packet::ClientboundPlayerCombatKillPacket,
    serverbound_client_command_packet::{Action::PerformRespawn, ServerboundClientCommandPacket},
};
use serenity::futures::future;

use crate::{chat::handle_chat, packet::handle_packet, State};

pub async fn handle(client: Client, event: Event, state: State) -> Result<()> {
    match event {
        Event::Chat(chat) => handle_chat(client, chat, state).await?,
        Event::Death(death) => handle_death(client, death, state).await?,
        Event::Init => println!("Minecraft Bot Initialized"),
        Event::Login => println!("Minecraft Bot Logged in"),
        Event::Packet(packet) => handle_packet(client, packet, state).await?,
        Event::Tick => handle_tick(client, state).await?,
        _ => {}
    }

    Ok(())
}

async fn handle_death(
    client: Client,
    _death: Option<Arc<ClientboundPlayerCombatKillPacket>>,
    _state: State,
) -> Result<()> {
    let respawn_command_packet = ServerboundClientCommandPacket {
        action: PerformRespawn,
    };
    client.write_packet(respawn_command_packet.get()).await?;

    Ok(())
}

pub async fn handle_tick(client: Client, state: State) -> Result<()> {
    // Anti-Spam Kick - https://github.com/mat-1/potato-bot-2
    let _ = state
        .spam_tick
        .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |size| {
            if size > 0 {
                Some(size - 1)
            } else {
                None
            }
        });

    let messages = {
        let messages = &mut state.mc_queue.lock().unwrap();
        let max_drain = (100 - state.spam_tick.load(Ordering::SeqCst)) / 20;
        let len = messages.len();
        messages
            .drain(..max_drain.min(len))
            .collect::<Vec<String>>()
    };

    if !messages.is_empty() {
        let mut futures = Vec::new();

        state
            .spam_tick
            .fetch_add(messages.len() * 20, Ordering::SeqCst);

        for message in messages {
            if message.len() > 256 {
                continue;
            }

            futures.push(async {
                let message = message;
                client.chat(&message).await
            })
        }

        future::join_all(futures).await;
    }

    Ok(())
}
