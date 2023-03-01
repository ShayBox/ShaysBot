use std::collections::HashMap;

use anyhow::Result;
use base64::{
    alphabet::Alphabet,
    engine::{GeneralPurpose, GeneralPurposeConfig},
    Engine,
};
use ncr_crypto::{decode_and_verify, decrypt_with_passphrase, encrypt_with_passphrase};

const BASE64_ALPHABET: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
const BASE64R_CHARSET: [(char, char); 65] = [
    ('A', '!'),
    ('B', '"'),
    ('C', '#'),
    ('D', '$'),
    ('E', '%'),
    ('F', '¼'),
    ('G', '\''),
    ('H', '('),
    ('I', ')'),
    ('J', ','),
    ('K', '-'),
    ('L', '.'),
    ('M', ':'),
    ('N', ';'),
    ('O', '<'),
    ('P', '='),
    ('Q', '>'),
    ('R', '?'),
    ('S', '@'),
    ('T', '['),
    ('U', '\\'),
    ('V', ']'),
    ('W', '^'),
    ('X', '_'),
    ('Y', '`'),
    ('Z', '{'),
    ('a', '|'),
    ('b', '}'),
    ('c', '~'),
    ('d', '¡'),
    ('e', '¢'),
    ('f', '£'),
    ('g', '¤'),
    ('h', '¥'),
    ('i', '¦'),
    ('j', '¨'),
    ('k', '©'),
    ('l', 'ª'),
    ('m', '«'),
    ('n', '¬'),
    ('o', '®'),
    ('p', '¯'),
    ('q', '°'),
    ('r', '±'),
    ('s', '²'),
    ('t', '³'),
    ('u', 'µ'),
    ('v', '¶'),
    ('w', '·'),
    ('x', '×'),
    ('y', '¹'),
    ('z', 'º'),
    ('0', '0'),
    ('1', '1'),
    ('2', '2'),
    ('3', '3'),
    ('4', '4'),
    ('5', '5'),
    ('6', '6'),
    ('7', '7'),
    ('8', '8'),
    ('9', '9'),
    ('+', '+'),
    ('/', '»'),
    ('=', '¿'),
];

pub struct NCReply {
    pub encrypt_fn: fn(input: &str, passphrase: &[u8]) -> String,
    pub passphrase: Vec<u8>,
}

pub fn decrypt_ncr(message: String, passphrases: Vec<String>) -> (String, Option<NCReply>) {
    for try_decode in [try_decrypt_aes_cfb8_base64, try_decrypt_aes_cfb8_base64r] {
        for passphrase in &passphrases {
            if let Ok(tuple) = try_decode(&message, passphrase.as_bytes()) {
                return tuple;
            }
        }
    }

    (message, None)
}

// AES/CFB8+Base64 (Not used in the mod, but why not)
pub fn try_decode_base64(input: &str) -> Result<Vec<u8>> {
    let alphabet = Alphabet::new(BASE64_ALPHABET).expect("Failed to create Alphabet");
    let b64 = GeneralPurpose::new(&alphabet, GeneralPurposeConfig::new());
    b64.decode(input).map_err(anyhow::Error::msg)
}

pub fn encode_base64(input: Vec<u8>) -> String {
    let alphabet = Alphabet::new(BASE64_ALPHABET).expect("Failed to create Alphabet");
    let b64 = GeneralPurpose::new(&alphabet, GeneralPurposeConfig::new());
    b64.encode(input)
}

pub fn try_decrypt_aes_cfb8_base64(
    input: &str,
    passphrase: &[u8],
) -> Result<(String, Option<NCReply>)> {
    let ciphertext = try_decode_base64(input)?;
    let decrypted = decrypt_with_passphrase(&ciphertext, passphrase);
    let decoded = decode_and_verify(&decrypted).map_err(anyhow::Error::msg)?;

    Ok((
        decoded.replacen("#%", "", 1),
        Some(NCReply {
            encrypt_fn: encrypt_aes_cfb8_base64,
            passphrase: passphrase.to_vec(),
        }),
    ))
}

pub fn encrypt_aes_cfb8_base64(input: &str, passphrase: &[u8]) -> String {
    let plaintext = "#%".to_owned() + input;
    let encrypted = encrypt_with_passphrase(plaintext.as_bytes(), passphrase);
    encode_base64(encrypted)
}

// AES/CFB8+Base64R (Default algorithm)
pub fn try_decode_base64r(input: &str) -> Result<Vec<u8>> {
    let base64r = HashMap::from(BASE64R_CHARSET);
    let base64r_reversed = base64r
        .iter()
        .map(|(a, b)| (b.to_owned(), a.to_owned()))
        .collect::<HashMap<char, char>>();
    let input = input
        .chars()
        .map(|char| match base64r_reversed.get(&char) {
            Some(new_char) => *new_char,
            None => char,
        })
        .collect::<String>();

    let alphabet = Alphabet::new(BASE64_ALPHABET).expect("Failed to create Alphabet");
    let b64 = GeneralPurpose::new(&alphabet, GeneralPurposeConfig::new());
    b64.decode(input).map_err(anyhow::Error::msg)
}

pub fn encode_base64r(input: Vec<u8>) -> String {
    let base64r = HashMap::from(BASE64R_CHARSET);
    let encoded = encode_base64(input);

    encoded
        .chars()
        .map(|char| match base64r.get(&char) {
            Some(new_char) => *new_char,
            None => char,
        })
        .collect::<String>()
}

pub fn try_decrypt_aes_cfb8_base64r(
    input: &str,
    passphrase: &[u8],
) -> Result<(String, Option<NCReply>)> {
    let ciphertext = try_decode_base64r(input)?;
    let decrypted = decrypt_with_passphrase(&ciphertext, passphrase);
    let decoded = decode_and_verify(&decrypted).map_err(anyhow::Error::msg)?;

    Ok((
        decoded.replacen("#%", "", 1),
        Some(NCReply {
            encrypt_fn: encrypt_aes_cfb8_base64r,
            passphrase: passphrase.to_vec(),
        }),
    ))
}

pub fn encrypt_aes_cfb8_base64r(input: &str, passphrase: &[u8]) -> String {
    let plaintext = "#%".to_owned() + input;
    let encrypted = encrypt_with_passphrase(plaintext.as_bytes(), passphrase);
    encode_base64r(encrypted)
}
