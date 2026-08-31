//! Seed Hindi / English / Punjabi program_media rows.

use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};

use crate::entities::program_media::{self, Entity as ProgramMediaEntity};

const LANGUAGES: [&str; 3] = ["Hindi", "English", "Punjabi"];

pub async fn seed(db: &DatabaseConnection) -> Result<(), sea_orm::DbErr> {
    for language in LANGUAGES {
        if ProgramMediaEntity::find()
            .filter(program_media::Column::DeletedAt.is_null())
            .filter(program_media::Column::Language.eq(language))
            .one(db)
            .await?
            .is_some()
        {
            continue;
        }
        let now = Utc::now();
        program_media::ActiveModel {
            id: Default::default(),
            created_at: Set(Some(now)),
            updated_at: Set(Some(now)),
            deleted_at: Set(None),
            language: Set(language.into()),
        }
        .insert(db)
        .await?;
    }
    Ok(())
}
