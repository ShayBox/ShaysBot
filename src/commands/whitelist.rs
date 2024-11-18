use azalea::{
    app::{App, Plugin, Update},
    ecs::prelude::*,
    PlayerInfo,
    TabList,
};
use derive_config::DeriveTomlConfig;
use handlers::prelude::*;
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    commands::{prelude::*, Commands},
    Settings,
};

/// Whitelist command
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WhitelistCommandPlugin;

impl Command for WhitelistCommandPlugin {
    fn aliases(&self) -> Vec<&'static str> {
        vec!["whitelist"]
    }
}

impl Plugin for WhitelistCommandPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            handle_whitelist_command_event
                .ambiguous_with_all()
                .before(handle_discord_whisper_event)
                .before(handle_minecraft_whisper_event)
                .after(handle_chat_received_event),
        );
    }
}

/// # Panics
/// Will panic if `DeriveTomlConfig::save` fails.
pub fn handle_whitelist_command_event(
    mut command_events: EventReader<CommandEvent>,
    mut whisper_events: EventWriter<WhisperEvent>,
    mut settings: ResMut<Settings>,
    query: Query<&TabList>,
) {
    for event in command_events.read() {
        let Commands::Whitelist(_plugin) = event.command else {
            continue;
        };

        let Ok(tab_list) = query.get_single() else {
            continue;
        };

        let mut args = event.args.clone();
        let mut whisper_event = WhisperEvent {
            entity:  event.entity,
            source:  event.source,
            sender:  event.sender,
            content: String::new(),
        };

        let Some(action) = args.pop_front() else {
            whisper_event.content = String::from("[400] Missing Action: 'add', 'remove', 'link'");
            whisper_events.send(whisper_event);
            continue;
        };

        let user = args.pop_front();
        whisper_event.content = match action.as_ref() {
            "add" => handle_add(&mut settings, user, tab_list),
            "remove" => handle_remove(&mut settings, user, tab_list),
            "link" => handle_link(&mut settings, user, &event.sender),
            _ => String::from("[400] Invalid Action: 'add', 'remove', or 'link'"),
        };

        whisper_events.send(whisper_event);
    }
}

fn handle_add(settings: &mut ResMut<Settings>, user: Option<String>, tab_list: &TabList) -> String {
    let Some(player_name) = user else {
        return String::from("[400] Missing Minecraft player name");
    };

    let Some((uuid, info)) = try_find_player(tab_list, &player_name) else {
        return String::from("[404] Player not found");
    };

    if settings.whitelisted.contains_key(uuid) {
        String::from("[409] Already whitelisted")
    } else {
        settings.whitelisted.insert(*uuid, None);
        settings.save().expect("Failed to save settings");

        format!("[200] Successfully added: {}", info.profile.name)
    }
}

fn handle_remove(
    settings: &mut ResMut<Settings>,
    user: Option<String>,
    tab_list: &TabList,
) -> String {
    let Some(player_name) = user else {
        return String::from("[400] Missing Minecraft player name");
    };

    let Some((uuid, info)) = try_find_player(tab_list, &player_name) else {
        return String::from("[404] Player not found");
    };

    if settings.whitelisted.contains_key(uuid) {
        settings.whitelisted.remove(uuid);
        settings.save().expect("Failed to save settings");

        format!("[200] Successfully removed: {}", info.profile.name)
    } else {
        String::from("[409] Already not whitelisted")
    }
}

fn handle_link(
    settings: &mut ResMut<Settings>,
    user: Option<String>,
    sender: &CommandSender,
) -> String {
    match sender {
        CommandSender::Discord(_) => {
            let Some(auth_code) = user else {
                return String::from("[400] Missing auth code (Join: auth.aristois.net)");
            };

            let path = format!("https://auth.aristois.net/token/{auth_code}");
            let response = ureq::get(&path).call().expect("Failed to authenticate");

            let code = response.status();
            let Ok(json) = response.into_json::<Json>() else {
                return String::from("[500] Failed to parse JSON");
            };

            let Some(uuid) = json.uuid else {
                return format!("[{code}] Authentication {}: {}", json.status, json.message);
            };

            settings.whitelisted.insert(uuid, Some(auth_code));
            settings.save().expect("Failed to save settings");

            String::from("[200] Successfully linked")
        }
        CommandSender::Minecraft(uuid) => {
            let Some(user_id) = user else {
                return String::from("[400] Missing Discord user id");
            };

            settings.whitelisted.insert(*uuid, Some(user_id));
            settings.save().expect("Failed to save settings");

            String::from("[200] Successfully linked")
        }
    }
}

fn try_find_player<'a>(tab_list: &'a TabList, name: &str) -> Option<(&'a Uuid, &'a PlayerInfo)> {
    tab_list.iter().find(|(_, info)| info.profile.name == name)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Json {
    message: String,
    status:  String,
    uuid:    Option<Uuid>,
}
