use std::{
    collections::VecDeque,
    fmt::Write,
    sync::LazyLock,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::Result;
use azalea::{
    app::{App, Plugin, Update},
    chat::{ChatKind, ChatReceivedEvent, handle_send_chat_event, handler::SendChatKindEvent},
    ecs::prelude::*,
    local_player::TabList,
};
use ncr::{
    AesKey,
    NcrError,
    encoding::{
        Base64Encoding,
        Base64rEncoding,
        Encoding,
        Mc256Encoding,
        NewBase64rEncoding,
        Sus16Encoding,
    },
    encryption::{Cfb8Encryption, EcbEncryption, Encryption, GcmEncryption},
    utils::{prepend_header, trim_header},
};

use crate::prelude::*;

/// Minecraft chat command parsing integration
pub struct MinecraftParserPlugin;

impl Plugin for MinecraftParserPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CmdCooldown::default())
            .add_message::<CmdEvent>()
            .add_message::<MsgEvent>()
            .add_systems(
                Update,
                (
                    Self::handle_chat_received_events,
                    Self::handle_send_msg_events.before(handle_send_chat_event),
                )
                    .chain(),
            );
    }
}

impl MinecraftParserPlugin {
    pub fn handle_chat_received_events(
        mut chat_received_events: MessageReader<ChatReceivedEvent>,
        mut cmd_events: MessageWriter<CmdEvent>,
        mut cooldown: ResMut<CmdCooldown>,
        query: Query<&TabList>,
        settings: Res<GlobalSettings>,
    ) {
        let mut events = Vec::new();
        for event in chat_received_events.read() {
            let (username, content) = event.packet.split_sender_and_content();
            let (username, content, message) = if let Some(username) = username {
                (username, content, event.packet.is_whisper()) /* Vanilla Server Format */
            } else if let Some((_whole, username, whisper, content)) = regex_captures!(
                r"^(?:\[.+\] )?([a-zA-Z_0-9]{1,16}) (?:> )?(whispers: |-> me] )?(.+)$",
                &content /* Custom Server Formats */
            ) {
                (str!(username), str!(content), !whisper.is_empty())
            } else {
                continue;
            };

            let Ok(tab_list) = query.get(event.entity) else {
                return; /* Not Connected */
            };

            let Some((uuid, _)) = tab_list.iter().find(|(_, i)| i.profile.name == username) else {
                continue; /* Not Online */
            };

            if settings.whitelist_only && !settings.users.contains_key(uuid) {
                continue; /* Not Whitelisted */
            }

            let key = AesKey::decode_base64(&settings.chat.key).unwrap_or_else(|_| KEY.clone());
            let (encryption, content) = find_encryption(&content, &key);
            let mut args = content
                .split(' ')
                .map(String::from)
                .collect::<VecDeque<_>>();

            let Some(alias) = args.pop_front() else {
                continue; /* Command Missing */
            };

            if !alias.starts_with(&settings.command_prefix) {
                continue; /* Command Invalid */
            }

            let Some(command) = Cmds::find(&alias.replace(&settings.command_prefix, "")) else {
                continue; /* Command Invalid */
            };

            if cooldown.check(&username, settings.command_cooldown) {
                info!("Command on cooldown");
                continue; /* Command Cooldown */
            }

            events.push(CmdEvent {
                args,
                cmd: command,
                entity: Some(event.entity),
                message,
                sender: CmdSender::Minecraft(*uuid),
                source: CmdSource::Minecraft(encryption),
            });
        }

        cmd_events.write_batch(events);
    }

    pub fn handle_send_msg_events(
        mut chat_kind_events: MessageWriter<SendChatKindEvent>,
        mut msg_events: MessageReader<MsgEvent>,
        query: Query<(&TabList, &LocalSettings)>,
        settings: Res<GlobalSettings>,
    ) {
        for mut event in msg_events.read().cloned() {
            #[rustfmt::skip]
            let (
                Some(entity),
                CmdSource::Minecraft(type_encryption),
                CmdSender::Minecraft(uuid)
            ) = (event.entity, event.source, event.sender) else {
                continue;
            };

            let Ok((tab_list, local_settings)) = query.get(entity) else {
                return;
            };

            if local_settings.disable_responses {
                continue; /* Responses Disabled */
            }

            let Some(username) = tab_list
                .iter()
                .find(|(_, info)| info.profile.uuid == uuid)
                .map(|(_, info)| info.profile.name.clone())
            else {
                continue; /* Player Offline */
            };

            info!("Command Response: {}", event.content);
            if local_settings.anti_spam.enabled {
                if let Ok(duration) = SystemTime::now().duration_since(UNIX_EPOCH) {
                    let _ = write!(event.content, " [{}]", duration.as_secs());
                }
            }

            try_encrypt(&mut event.content, &settings.chat, type_encryption);
            chat_kind_events.write(SendChatKindEvent {
                content: format!("w {username} {}", event.content),
                entity,
                kind: ChatKind::Command,
            });
        }
    }
}

