//! Lariv `environment` JSON cookie (admission-session selector, etc.).

use std::collections::HashMap;

use axum::http::HeaderMap;

pub const ACADEMIC_RECORDS_SESSION_KEY: &str = "academicrecords_session";
pub const EXAM_REGISTRATIONS_SESSION_KEY: &str = "examregistrations_session";
pub const ASSIGNMENT_SUBMISSIONS_SESSION_KEY: &str = "assignmentsubmissions_session";

pub fn parse_environment_from_headers(headers: &HeaderMap) -> HashMap<String, String> {
    let Some(cookie_header) = headers.get(axum::http::header::COOKIE) else {
        return HashMap::new();
    };
    let Ok(raw) = cookie_header.to_str() else {
        return HashMap::new();
    };
    for part in raw.split(';') {
        let part = part.trim();
        if let Some(val) = part.strip_prefix("environment=") {
            let decoded = percent_decode(val);
            if let Ok(value) = serde_json::from_str::<serde_json::Value>(&decoded)
                && let Some(map) = json_object_to_string_map(&value)
            {
                return map;
            }
        }
    }
    HashMap::new()
}

fn json_object_to_string_map(value: &serde_json::Value) -> Option<HashMap<String, String>> {
    let obj = value.as_object()?;
    Some(
        obj.iter()
            .filter_map(|(k, v)| json_scalar_to_string(v).map(|s| (k.clone(), s)))
            .collect(),
    )
}

fn json_scalar_to_string(v: &serde_json::Value) -> Option<String> {
    match v {
        serde_json::Value::String(s) => Some(s.clone()),
        serde_json::Value::Number(n) => Some(n.to_string()),
        serde_json::Value::Bool(b) => Some(b.to_string()),
        _ => None,
    }
}

fn percent_decode(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%'
            && i + 2 < bytes.len()
            && let Ok(v) =
                u8::from_str_radix(std::str::from_utf8(&bytes[i + 1..i + 3]).unwrap_or(""), 16)
        {
            out.push(v);
            i += 3;
            continue;
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// Session id from the `environment` cookie for `key`.
///
/// Missing cookie key → `None` (caller should use the default session). Empty
/// value → `Some(None)` (all sessions). A positive id → `Some(Some(id))`.
pub fn selected_session(env: &HashMap<String, String>, key: &str) -> Option<Option<i64>> {
    match env.get(key) {
        None => None,
        Some(raw) if raw.trim().is_empty() => Some(None),
        Some(raw) => match raw.trim().parse::<i64>() {
            Ok(id) if id > 0 => Some(Some(id)),
            _ => None,
        },
    }
}

/// Session id to restrict academic-record lists, and whether to restrict at all.
///
/// Missing cookie key → default session (restrict). Empty value `"—"` → all sessions.
pub fn selected_academic_record_session(env: &HashMap<String, String>) -> Option<Option<i64>> {
    selected_session(env, ACADEMIC_RECORDS_SESSION_KEY)
}
