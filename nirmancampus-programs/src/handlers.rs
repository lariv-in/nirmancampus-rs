pub mod programs;
pub mod structure;

use lariv_rs::components::ManyToManyItem;
use nirmancampus_courses::entities::course::{self, Entity as CourseEntity};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

pub(crate) async fn course_items_from_ids(
    db: &sea_orm::DatabaseConnection,
    ids: &[i64],
) -> Vec<ManyToManyItem> {
    if ids.is_empty() {
        return Vec::new();
    }
    let courses = CourseEntity::find()
        .filter(course::Column::DeletedAt.is_null())
        .filter(course::Column::Id.is_in(ids.to_vec()))
        .all(db)
        .await
        .unwrap_or_default();
    ids.iter()
        .filter_map(|id| {
            courses
                .iter()
                .find(|c| c.id == *id)
                .map(|c| ManyToManyItem::new(c.id.to_string(), c.name().to_string()))
        })
        .collect()
}

pub(crate) async fn course_codes_label(db: &sea_orm::DatabaseConnection, ids: &[i64]) -> String {
    if ids.is_empty() {
        return String::new();
    }
    let courses = CourseEntity::find()
        .filter(course::Column::DeletedAt.is_null())
        .filter(course::Column::Id.is_in(ids.to_vec()))
        .all(db)
        .await
        .unwrap_or_default();
    let codes: Vec<&str> = ids
        .iter()
        .filter_map(|id| courses.iter().find(|c| c.id == *id).map(|c| c.code()))
        .filter(|c| !c.is_empty())
        .collect();
    codes.join(", ")
}
