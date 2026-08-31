//! Load user names without decoding the full user row (password, phone, etc.).

use std::collections::HashMap;

use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QuerySelect};

use lariv_rs::plugins::users::entities::user::{self, Entity as UserEntity};

pub async fn load_user_names(
    db: &sea_orm::DatabaseConnection,
    ids: impl IntoIterator<Item = i64>,
) -> Result<HashMap<i64, String>, sea_orm::DbErr> {
    let mut ids: Vec<i64> = ids.into_iter().filter(|&id| id > 0).collect();
    ids.sort_unstable();
    ids.dedup();
    if ids.is_empty() {
        return Ok(HashMap::new());
    }
    let rows = UserEntity::find()
        .select_only()
        .column(user::Column::Id)
        .column(user::Column::Name)
        .filter(user::Column::Id.is_in(ids))
        .into_tuple::<(i64, String)>()
        .all(db)
        .await?;
    Ok(rows.into_iter().collect())
}

pub async fn user_display(db: &sea_orm::DatabaseConnection, user_id: i64) -> String {
    match load_user_names(db, [user_id]).await {
        Ok(mut names) => names
            .remove(&user_id)
            .unwrap_or_else(|| format!("User #{user_id}")),
        Err(e) => {
            tracing::error!(error = %e, user_id, "failed to load user name");
            format!("User #{user_id}")
        }
    }
}
