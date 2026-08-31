use sea_orm::DatabaseConnection;

#[derive(Clone)]
pub struct AssignmentSubmissionsState {
    pub db: DatabaseConnection,
}

impl AssignmentSubmissionsState {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}