/* No Chat Reports Mod */

pub static KEY: LazyLock<AesKey> = LazyLock::new(|| {
    AesKey::from([
        110, 87, 235, 158, 0, 43, 147, 119, 33, 27, 172, 51, 157, 195, 153, 228,
    ])
});

static ENCODERS: LazyLock<[EncodingType; 3]> = LazyLock::new(|| {
    [
        EncodingType::NewBase64r,
        EncodingType::Base64r,
        EncodingType::Base64,
    ]
});

#[derive(Clone, Copy, Debug)]
pub enum EncodingType {
    Base64,
    Base64r,
    NewBase64r,
    Mc256,
    Sus16,
}

impl Encoding for EncodingType {
    fn encode(self, text: &[u8]) -> String {
        match self {
            Self::NewBase64r => NewBase64rEncoding.encode(text),
            Self::Base64r => Base64rEncoding.encode(text),
            Self::Base64 => Base64Encoding.encode(text),
            Self::Mc256 => Mc256Encoding.encode(text),
            Self::Sus16 => Sus16Encoding.encode(text),
        }
    }

    fn decode(self, text: &str) -> Result<Vec<u8>, NcrError> {
        match self {
            Self::NewBase64r => NewBase64rEncoding.decode(text),
            Self::Base64r => Base64rEncoding.decode(text),
            Self::Base64 => Base64Encoding.decode(text),
            Self::Mc256 => Mc256Encoding.decode(text),
            Self::Sus16 => Sus16Encoding.decode(text),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum EncryptionType {
    CFB(EncodingType),
    ECB(EncodingType),
    GCM(EncodingType),
}

impl Encryption for EncryptionType {
    type KeyType = AesKey;

    fn encrypt(self, plaintext: &str, key: &Self::KeyType) -> Result<String, NcrError> {
        match self {
            Self::CFB(encryption) => Cfb8Encryption(encryption).encrypt(plaintext, key),
            Self::ECB(encryption) => EcbEncryption(encryption).encrypt(plaintext, key),
            Self::GCM(encryption) => GcmEncryption(encryption).encrypt(plaintext, key),
        }
    }

    fn decrypt(self, ciphertext: &str, key: &Self::KeyType) -> Result<String, NcrError> {
        match self {
            Self::CFB(encryption) => Cfb8Encryption(encryption).decrypt(ciphertext, key),
            Self::ECB(encryption) => EcbEncryption(encryption).decrypt(ciphertext, key),
            Self::GCM(encryption) => GcmEncryption(encryption).decrypt(ciphertext, key),
        }
    }
}

#[must_use]
pub fn find_encryption(content: &str, key: &AesKey) -> (Option<EncryptionType>, String) {
    for &encoder in ENCODERS.iter() {
        let encryptors = [
            EncryptionType::CFB(encoder),
            EncryptionType::ECB(encoder),
            EncryptionType::GCM(encoder),
        ];

        for encryptor in encryptors {
            if let Ok(plaintext) = encryptor.decrypt(content, key) {
                if let Ok(trimmed) = trim_header(&plaintext) {
                    return (Some(encryptor), String::from(trimmed));
                }
            }
        }
    }

    (None, String::from(content))
}

pub fn try_encrypt(
    content: &mut String,
    chat_encryption: &ChatEncryption,
    type_encryption: Option<EncryptionType>,
) {
    if chat_encryption.mode == EncryptionMode::Never {
        return;
    }

    let key = AesKey::decode_base64(&chat_encryption.key).unwrap_or_else(|_| KEY.clone());
    let plaintext = prepend_header(content);

    if let Some(encryption) = type_encryption {
        if let Ok(ciphertext) = encryption.encrypt(&plaintext, &key) {
            *content = ciphertext;
        }
    } else if chat_encryption.mode == EncryptionMode::Always {
        if let Ok(ciphertext) = Cfb8Encryption(NewBase64rEncoding).encrypt(&plaintext, &key) {
            *content = ciphertext;
        }
    }
}
