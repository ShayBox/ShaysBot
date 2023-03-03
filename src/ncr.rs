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

pub struct NCREncryption {
    pub encrypt_fn: fn(input: &str, passphrase: &str) -> String,
    pub passphrase: String,
}

pub fn decrypt_ncr(message: String, passphrases: Vec<String>) -> (String, Option<NCREncryption>) {
    for try_decrypt_fn in [try_decrypt_aes_cfb8_base64, try_decrypt_aes_cfb8_base64r] {
        for passphrase in &passphrases {
            let result = try_decrypt_fn(&message, passphrase);
            if let Ok((message, ncr)) = result {
                return (message, ncr);
            }
        }
    }

    // Message wasn't encrypted or wasn't able to be decrypted
    (message, None)
}

// Base64
fn try_decode_base64(input: &str) -> Result<Vec<u8>> {
    let base64_alphabet = Alphabet::new(BASE64_ALPHABET).expect("Failed to create Alphabet");
    let base64 = GeneralPurpose::new(&base64_alphabet, GeneralPurposeConfig::new());
    base64.decode(input).map_err(anyhow::Error::msg)
}

fn encode_base64(input: Vec<u8>) -> String {
    let base64_alphabet = Alphabet::new(BASE64_ALPHABET).expect("Failed to create Alphabet");
    let base64 = GeneralPurpose::new(&base64_alphabet, GeneralPurposeConfig::new());
    base64.encode(input)
}

// AES/CFB8+Base64 (Not used in the mod, but why not)
fn try_decrypt_aes_cfb8_base64(
    input: &str,
    passphrase: &str,
) -> Result<(String, Option<NCREncryption>)> {
    let ciphertext = try_decode_base64(input)?;
    let decrypted = decrypt_with_passphrase(&ciphertext, passphrase.as_ref());
    let decoded = decode_and_verify(&decrypted).map_err(anyhow::Error::msg)?;

    Ok((
        decoded.replacen("#%", "", 1),
        Some(NCREncryption {
            encrypt_fn: encrypt_aes_cfb8_base64,
            passphrase: passphrase.into(),
        }),
    ))
}

fn encrypt_aes_cfb8_base64(input: &str, passphrase: &str) -> String {
    let plaintext = "#%".to_owned() + input;
    let encrypted = encrypt_with_passphrase(plaintext.as_bytes(), passphrase.as_ref());
    encode_base64(encrypted)
}

// Base64R
fn try_decode_base64r(input: &str) -> Result<Vec<u8>> {
    let base64r_charset_reversed = HashMap::from(BASE64R_CHARSET)
        .iter()
        .map(|(a, b)| (b.to_owned(), a.to_owned()))
        .collect::<HashMap<char, char>>();

    let input = input
        .chars()
        .map(|char| match base64r_charset_reversed.get(&char) {
            Some(new_char) => *new_char,
            None => char,
        })
        .collect::<String>();

    try_decode_base64(&input)
}

fn encode_base64r(input: Vec<u8>) -> String {
    let base64r_charset = HashMap::from(BASE64R_CHARSET);
    let encoded = encode_base64(input);

    encoded
        .chars()
        .map(|char| match base64r_charset.get(&char) {
            Some(new_char) => *new_char,
            None => char,
        })
        .collect::<String>()
}

// AES/CFB8+Base64R (Default algorithm)
fn try_decrypt_aes_cfb8_base64r(
    input: &str,
    passphrase: &str,
) -> Result<(String, Option<NCREncryption>)> {
    let ciphertext = try_decode_base64r(input)?;
    let decrypted = decrypt_with_passphrase(&ciphertext, passphrase.as_ref());
    let decoded = decode_and_verify(&decrypted).map_err(anyhow::Error::msg)?;

    Ok((
        decoded.replacen("#%", "", 1),
        Some(NCREncryption {
            encrypt_fn: encrypt_aes_cfb8_base64r,
            passphrase: passphrase.into(),
        }),
    ))
}

fn encrypt_aes_cfb8_base64r(input: &str, passphrase: &str) -> String {
    let plaintext = "#%".to_owned() + input;
    let encrypted = encrypt_with_passphrase(plaintext.as_bytes(), passphrase.as_ref());
    encode_base64r(encrypted)
}
