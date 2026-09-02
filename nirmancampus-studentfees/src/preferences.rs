use sea_orm::{ActiveModelTrait, ActiveValue::Set, DatabaseConnection, EntityTrait};

use crate::entities::preferences::{self, Entity as PreferencesEntity, Model as StudentFeesPreferences};

pub async fn load_preferences(db: &DatabaseConnection) -> anyhow::Result<StudentFeesPreferences> {
    if let Some(prefs) = PreferencesEntity::find_by_id(1).one(db).await? {
        return Ok(prefs);
    }
    let model = preferences::ActiveModel {
        id: Set(1),
        host: Set(String::new()),
        port: Set(3306),
        username: Set(String::new()),
        password: Set(String::new()),
        database: Set(String::new()),
    };
    Ok(model.insert(db).await?)
}

pub async fn save_preferences(
    db: &DatabaseConnection,
    host: String,
    port: i32,
    username: String,
    password: String,
    database: String,
) -> anyhow::Result<StudentFeesPreferences> {
    load_preferences(db).await?;
    let am = preferences::ActiveModel {
        id: Set(1),
        host: Set(host),
        port: Set(port),
        username: Set(username),
        password: Set(password),
        database: Set(database),
    };
    Ok(am.update(db).await?)
}
