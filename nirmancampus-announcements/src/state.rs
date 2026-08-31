use sea_orm::DatabaseConnection;

#[derive(Clone)]
pub struct AnnouncementsState {
    pub db: DatabaseConnection,
}

impl AnnouncementsState {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}
