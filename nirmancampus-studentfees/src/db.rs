use std::time::Duration;

use sea_orm::{
    ConnectionTrait, ConnectOptions, Database, DatabaseBackend, DatabaseConnection, Statement,
};

use crate::entities::preferences::Model as StudentFeesPreferences;

fn encode_component(s: &str) -> String {
    let mut out = String::new();
    for b in s.as_bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(*b as char);
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

pub fn connection_url(prefs: &StudentFeesPreferences) -> String {
    let user = encode_component(prefs.username.trim());
    let password = encode_component(&prefs.password);
    let host = prefs.host.trim();
    let port = if prefs.port <= 0 { 3306 } else { prefs.port };
    let database = encode_component(prefs.database.trim());
    format!("mysql://{user}:{password}@{host}:{port}/{database}?ssl-mode=DISABLED&charset=utf8mb4")
}

pub async fn connect_mysql(url: &str) -> anyhow::Result<DatabaseConnection> {
    let mut opt = ConnectOptions::new(url.to_owned());
    opt.max_connections(5)
        .min_connections(1)
        .max_lifetime(Duration::from_secs(3600))
        .idle_timeout(Duration::from_secs(900))
        .sqlx_logging(false);
    let conn = Database::connect(opt).await?;
    conn.execute(Statement::from_string(
        DatabaseBackend::MySql,
        "SET NAMES utf8mb4 COLLATE utf8mb4_unicode_ci",
    ))
    .await?;
    conn.execute(Statement::from_string(
        DatabaseBackend::MySql,
        "SET time_zone = '+00:00'",
    ))
    .await?;
    conn.ping().await?;
    Ok(conn)
}

#[cfg(test)]
mod tests {
    use super::encode_component;

    #[test]
    fn encodes_special_characters() {
        assert_eq!(encode_component("a b"), "a%20b");
        assert_eq!(encode_component("p@ss:word"), "p%40ss%3Aword");
    }
}
