use sea_orm::DatabaseConnection;

#[derive(Clone)]
pub struct CoursesState {
    pub db: DatabaseConnection,
}

impl CoursesState {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}
