//! Signed cookie session for public tblfee lookup.

use axum::http::{HeaderMap, header};

use lariv_rs::plugins::users::session::is_secure_request;
use lariv_rs::web::{clear_cookie_header, set_cookie_header};

use hmac::{Hmac, Mac};
use sha2::Sha256;

const COOKIE_NAME: &str = "tblfee-session";
const SESSION_TTL_SECS: i64 = 12 * 60 * 60;

type HmacSha256 = Hmac<Sha256>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FeeScope {
    Receipt(i64),
    Enroll(String),
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn hex_decode(s: &str) -> Option<Vec<u8>> {
    if s.len() % 2 != 0 {
        return None;
    }
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).ok())
        .collect()
}

fn sign(key: &[u8], payload: &str) -> String {
    let mut mac = HmacSha256::new_from_slice(key).expect("hmac key");
    mac.update(payload.as_bytes());
    hex_encode(&mac.finalize().into_bytes())
}

fn verify(key: &[u8], payload: &str, mac_hex: &str) -> bool {
    let Some(expected) = hex_decode(mac_hex) else {
        return false;
    };
    let Ok(mut mac) = HmacSha256::new_from_slice(key) else {
        return false;
    };
    mac.update(payload.as_bytes());
    mac.verify_slice(&expected).is_ok()
}

fn encode_payload(scope: &FeeScope) -> String {
    match scope {
        FeeScope::Receipt(id) => format!("r:{id}"),
        FeeScope::Enroll(enroll) => format!("e:{enroll}"),
    }
}

fn decode_payload(payload: &str) -> Option<FeeScope> {
    if let Some(id) = payload.strip_prefix("r:") {
        let id: i64 = id.parse().ok()?;
        if id <= 0 {
            return None;
        }
        return Some(FeeScope::Receipt(id));
    }
    if let Some(enroll) = payload.strip_prefix("e:") {
        let enroll = enroll.trim();
        if enroll.is_empty() {
            return None;
        }
        return Some(FeeScope::Enroll(enroll.to_string()));
    }
    None
}

pub fn encode_cookie(scope: &FeeScope, signing_key: &[u8]) -> String {
    let payload = encode_payload(scope);
    let payload_hex = hex_encode(payload.as_bytes());
    let mac = sign(signing_key, &payload_hex);
    format!("{payload_hex}.{mac}")
}

pub fn decode_cookie(value: &str, signing_key: &[u8]) -> Option<FeeScope> {
    let (payload_hex, mac) = value.rsplit_once('.')?;
    if !verify(signing_key, payload_hex, mac) {
        return None;
    }
    let bytes = hex_decode(payload_hex)?;
    let payload = String::from_utf8(bytes).ok()?;
    decode_payload(&payload)
}

pub fn scope_from_headers(headers: &HeaderMap, signing_key: &[u8]) -> Option<FeeScope> {
    let cookie_header = headers.get(header::COOKIE)?.to_str().ok()?;
    for part in cookie_header.split(';') {
        let part = part.trim();
        let prefix = format!("{COOKIE_NAME}=");
        if let Some(value) = part.strip_prefix(&prefix) {
            return decode_cookie(value, signing_key);
        }
    }
    None
}

pub fn set_scope_cookie(
    headers: &mut HeaderMap,
    scope: &FeeScope,
    signing_key: &[u8],
    request_headers: &HeaderMap,
) {
    let token = encode_cookie(scope, signing_key);
    let value = set_cookie_header(
        COOKIE_NAME,
        &token,
        SESSION_TTL_SECS,
        is_secure_request(request_headers),
    );
    headers.append(header::SET_COOKIE, value);
}

pub fn clear_scope_cookie(headers: &mut HeaderMap, request_headers: &HeaderMap) {
    let value = clear_cookie_header(COOKIE_NAME, is_secure_request(request_headers));
    headers.append(header::SET_COOKIE, value);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cookie_roundtrip() {
        let key = b"test-signing-key-bytes-for-hmac";
        let receipt = FeeScope::Receipt(32);
        let token = encode_cookie(&receipt, key);
        assert_eq!(decode_cookie(&token, key), Some(receipt));
        let enroll = FeeScope::Enroll("2000210422".into());
        let token = encode_cookie(&enroll, key);
        assert_eq!(decode_cookie(&token, key), Some(enroll));
        assert_eq!(
            decode_cookie(&token, b"other-key-other-key-other-key!!"),
            None
        );
    }
}
