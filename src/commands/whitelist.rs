use azalea::{
    app::{App, Plugin, Update},
    ecs::prelude::*,
    PlayerInfo,
    TabList,
};
#[cfg(feature = "discord")]
use serde::Deserialize;
use uuid::Uuid;

use crate::prelude::*;

/// Whitelist Players and link their Discord.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WhitelistCommandPlugin;

impl ChatCmd for WhitelistCommandPlugin {
    fn aliases(&self) -> Vec<&'static str> {
        vec!["whitelist"]
    }
}

impl Plugin for WhitelistCommandPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            Self::handle_whitelist_command_events
                .ambiguous_with_all()
                .before(MinecraftChatPlugin::handle_send_whisper_events)
                .after(MinecraftChatPlugin::handle_chat_received_events),
        );
    }
}

impl WhitelistCommandPlugin {
    /// # Panics
    /// Will panic if `DeriveTomlConfig::save` fails.
    pub fn handle_whitelist_command_events(
        mut command_events: EventReader<CommandEvent>,
        mut whisper_events: EventWriter<WhisperEvent>,
        mut settings: ResMut<GlobalSettings>,
        query: Query<&TabList>,
    ) {
        if let Some(event) = command_events.read().next() {
            let ChatCmds::Whitelist(_plugin) = event.command else {
                return;
            };

            let Ok(tab_list) = query.get(event.entity) else {
                return;
            };

            let mut args = event.args.clone();
            let mut whisper_event = WhisperEvent {
                content: String::new(),
                entity:  event.entity,
                sender:  event.sender,
                source:  event.source.clone(),
                status:  200,
            };

            let Some(action) = args.pop_front() else {
                whisper_event.content = str!("Missing action | Actions: add, remove, link, & set");
                whisper_event.status = 404;
                whisper_events.send(whisper_event);
                return;
            };

            let discord_id = args.pop_front();
            let (status, content) = match action.as_ref() {
                "add" => handle_add(&mut settings, discord_id, tab_list),
                "link" => handle_link(&mut settings, discord_id, &event.sender),
                "pass" => handle_pass(&mut settings, discord_id, &event.sender),
                "remove" => handle_remove(&mut settings, discord_id, tab_list),
                _ => (
                    406,
                    str!("Invalid action | Actions: add, remove, link, & set"),
                ),
            };

            whisper_event.content = content;
            whisper_event.status = status;
            whisper_events.send(whisper_event);
        }

        command_events.clear();
    }
}

fn handle_add(
    settings: &mut ResMut<GlobalSettings>,
    user: Option<String>,
    tab_list: &TabList,
) -> (u16, String) {
    let Some(player_name) = user else {
        return (404, str!("Missing player name"));
    };

    let Some((uuid, info)) = try_find_player(tab_list, &player_name) else {
        return (404, str!("Player not found"));
    };

    if settings.users.contains_key(uuid) {
        (409, str!("Already whitelisted"))
    } else {
        settings.users.entry(*uuid).or_default();
        settings.clone().save().expect("Failed to save settings");

        (200, format!("Successfully added: {}", info.profile.name))
    }
}

fn handle_remove(
    settings: &mut ResMut<GlobalSettings>,
    user: Option<String>,
    tab_list: &TabList,
) -> (u16, String) {
    let Some(player_name) = user else {
        return (404, str!("Missing Minecraft player name"));
    };

    let Some((uuid, info)) = try_find_player(tab_list, &player_name) else {
        return (404, str!("Player not found"));
    };

    if settings.users.contains_key(uuid) {
        settings.users.remove(uuid);
        settings.clone().save().expect("Failed to save settings");

        (200, format!("Successfully removed: {}", info.profile.name))
    } else {
        (409, str!("Already not whitelisted"))
    }
}

fn handle_pass(
    settings: &mut ResMut<GlobalSettings>,
    api_password: Option<String>,
    sender: &CommandSender,
) -> (u16, String) {
    match sender {
        #[cfg(feature = "api")]
        CommandSender::ApiServer(uuid) => {
            let Some(api_password) = api_password else {
                return (404, str!("Missing Discord user id"));
            };

            settings
                .users
                .entry(*uuid)
                .and_modify(|user| user.api_password.clone_from(&api_password))
                .or_insert_with(|| User {
                    api_password,
                    ..Default::default()
                });
            settings.clone().save().expect("Failed to save settings");

            (200, str!("Successfully updated password"))
        }
        #[cfg(feature = "discord")]
        CommandSender::Discord(_) => (500, str!("You can't update your API password on Discord")),
        CommandSender::Minecraft(uuid) => {
            let Some(discord_id) = api_password else {
                return (404, str!("Missing Discord user id"));
            };

            settings
                .users
                .entry(*uuid)
                .and_modify(|user| user.discord_id.clone_from(&discord_id))
                .or_insert_with(|| User {
                    discord_id,
                    ..Default::default()
                });
            settings.clone().save().expect("Failed to save settings");

            (200, str!("Successfully linked"))
        }
    }
}

fn handle_link(
    settings: &mut ResMut<GlobalSettings>,
    discord_id: Option<String>,
    sender: &CommandSender,
) -> (u16, String) {
    match sender {
        #[cfg(feature = "api")]
        CommandSender::ApiServer(uuid) => {
            let Some(discord_id) = discord_id else {
                return (404, str!("Missing Discord user id"));
            };

            settings
                .users
                .entry(*uuid)
                .and_modify(|user| user.discord_id.clone_from(&discord_id))
                .or_insert_with(|| User {
                    discord_id,
                    ..Default::default()
                });
            settings.clone().save().expect("Failed to save settings");

            (200, str!("Successfully linked discord"))
        }
        #[cfg(feature = "discord")]
        CommandSender::Discord(_) => {
            let Some(discord_id) = discord_id else {
                return (404, str!("Missing auth code (Join: auth.aristois.net)"));
            };

            let path = format!("https://auth.aristois.net/token/{discord_id}");
            let Ok(response) = ureq::get(&path).call() else {
                return (406, str!("Invalid auth code (Join: auth.aristois.net)"));
            };

            let code = response.status();
            let Ok(json) = response.into_json::<Json>() else {
                return (500, str!("Failed to parse JSON"));
            };

            let Some(uuid) = json.uuid else {
                return (
                    code,
                    format!("Authentication {}: {}", json.status, json.message),
                );
            };

            settings
                .users
                .entry(uuid)
                .and_modify(|user| user.discord_id.clone_from(&discord_id))
                .or_insert_with(|| User {
                    discord_id,
                    ..Default::default()
                });
            settings.clone().save().expect("Failed to save settings");

            (200, str!("Successfully linked"))
        }
        CommandSender::Minecraft(uuid) => {
            let Some(discord_id) = discord_id else {
                return (404, str!("Missing Discord user id"));
            };

            settings
                .users
                .entry(*uuid)
                .and_modify(|user| user.discord_id.clone_from(&discord_id))
                .or_insert_with(|| User {
                    discord_id,
                    ..Default::default()
                });
            settings.clone().save().expect("Failed to save settings");

            (200, str!("Successfully linked"))
        }
    }
}

fn try_find_player<'a>(tab_list: &'a TabList, name: &str) -> Option<(&'a Uuid, &'a PlayerInfo)> {
    tab_list.iter().find(|(_, info)| info.profile.name == name)
}

#[cfg(feature = "discord")]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Json {
    message: String,
    status:  String,
    uuid:    Option<Uuid>,
}
