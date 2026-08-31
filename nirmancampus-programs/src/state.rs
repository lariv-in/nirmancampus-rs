use sea_orm::DatabaseConnection;

#[derive(Clone)]
pub struct ProgramsState {
    pub db: DatabaseConnection,
}

impl ProgramsState {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}
