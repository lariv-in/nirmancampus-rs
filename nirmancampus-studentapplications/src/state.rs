use sea_orm::DatabaseConnection;

#[derive(Clone)]
pub struct StudentApplicationsState {
    pub db: DatabaseConnection,
}

impl StudentApplicationsState {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}
