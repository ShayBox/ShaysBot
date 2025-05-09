use azalea::{
    app::{App, Plugin, Update},
    ecs::prelude::*,
    PlayerInfo,
    TabList,
};
#[cfg(feature = "bot")]
use serde::Deserialize;
#[cfg(feature = "bot")]
use serenity::all::UserId;
use uuid::Uuid;

use crate::prelude::*;

/// Add or remove players from the whitelist or link their Discord.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WhitelistCommandPlugin;

impl Cmd for WhitelistCommandPlugin {
    fn aliases(&self) -> Vec<&'static str> {
        vec!["whitelist"]
    }
}

impl Plugin for WhitelistCommandPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            Self::handle_whitelist_cmd_events
                .ambiguous_with_all()
                .before(MinecraftParserPlugin::handle_send_msg_events)
                .after(MinecraftParserPlugin::handle_chat_received_events),
        );
    }
}

impl WhitelistCommandPlugin {
    /// # Panics
    /// Will panic if `DeriveTomlConfig::save` fails.
    pub fn handle_whitelist_cmd_events(
        mut cmd_events: EventReader<CmdEvent>,
        mut msg_events: EventWriter<MsgEvent>,
        mut settings: ResMut<GlobalSettings>,
        tab_list: Res<TabList>,
    ) {
        if let Some(event) = cmd_events.read().next() {
            let Cmds::Whitelist(_plugin) = event.cmd else {
                return;
            };

            let mut args = event.args.clone();
            let mut msg_event = MsgEvent {
                content: String::new(),
                entity:  event.entity,
                sender:  event.sender,
                source:  event.source.clone(),
                status:  200,
            };

            let Some(action) = args.pop_front() else {
                msg_event.content = str!("Missing action | Actions: add, remove, link, & set");
                msg_event.status = 404;
                msg_events.write(msg_event);
                return;
            };

            let discord_id = args.pop_front();
            let (status, content) = match action.as_ref() {
                "add" => handle_add(&mut settings, discord_id, &tab_list),
                #[cfg(feature = "bot")]
                "link" => handle_link(&mut settings, discord_id, &event.sender),
                "remove" => handle_remove(&mut settings, discord_id, &tab_list),
                #[cfg(feature = "api")]
                "set" => handle_set(&mut settings, discord_id, &event.sender),
                _ => (
                    406,
                    str!("Invalid action | Actions: add, remove, link, & set"),
                ),
            };

            msg_event.content = content;
            msg_event.status = status;
            msg_events.write(msg_event);
        }

        cmd_events.clear();
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
        settings.save().expect("Failed to save settings");

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
        settings.save().expect("Failed to save settings");

        (200, format!("Successfully removed: {}", info.profile.name))
    } else {
        (409, str!("Already not whitelisted"))
    }
}

#[cfg(feature = "bot")]
fn handle_link(
    settings: &mut ResMut<GlobalSettings>,
    first_arg: Option<String>,
    sender: &CmdSender,
) -> (u16, String) {
    match sender {
        #[cfg(feature = "api")]
        CmdSender::ApiServer(uuid) => {
            let Some(discord_id) = first_arg else {
                return (404, str!("Missing Discord user id"));
            };

            let Ok(discord_id) = discord_id.parse::<UserId>() else {
                return (404, str!("Invalid Discord user id"));
            };

            settings
                .users
                .entry(*uuid)
                .and_modify(|user| user.discord_id = discord_id.to_string())
                .or_insert_with(|| User {
                    discord_id: discord_id.to_string(),
                    ..Default::default()
                });
            settings.save().expect("Failed to save settings");

            (200, str!("Successfully linked discord"))
        }
        CmdSender::Discord(discord_id) => {
            let Some(link_id) = first_arg else {
                return (404, str!("Missing auth code (Join: auth.aristois.net)"));
            };

            let path = format!("https://auth.aristois.net/token/{link_id}");
            let Ok(mut response) = ureq::get(&path).call() else {
                return (406, str!("Invalid auth code (Join: auth.aristois.net)"));
            };

            let code = response.status();
            let Ok(json) = response.body_mut().read_json::<Json>() else {
                return (500, str!("Failed to parse JSON"));
            };

            let Some(uuid) = json.uuid else {
                return (
                    code.as_u16(),
                    format!("Authentication {}: {}", json.status, json.message),
                );
            };

            settings
                .users
                .entry(uuid)
                .and_modify(|user| user.discord_id = discord_id.to_string())
                .or_insert_with(|| User {
                    discord_id: discord_id.to_string(),
                    ..Default::default()
                });
            settings.save().expect("Failed to save settings");

            (200, str!("Successfully linked"))
        }
        CmdSender::Minecraft(uuid) => {
            let Some(discord_id) = first_arg else {
                return (404, str!("Missing Discord user id"));
            };

            let Ok(discord_id) = discord_id.parse::<UserId>() else {
                return (404, str!("Invalid Discord user id"));
            };

            settings
                .users
                .entry(*uuid)
                .and_modify(|user| user.discord_id = discord_id.to_string())
                .or_insert_with(|| User {
                    discord_id: discord_id.to_string(),
                    ..Default::default()
                });
            settings.save().expect("Failed to save settings");

            (200, str!("Successfully linked"))
        }
    }
}

#[cfg(feature = "api")]
fn handle_set(
    settings: &mut ResMut<GlobalSettings>,
    api_password: Option<String>,
    sender: &CmdSender,
) -> (u16, String) {
    match sender {
        CmdSender::ApiServer(_) => (500, str!("You can't update your API password on the API")),
        #[cfg(feature = "bot")]
        CmdSender::Discord(_) => (500, str!("You can't update your API password on Discord")),
        CmdSender::Minecraft(uuid) => {
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
            settings.save().expect("Failed to save settings");

            (200, str!("Successfully updated password"))
        }
    }
}

fn try_find_player<'a>(tab_list: &'a TabList, name: &str) -> Option<(&'a Uuid, &'a PlayerInfo)> {
    tab_list.iter().find(|(_, info)| info.profile.name == name)
}

#[cfg(feature = "bot")]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Json {
    message: String,
    status:  String,
    uuid:    Option<Uuid>,
}
