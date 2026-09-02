use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use lariv_rs::datetime::{format_date, parse_date};
use nirmancampus_common::optional_string;

pub fn opt_str(value: &Option<String>) -> &str {
    value.as_deref().unwrap_or("")
}

pub fn parse_optional_text(s: &str) -> Option<String> {
    optional_string(s)
}

pub fn parse_date_flexible(s: &str) -> Option<NaiveDate> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }
    const FMTS: &[&str] = &[
        "%d-%m-%Y",
        "%d/%m/%Y",
        "%Y-%m-%d",
        "%d-%m-%Y %H:%M:%S",
        "%d/%m/%Y %H:%M:%S",
        "%Y-%m-%d %H:%M:%S",
        "%Y-%m-%d %H:%M:%S%.f",
        "%Y-%m-%dT%H:%M:%S",
        "%Y-%m-%dT%H:%M:%S%.f",
    ];
    for fmt in FMTS {
        if let Ok(d) = NaiveDate::parse_from_str(s, fmt) {
            return Some(d);
        }
        if let Ok(dt) = NaiveDateTime::parse_from_str(s, fmt) {
            return Some(dt.date());
        }
    }
    parse_date(s)
}

pub fn parse_dod(s: &str) -> Option<NaiveDateTime> {
    Some(parse_date_flexible(s)?.and_time(NaiveTime::MIN))
}

pub fn format_dod(dt: Option<NaiveDateTime>) -> String {
    dt.map(|d| d.date().format("%d-%m-%Y").to_string())
        .unwrap_or_default()
}

pub fn format_dod_form(dt: Option<NaiveDateTime>) -> String {
    dt.map(|d| format_date(d.date())).unwrap_or_default()
}

pub fn parse_flag(s: &str) -> i8 {
    match s.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "y" | "on" => 1,
        _ => 0,
    }
}

pub fn flag_from_bool(v: bool) -> i8 {
    if v { 1 } else { 0 }
}

pub fn flag_bool(v: i8) -> bool {
    v != 0
}

pub fn flag_label(v: i8) -> &'static str {
    nirmancampus_common::ui::yes_no(v != 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn parses_dod_formats() {
        let expected = NaiveDate::from_ymd_opt(2019, 12, 7).unwrap();
        assert_eq!(parse_date_flexible("07-12-2019"), Some(expected));
        assert_eq!(parse_date_flexible("07/12/2019"), Some(expected));
        assert_eq!(parse_date_flexible("2019-12-07"), Some(expected));
        let dt = parse_dod("07-12-2019").expect("dod");
        assert_eq!(dt.date(), expected);
        assert_eq!(dt.time(), NaiveTime::MIN);
    }

    #[test]
    fn parses_flags() {
        assert_eq!(parse_flag("1"), 1);
        assert_eq!(parse_flag("true"), 1);
        assert_eq!(parse_flag("Yes"), 1);
        assert_eq!(parse_flag("0"), 0);
        assert_eq!(parse_flag("false"), 0);
        assert_eq!(parse_flag(""), 0);
        assert!(flag_bool(1));
        assert!(!flag_bool(0));
    }

    #[test]
    fn optional_text_trims_empty() {
        assert_eq!(parse_optional_text("  "), None);
        assert_eq!(parse_optional_text(" IGNOU "), Some("IGNOU".into()));
    }
}
