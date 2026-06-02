use std::{
    collections::HashMap,
    sync::{atomic::AtomicUsize, Arc},
};

use azalea::{
    app::{App, Plugin, Update},
    block::BlockTrait,
    disconnect::DisconnectEvent,
    ecs::prelude::*,
    local_player::TabList,
    packet::{game::ReceiveGamePacketEvent, login::ReceiveLoginPacketEvent},
    player::GameProfileComponent,
    prelude::Resource,
    protocol::packets::game::ClientboundGamePacket,
    registry::builtin::EntityKind,
};
use serenity::json::json;

use crate::prelude::*;

/// Event type identifiers for webhook logging.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum EventType {
    PlayerJoin,
    PlayerLeave,
    PlayerEnter,
    PlayerExit,
    PlayerCommand,
    PlayerPearl,
    PlayerBreak,
    PlayerPlace,
}

/// A client that sends webhook messages with round-robin URL selection.
#[derive(Resource)]
pub struct WebhookClient {
    /// Per-event-type configuration.
    configs:      Arc<HashMap<EventType, EventConfig>>,
    /// Global fallback URLs (used when event-specific webhooks is empty).
    global_urls:  Arc<Vec<String>>,
    /// Round-robin index for the global URL list.
    global_index: Arc<AtomicUsize>,
}

#[derive(Clone)]
struct EventConfig {
    enabled: bool,
    urls:    Vec<String>,
    /// Optional custom block filter for break/place events.
    blocks:  Option<Vec<String>>,
}

