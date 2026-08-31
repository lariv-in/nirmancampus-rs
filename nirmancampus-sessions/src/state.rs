use sea_orm::DatabaseConnection;

#[derive(Clone)]
pub struct SessionsState {
    pub db: DatabaseConnection,
}

impl SessionsState {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}
