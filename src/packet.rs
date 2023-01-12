use std::{borrow::Borrow, sync::Arc};

use anyhow::Result;
use azalea::Client;
use azalea_protocol::packets::game::ClientboundGamePacket;

use crate::State;

pub async fn handle_packet(
    _client: Client,
    packet: Arc<ClientboundGamePacket>,
    _state: State,
) -> Result<()> {
    match packet.borrow() {
        ClientboundGamePacket::Disconnect(disconnect) => {
            println!("Disconnected: {}", disconnect.reason)
        }
        ClientboundGamePacket::SetTime(_) => {}
        _ => {}
    }

    Ok(())
}
