//! Signed cookie session for public tblfee lookup, plus student-facing masking.

use axum::http::{HeaderMap, header};

use crate::entities::tblfee;
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

#[derive(Clone, Debug)]
pub struct StudentFeeView {
    pub session: String,
    pub receipt_id: String,
    pub name: String,
    pub dob: String,
    pub category: String,
    pub father_name: String,
    pub mobile: String,
    pub enrollment: String,
    pub program_code: String,
    pub courses: String,
    pub date_of_deposit: String,
    pub submit_type: String,
}

impl StudentFeeView {
    pub fn from_model(row: &tblfee::Model) -> Self {
        Self {
            session: row.adm_session.trim().to_string(),
            receipt_id: row.id.to_string(),
            name: row.student.clone(),
            dob: mask_dob_year(&row.dob),
            category: row.category.clone(),
            father_name: row.father_name.clone(),
            mobile: mask_mobile(&row.contact),
            enrollment: row.enroll.clone(),
            program_code: row.prog.clone(),
            courses: row.courses.clone(),
            date_of_deposit: row.dod_display(),
            submit_type: row.submit.clone(),
        }
    }
}

pub fn digits_only(s: &str) -> String {
    s.chars().filter(|c| c.is_ascii_digit()).collect()
}

pub fn contact_matches(stored: &str, userid: &str) -> bool {
    let user = userid.trim();
    if user.is_empty() {
        return false;
    }
    if stored.trim() == user {
        return true;
    }
    let a = digits_only(stored);
    let b = digits_only(user);
    if a.is_empty() || b.is_empty() {
        return false;
    }
    if a == b {
        return true;
    }
    let a10 = if a.len() >= 10 {
        &a[a.len() - 10..]
    } else {
        a.as_str()
    };
    let b10 = if b.len() >= 10 {
        &b[b.len() - 10..]
    } else {
        b.as_str()
    };
    a10 == b10 && a10.len() >= 10
}

pub fn mask_dob_year(dob: &str) -> String {
    let s = dob.trim();
    if s.is_empty() {
        return String::new();
    }
    let parts: Vec<&str> = if s.contains('-') {
        s.split('-').collect()
    } else if s.contains('/') {
        s.split('/').collect()
    } else {
        return s.to_string();
    };
    if parts.len() != 3 {
        return s.to_string();
    }
    if parts[0].len() == 4 && parts[0].chars().all(|c| c.is_ascii_digit()) {
        return format!("****-{}-{}", parts[1], parts[2]);
    }
    if parts[2].len() == 4 && parts[2].chars().all(|c| c.is_ascii_digit()) {
        let sep = if s.contains('-') { "-" } else { "/" };
        return format!("{}{sep}{}{sep}****", parts[0], parts[1]);
    }
    s.to_string()
}

pub fn mask_mobile(contact: &str) -> String {
    let raw = contact.trim();
    if raw.is_empty() {
        return String::new();
    }
    let digits = digits_only(raw);
    if digits.len() < 4 {
        return "*".repeat(raw.chars().count().max(1));
    }
    let visible = &digits[digits.len() - 4..];
    format!("{}{visible}", "*".repeat(digits.len().saturating_sub(4)))
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
    fn masks_dob_year_dmy() {
        assert_eq!(mask_dob_year("07-02-1984"), "07-02-****");
        assert_eq!(mask_dob_year("07/02/1984"), "07/02/****");
        assert_eq!(mask_dob_year("1984-02-07"), "****-02-07");
    }

    #[test]
    fn masks_mobile_last_four() {
        assert_eq!(mask_mobile("9915636130"), "******6130");
        assert_eq!(mask_mobile("91 9915636130"), "********6130");
    }

    #[test]
    fn contact_matches_last_ten_digits() {
        assert!(contact_matches("9915636130", "9915636130"));
        assert!(contact_matches("919915636130", "9915636130"));
        assert!(!contact_matches("9915636130", "9915636131"));
        assert!(!contact_matches("9915636130", ""));
    }

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
