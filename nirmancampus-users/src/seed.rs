//! Seed Nirmancampus roles (`admin`, `student`, `unassigned`).

use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};

use lariv_rs::plugins::users::entities::role::{self, Entity as RoleEntity};
use nirmancampus_common::{ROLE_ADMIN, ROLE_STUDENT, ROLE_UNASSIGNED};

#[derive(Clone)]
pub struct NirmancampusUsersState {
    pub db: DatabaseConnection,
}

impl NirmancampusUsersState {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

async fn ensure_role(db: &DatabaseConnection, name: &str) -> Result<(), sea_orm::DbErr> {
    if RoleEntity::find()
        .filter(role::Column::Name.eq(name))
        .one(db)
        .await?
        .is_some()
    {
        return Ok(());
    }
    let now = Utc::now();
    role::ActiveModel {
        id: Default::default(),
        created_at: Set(Some(now)),
        updated_at: Set(Some(now)),
        name: Set(name.into()),
    }
    .insert(db)
    .await?;
    Ok(())
}

pub async fn seed(state: &NirmancampusUsersState) -> Result<(), sea_orm::DbErr> {
    ensure_role(&state.db, ROLE_ADMIN).await?;
    ensure_role(&state.db, ROLE_STUDENT).await?;
    ensure_role(&state.db, ROLE_UNASSIGNED).await?;
    Ok(())
}
