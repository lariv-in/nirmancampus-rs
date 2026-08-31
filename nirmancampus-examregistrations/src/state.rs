use sea_orm::DatabaseConnection;

#[derive(Clone)]
pub struct ExamRegistrationsState {
    pub db: DatabaseConnection,
}

impl ExamRegistrationsState {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}
