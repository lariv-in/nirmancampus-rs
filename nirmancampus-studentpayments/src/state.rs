use sea_orm::DatabaseConnection;

#[derive(Clone)]
pub struct StudentPaymentsState {
    pub db: DatabaseConnection,
}

impl StudentPaymentsState {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}
