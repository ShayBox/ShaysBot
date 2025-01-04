use std::{collections::VecDeque, sync::LazyLock};

use azalea::{
    app::{App, Plugin, Update},
    chat::{handle_send_chat_event, ChatKind, ChatReceivedEvent, SendChatKindEvent},
    ecs::prelude::*,
    TabList,
};
use ncr::{
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
    AesKey,
    NcrError,
};

use crate::prelude::*;

pub struct MinecraftChatPlugin;

impl Plugin for MinecraftChatPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CommandCooldown::default())
            .add_event::<CommandEvent>()
            .add_event::<WhisperEvent>()
            .add_systems(
                Update,
                (
                    Self::handle_chat_received_events,
                    Self::handle_whisper_events.before(handle_send_chat_event),
                )
                    .chain(),
            );
    }
}

impl MinecraftChatPlugin {
    pub fn handle_chat_received_events(
        mut chat_received_events: EventReader<ChatReceivedEvent>,
        mut command_events: EventWriter<CommandEvent>,
        mut cooldown: ResMut<CommandCooldown>,
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

            if settings.whitelist && !settings.whitelisted.contains_key(uuid) {
                continue; /* Not Whitelisted */
            }

            let key =
                AesKey::decode_base64(&settings.encryption.key).unwrap_or_else(|_| KEY.clone());
            let (encryption, content) = find_encryption(&content, &key);
            let mut args = content
                .split(' ')
                .map(String::from)
                .collect::<VecDeque<_>>();

            let Some(alias) = args.pop_front() else {
                continue; /* Command Missing */
            };

            if !alias.starts_with(&settings.command_prefix) {
                continue;
            }

            let Some(command) = ChatCmds::find(&alias.replace(&settings.command_prefix, "")) else {
                continue; /* Command Invalid */
            };

            if cooldown.check(&username, settings.command_cooldown) {
                info!("Command on cooldown");
                continue; /* Command Cooldown */
            }

            events.push(CommandEvent {
                entity: event.entity,
                args,
                command,
                message,
                sender: CommandSender::Minecraft(*uuid),
                source: CommandSource::Minecraft(encryption),
            });
        }

        command_events.send_batch(events);
    }

    pub fn handle_whisper_events(
        mut chat_kind_events: EventWriter<SendChatKindEvent>,
        mut whisper_events: EventReader<WhisperEvent>,
        query: Query<(&TabList, &LocalSettings)>,
        settings: Res<GlobalSettings>,
    ) {
        for mut event in whisper_events.read().cloned() {
            #[rustfmt::skip]
            let (
                CommandSource::Minecraft(type_encryption),
                CommandSender::Minecraft(uuid)
            ) = (event.source, event.sender) else {
                continue;
            };

            let Ok((tab_list, bot_settings)) = query.get(event.entity) else {
                return;
            };

            if bot_settings.disable_responses {
                continue; /* Responses Disabled */
            }

            let Some(username) = tab_list
                .iter()
                .find(|(_, info)| info.profile.uuid == uuid)
                .map(|(_, info)| info.profile.name.clone())
            else {
                continue; /* Player Offline */
            };

            try_encrypt(&mut event.content, &settings.encryption, type_encryption);

            chat_kind_events.send(SendChatKindEvent {
                entity:  event.entity,
                kind:    ChatKind::Command,
                content: format!("w {username} {}", event.content),
            });
        }
    }
}

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
