use azalea::{
    app::{App, Plugin, Startup, Update},
    ecs::prelude::*,
    TabList,
};
use derive_config::DeriveTomlConfig;

use crate::plugins::commands::prelude::*;

/// Whitelist command
pub struct WhitelistCommandPlugin;

impl Plugin for WhitelistCommandPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, handle_whitelist_register)
            .add_systems(
                Update,
                handle_whitelist_command_event
                    .ambiguous_with_all()
                    .before(handle_discord_whisper_event)
                    .before(handle_minecraft_whisper_event)
                    .after(handle_chat_received_event),
            );
    }
}

pub fn handle_whitelist_register(mut registry: ResMut<Registry>) {
    registry.register("whitelist", Command::Whitelist);
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
        if event.command != Command::Whitelist {
            continue;
        }

        let Ok(tab_list) = query.get_single() else {
            continue;
        };

        let mut args = event.args.clone();
        let mut whisper_event = WhisperEvent {
            entity:  event.entity,
            source:  event.source.clone(),
            sender:  event.sender.clone(),
            content: String::new(),
        };

        let Some(action) = args.pop_front() else {
            whisper_event.content =
                String::from("[400] Missing Action: 'add', 'remove', or 'link'");
            whisper_events.send(whisper_event);
            continue;
        };

        let Some(username) = args.pop_front() else {
            whisper_event.content = String::from("[400] Missing username");
            whisper_events.send(whisper_event);
            continue;
        };

        whisper_event.content = match action.as_ref() {
            "add" => {
                let Some((uuid, _info)) = tab_list
                    .iter()
                    .find(|(_, info)| info.profile.name == username)
                else {
                    whisper_event.content = String::from("[404] Player not found");
                    whisper_events.send(whisper_event);
                    continue;
                };

                if settings.whitelist.contains_key(uuid) {
                    String::from("[409] Already whitelisted.")
                } else {
                    settings.whitelist.insert(*uuid, None);
                    settings.save().expect("Failed to save settings");

                    String::from("[200] Successfully added")
                }
            }
            "remove" => {
                let Some((uuid, _info)) = tab_list
                    .iter()
                    .find(|(_, info)| info.profile.name == username)
                else {
                    whisper_event.content = String::from("[404] Player not found");
                    whisper_events.send(whisper_event);
                    continue;
                };

                if settings.whitelist.contains_key(uuid) {
                    settings.whitelist.remove(uuid);
                    settings.save().expect("Failed to save settings");

                    String::from("[200] Succesfully removed")
                } else {
                    String::from("[409] Already not whitelisted")
                }
            }
            "link" => match &event.sender {
                CommandSender::Discord(_) => {
                    String::from("[403] You must run this sub-command in-game")
                }
                CommandSender::Minecraft(sender) => {
                    let Some((uuid, _info)) = tab_list
                        .iter()
                        .find(|(_, info)| &info.profile.name == sender)
                    else {
                        whisper_event.content = String::from("[404] Sender not found");
                        whisper_events.send(whisper_event);
                        continue;
                    };

                    settings.whitelist.insert(*uuid, Some(username));
                    settings.save().expect("Failed to save settings");

                    String::from("[200] Successfully linked")
                }
            },
            _ => String::from("[400] Invalid Action: 'add', 'remove', or 'link'"),
        };

        whisper_events.send(whisper_event);
    }
}
