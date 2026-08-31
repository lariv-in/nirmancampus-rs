use sea_orm::DatabaseConnection;

#[derive(Clone)]
pub struct AcademicRecordsState {
    pub db: DatabaseConnection,
}

impl AcademicRecordsState {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}
