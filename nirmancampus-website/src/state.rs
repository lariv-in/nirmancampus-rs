use sea_orm::DatabaseConnection;

#[derive(Clone)]
pub struct WebsiteState {
    pub db: DatabaseConnection,
    pub static_dir: String,
}

impl WebsiteState {
    pub fn new(db: DatabaseConnection, static_dir: String) -> Self {
        Self { db, static_dir }
    }
}

pub const CONTACT_PAGE_SETTINGS_ID: i64 = 1;
