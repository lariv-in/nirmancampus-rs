use sea_orm::DatabaseConnection;

#[derive(Clone)]
pub struct StudentsState {
    pub db: DatabaseConnection,
}

impl StudentsState {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}