impl WebhookClient {
    fn new(configs: HashMap<EventType, EventConfig>, global_urls: Vec<String>) -> Self {
        Self {
            configs:      Arc::new(configs),
            global_urls:  Arc::new(global_urls),
            global_index: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Get the URLs to use for a given event type, falling back to global URLs.
    fn get_urls(&self, event_type: EventType) -> Option<Arc<Vec<String>>> {
        let config = self.configs.get(&event_type)?;
        if !config.enabled {
            return None;
        }

        // Use event-specific URLs if configured, otherwise fall back to global
        Some(if config.urls.is_empty() {
            Arc::clone(&self.global_urls)
        } else {
            Arc::new(config.urls.clone())
        })
    }

    /// Get the custom block filter for a given event type.
    fn get_blocks(&self, event_type: EventType) -> Option<Vec<String>> {
        self.configs.get(&event_type).and_then(|c| c.blocks.clone())
    }

    /// Send a message to Discord via webhook (non-blocking).
    fn send(&self, event_type: EventType, content: String) {
        self.send_with_block(event_type, &content, None::<&str>, None);
    }

    /// Send a block-related message with custom block filtering.
    fn send_with_block(
        &self,
        event_type: EventType,
        content: &str,
        block_id: Option<&str>,
        blocks: Option<&[String]>,
    ) {
        let urls = match self.get_urls(event_type) {
            Some(urls) if !urls.is_empty() => urls,
            _ => return, /* Disabled or no URLs */
        };

        // Check block filter for break/place events
        if let (Some(block_id), Some(block_list)) = (block_id, blocks) {
            if !is_logged_block(block_id, Some(block_list)) {
                return; /* Block not in filter list */
            }
        }

        let idx = self
            .global_index
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let url = urls[idx % urls.len()].clone();
        let content = content.to_string();

        tokio::task::spawn_local({
            async move {
                if let Err(e) = Self::post_webhook(&url, &content) {
                    error!("Failed to send webhook: {e}");
                }
            }
        });
    }

    fn post_webhook(url: &str, content: &str) -> anyhow::Result<()> {
        let payload = WebhookPayload { content };
        ureq::post(url).send_json(json!(payload))?;

        Ok(())
    }
}

/// Log events to Discord via webhooks with round-robin URL distribution.
pub struct LoggerPlugin;

impl Plugin for LoggerPlugin {
    fn build(&self, app: &mut App) {
        let global_settings = match app.world().get_resource::<GlobalSettings>() {
            Some(settings) => settings.clone(),
            None => return,
        };

        let config = &global_settings.logger;
        if config.webhooks.is_empty() {
            return; /* No webhooks configured */
        }

        let event_types = &config.event;

        let mut event_configs: HashMap<EventType, EventConfig> = HashMap::new();

        // Player join events
        event_configs.insert(
            EventType::PlayerJoin,
            EventConfig {
                enabled: event_types.player_join.enabled,
                urls:    event_types.player_join.webhooks.clone().unwrap_or_default(),
                blocks:  None,
            },
        );

        // Player leave events
        event_configs.insert(
            EventType::PlayerLeave,
            EventConfig {
                enabled: event_types.player_leave.enabled,
                urls:    event_types
                    .player_leave
                    .webhooks
                    .clone()
                    .unwrap_or_default(),
                blocks:  None,
            },
        );

        // Player visual range enter events (add_entity + player_info_update)
        event_configs.insert(
            EventType::PlayerEnter,
            EventConfig {
                enabled: event_types.player_enter.enabled,
                urls:    event_types
                    .player_enter
                    .webhooks
                    .clone()
                    .unwrap_or_default(),
                blocks:  None,
            },
        );

        // Player visual range exit events (remove_entities + player_info_remove)
        event_configs.insert(
            EventType::PlayerExit,
            EventConfig {
                enabled: event_types.player_exit.enabled,
                urls:    event_types.player_exit.webhooks.clone().unwrap_or_default(),
                blocks:  None,
            },
        );

        // Command events
        event_configs.insert(
            EventType::PlayerCommand,
            EventConfig {
                enabled: event_types.player_command.enabled,
                urls:    event_types
                    .player_command
                    .webhooks
                    .clone()
                    .unwrap_or_default(),
                blocks:  None,
            },
        );

        // Pearl events
        event_configs.insert(
            EventType::PlayerPearl,
            EventConfig {
                enabled: event_types.player_pearl.enabled,
                urls:    event_types
                    .player_pearl
                    .webhooks
                    .clone()
                    .unwrap_or_default(),
                blocks:  None,
            },
        );

        // Block break events
        event_configs.insert(
            EventType::PlayerBreak,
            EventConfig {
                enabled: event_types.player_break.enabled,
                urls:    event_types
                    .player_break
                    .webhooks
                    .clone()
                    .unwrap_or_default(),
                blocks:  event_types.player_break.blocks.clone(),
            },
        );

        // Block place events
        event_configs.insert(
            EventType::PlayerPlace,
            EventConfig {
                enabled: event_types.player_place.enabled,
                urls:    event_types
                    .player_place
                    .webhooks
                    .clone()
                    .unwrap_or_default(),
                blocks:  event_types.player_place.blocks.clone(),
            },
        );

        app.insert_resource(WebhookClient::new(event_configs, config.webhooks.clone()))
            .add_systems(
                Update,
                (
                    Self::handle_add_entity_packets,
                    Self::handle_block_break_packets,
                    Self::handle_block_update_packets,
                    Self::handle_cmd_events,
                    Self::handle_disconnect_events,
                    Self::handle_login_packets,
                    Self::handle_pearl_goto_events,
                    Self::handle_player_info_remove_packets,
                    Self::handle_player_info_update_packets,
                    Self::handle_remove_entities_packets,
                ),
            );
    }
}

impl LoggerPlugin {
    fn handle_add_entity_packets(
        mut packet_events: MessageReader<ReceiveGamePacketEvent>,
        query: Query<(&TabList, &GameProfileComponent)>,
        webhook: Option<Res<WebhookClient>>,
    ) {
        let Some(webhook) = webhook else {
            return;
        };

        for event in packet_events.read() {
            let ClientboundGamePacket::AddEntity(packet) = event.packet.as_ref() else {
                continue;
            };
            if packet.entity_type != EntityKind::Player {
                continue;
            }

            let Ok((tab_list, game_profile)) = query.get(event.entity) else {
                continue;
            };

            let Some((_, player_info)) = tab_list.iter().find(|(uuid, _)| uuid == &&packet.uuid)
            else {
                continue;
            };

            let username = game_profile.name.clone();
            let player_name = player_info.profile.name.clone();
            let content = format!("[{username}] {player_name} has entered visual range.");
            webhook.send(EventType::PlayerEnter, content);
        }
    }

    fn handle_block_break_packets(
        mut packet_events: MessageReader<ReceiveGamePacketEvent>,
        query: Query<(&BlockStates, &GameProfileComponent)>,
        webhook: Option<Res<WebhookClient>>,
    ) {
        let Some(webhook) = webhook else {
            return;
        };

        for event in packet_events.read() {
            let ClientboundGamePacket::BlockDestruction(packet) = event.packet.as_ref() else {
                continue;
            };

            let Ok((block_states, game_profile)) = query.get(event.entity) else {
                continue;
            };

            let Some(block_state) = block_states.0.get(&packet.pos) else {
                continue;
            };
            let block = Box::<dyn BlockTrait>::from(*block_state);
            if !is_logged_block(&block.id(), None) {
                continue;
            }

            let block_name = block.as_block_kind();
            let username = game_profile.name.clone();
            let content = format!("[{username}] {block_name:?} was mined in visual range.");

            let blocks = webhook.get_blocks(EventType::PlayerBreak);
            if !is_logged_block(&block.id(), blocks.as_deref()) {
                continue;
            }

            webhook.send_with_block(
                EventType::PlayerBreak,
                &content,
                Some(&block.id()),
                blocks.as_ref().map(|b| b.as_slice()),
            );
        }
    }

    fn handle_block_update_packets(
        mut packet_events: MessageReader<ReceiveGamePacketEvent>,
        query: Query<&GameProfileComponent>,
        webhook: Option<Res<WebhookClient>>,
    ) {
        let Some(webhook) = webhook else {
            return;
        };

        for event in packet_events.read() {
            let ClientboundGamePacket::BlockUpdate(packet) = event.packet.as_ref() else {
                continue;
            };

            let Ok(game_profile) = query.get(event.entity) else {
                continue;
            };

            let block = Box::<dyn BlockTrait>::from(packet.block_state);
            if !is_logged_block(&block.id(), None) {
                continue;
            }

            let block_name = block.as_block_kind();
            let username = game_profile.name.clone();
            let content = format!("[{username}] {block_name:?} was placed in visual range.");

            let blocks = webhook.get_blocks(EventType::PlayerPlace);
            if !is_logged_block(&block.id(), blocks.as_deref()) {
                continue;
            }

            webhook.send_with_block(
                EventType::PlayerPlace,
                &content,
                Some(&block.id()),
                blocks.as_ref().map(|b| b.as_slice()),
            );
        }
    }

    fn handle_cmd_events(
        mut cmd_events: MessageReader<CmdEvent>,
        query: Query<&GameProfileComponent>,
        webhook: Option<Res<WebhookClient>>,
    ) {
        let Some(webhook) = webhook else {
            return;
        };

        for event in cmd_events.read() {
            let Some(entity) = event.entity else {
                continue;
            };
            let Ok(game_profile) = query.get(entity) else {
                continue;
            };

            let username = game_profile.name.clone();
            let sender = match &event.sender {
                CmdSender::Minecraft(_) => "console".to_string(),
                #[cfg(feature = "bot")]
                CmdSender::Discord(user_id) => format!("discord:{user_id}"),
                #[cfg(feature = "api")]
                CmdSender::ApiServer(_) => "api".to_string(),
            };

            let cmd_name = match event.cmd {
                Cmds::Interact(_) => "/interact",
                Cmds::Join(_) => "/join",
                Cmds::Leave(_) => "/leave",
                Cmds::Pearl(_) => "/pearl",
                Cmds::Playtime(_) => "/playtime",
                Cmds::Seen(_) => "/seen",
                Cmds::Whitelist(_) => "/whitelist",
            };

            let args: String = event.args.iter().cloned().collect::<Vec<_>>().join(" ");
            let content = if args.is_empty() {
                format!("[{username}] ran `{cmd_name}` ({sender})")
            } else {
                format!("[{username}] ran `{cmd_name} {args}` ({sender})")
            };
            webhook.send(EventType::PlayerCommand, content);
        }
    }

    fn handle_disconnect_events(
        mut events: MessageReader<DisconnectEvent>,
        query: Query<&GameProfileComponent>,
        webhook: Option<Res<WebhookClient>>,
    ) {
        let Some(webhook) = webhook else {
            return;
        };

        for event in events.read() {
            let Ok(game_profile) = query.get(event.entity) else {
                continue;
            };

            let username = game_profile.name.clone();
            let content = match &event.reason {
                Some(reason) if reason.to_string().starts_with(ZENITH_PREFIX) => {
                    format!("[{username}] Disabling AutoReconnect")
                }
                Some(reason) => format!("[{username}] left the game. | {}", reason.to_string()),
                None => format!("[{username}] disconnected."),
            };
            webhook.send(EventType::PlayerLeave, content);
        }
    }

    fn handle_login_packets(
        mut events: MessageReader<ReceiveLoginPacketEvent>,
        query: Query<&GameProfileComponent>,
        webhook: Option<Res<WebhookClient>>,
    ) {
        let Some(webhook) = webhook else {
            return;
        };

        for event in events.read() {
            let Ok(game_profile) = query.get(event.entity) else {
                continue;
            };

            let username = &game_profile.name;
            let content = format!("[{username}] joined the game.");
            webhook.send(EventType::PlayerJoin, content);
        }
    }

    fn handle_pearl_goto_events(
        mut pearl_events: MessageReader<PearlGotoEvent>,
        query: Query<(&GameProfileComponent, &LocalSettings)>,
        stasis_chambers: Res<StasisChambers>,
        webhook: Option<Res<WebhookClient>>,
    ) {
        let Some(webhook) = webhook else {
            return;
        };

        for event in pearl_events.read() {
            let Ok((game_profile, local_settings)) = query.get(event.entity) else {
                continue;
            };

            let username = game_profile.name.clone();
            let location = &local_settings.auto_pearl.location;
            let count = stasis_chambers
                .0
                .values()
                .filter(|c| c.owner_uuid == event.0.owner_uuid)
                .count();

            let content = if count <= local_settings.auto_pearl.pearl_limit {
                format!("[{username}] pearl at `{location}` pulled ({count} remaining)")
            } else {
                format!(
                    "[{username}] pearl at `{location}` pulled (over limit: {count}/{})",
                    local_settings.auto_pearl.pearl_limit
                )
            };
            webhook.send(EventType::PlayerPearl, content);
        }
    }

    fn handle_player_info_remove_packets(
        mut packet_events: MessageReader<ReceiveGamePacketEvent>,
        query: Query<(&PlayerProfiles, &GameProfileComponent)>,
        webhook: Option<Res<WebhookClient>>,
    ) {
        let Some(webhook) = webhook else {
            return;
        };

        for event in packet_events.read() {
            let ClientboundGamePacket::PlayerInfoRemove(packet) = event.packet.as_ref() else {
                continue;
            };

            let Ok((player_profiles, game_profile)) = query.get(event.entity) else {
                continue;
            };

            for profile_uuid in &packet.profile_ids {
                let Some((_, player_profile)) = player_profiles
                    .0
                    .iter()
                    .find(|(_, profile)| profile.uuid == *profile_uuid)
                else {
                    continue;
                };

                let username = game_profile.name.clone();
                let player_name = player_profile.name.clone();
                let content = format!("[{username}] {player_name} logged out in visual range.");
                webhook.send(EventType::PlayerExit, content);
            }
        }
    }

    fn handle_player_info_update_packets(
        mut packet_events: MessageReader<ReceiveGamePacketEvent>,
        query: Query<(&PlayerProfiles, &GameProfileComponent)>,
        webhook: Option<Res<WebhookClient>>,
    ) {
        let Some(webhook) = webhook else {
            return;
        };

        for event in packet_events.read().cloned() {
            let ClientboundGamePacket::PlayerInfoUpdate(packet) = event.packet.as_ref() else {
                continue;
            };
            if !packet.actions.add_player {
                continue;
            }

            let Ok((player_profiles, game_profile)) = query.get(event.entity) else {
                continue;
            };

            let profiles: Vec<_> = packet
                .entries
                .clone()
                .into_iter()
                .filter_map(|info| {
                    player_profiles.0.iter().find(|(_, profile)| {
                        profile.uuid == info.profile.uuid && info.display_name.is_some()
                    })
                })
                .collect();

            for (_, player_profile) in profiles {
                let username = game_profile.name.clone();
                let player_name = player_profile.name.clone();
                let content = format!("[{username}] {player_name} joined in visual range.");
                webhook.send(EventType::PlayerEnter, content);
            }
        }
    }

    fn handle_remove_entities_packets(
        mut packet_events: MessageReader<ReceiveGamePacketEvent>,
        query: Query<(&PlayerProfiles, &GameProfileComponent)>,
        webhook: Option<Res<WebhookClient>>,
    ) {
        let Some(webhook) = webhook else {
            return;
        };

        for event in packet_events.read() {
            let ClientboundGamePacket::RemoveEntities(packet) = event.packet.as_ref() else {
                continue;
            };

            let Ok((player_profiles, game_profile)) = query.get(event.entity) else {
                continue;
            };

            for entity_id in &packet.entity_ids {
                let Some(player_profile) = player_profiles.0.get(entity_id) else {
                    continue;
                };

                let username = game_profile.name.clone();
                let player_name = player_profile.name.clone();
                let content = format!("[{username}] {player_name} has exited visual range.");
                webhook.send(EventType::PlayerExit, content);
            }
        }
    }
}

#[derive(serde::Serialize)]
struct WebhookPayload<'a> {
    content: &'a str,
}
