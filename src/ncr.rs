use std::sync::LazyLock;

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

use crate::settings::{ChatEncryption, EncryptionMode};

pub static KEY: LazyLock<AesKey> = LazyLock::new(|| {
    AesKey::from([
        110, 87, 235, 158, 0, 43, 147, 119, 33, 27, 172, 51, 157, 195, 153, 228,
    ])
});

#[derive(Clone, Copy, Debug)]
pub enum EncodingType {
    Base64,
    Base64r,
    NewBase64r,
    MC256,
    Sus16,
}

impl Encoding for EncodingType {
    fn encode(self, text: &[u8]) -> String {
        match self {
            Self::NewBase64r => NewBase64rEncoding.encode(text),
            Self::Base64r => Base64rEncoding.encode(text),
            Self::Base64 => Base64Encoding.encode(text),
            Self::MC256 => Mc256Encoding.encode(text),
            Self::Sus16 => Sus16Encoding.encode(text),
        }
    }

    fn decode(self, text: &str) -> Result<Vec<u8>, NcrError> {
        match self {
            Self::NewBase64r => NewBase64rEncoding.decode(text),
            Self::Base64r => Base64rEncoding.decode(text),
            Self::Base64 => Base64Encoding.decode(text),
            Self::MC256 => Mc256Encoding.decode(text),
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
pub fn find_encryption(ciphertext: &str, key: &AesKey) -> (Option<EncryptionType>, String) {
    let encoders = vec![
        EncodingType::NewBase64r,
        EncodingType::Base64r,
        EncodingType::Base64,
        EncodingType::NewBase64r,
        EncodingType::Base64r,
        EncodingType::Base64,
        EncodingType::NewBase64r,
        EncodingType::Base64r,
        EncodingType::Base64,
    ];

    for encoder in encoders {
        let encryptors = vec![
            EncryptionType::CFB(encoder),
            EncryptionType::ECB(encoder),
            EncryptionType::GCM(encoder),
        ];

        for encryptor in encryptors {
            if let Ok(plaintext) = encryptor.decrypt(ciphertext, key) {
                if let Ok(trimmed) = trim_header(&plaintext) {
                    return (Some(encryptor), String::from(trimmed));
                }
            }
        }
    }

    (None, String::from(ciphertext))
}

#[must_use]
pub fn try_encrypt(
    chat_encryption: &ChatEncryption,
    type_encryption: Option<EncryptionType>,
    plaintext: String,
) -> String {
    if chat_encryption.mode == EncryptionMode::Never {
        return plaintext;
    }

    let key = AesKey::decode_base64(&chat_encryption.key).unwrap_or_else(|_| KEY.clone());
    if let Some(encryption) = type_encryption.as_ref() {
        if let Ok(ciphertext) = encryption.encrypt(&prepend_header(&plaintext), &key) {
            return ciphertext;
        }
    } else if chat_encryption.mode == EncryptionMode::Always {
        if let Ok(ciphertext) =
            Cfb8Encryption(NewBase64rEncoding).encrypt(&prepend_header(&plaintext), &key)
        {
            return ciphertext;
        }
    }

    plaintext
}
