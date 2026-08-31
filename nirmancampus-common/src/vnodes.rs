//! Look up filesystem vnode display names for foreign-key and many-to-many file fields.

use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};

use lariv_rs::components::ManyToManyItem;
use lariv_rs::plugins::filesystem::entities::filesystem_node::{
    Column as VNodeColumn, Entity as VNodeEntity,
};

/// Display name for a filesystem node, or `"File #{id}"` if the row is missing.
pub async fn vnode_name(db: &DatabaseConnection, id: i64) -> String {
    if id <= 0 {
        return String::new();
    }
    lariv_rs::web::opt_or_log(VNodeEntity::find_by_id(id).one(db).await, "find vnode")
        .map(|n| n.name)
        .unwrap_or_else(|| format!("File #{id}"))
}

/// Display name for an optional filesystem node id.
pub async fn vnode_name_opt(db: &DatabaseConnection, id: Option<i64>) -> String {
    match id.filter(|&id| id > 0) {
        Some(id) => vnode_name(db, id).await,
        None => String::new(),
    }
}

/// Many-to-many chips keyed by vnode id, labelled with each file's name.
pub async fn vnode_items(db: &DatabaseConnection, ids: &[i64]) -> Vec<ManyToManyItem> {
    let ids: Vec<i64> = ids.iter().copied().filter(|&id| id > 0).collect();
    if ids.is_empty() {
        return Vec::new();
    }
    let nodes = VNodeEntity::find()
        .filter(VNodeColumn::Id.is_in(ids.clone()))
        .all(db)
        .await
        .unwrap_or_default();
    ids.into_iter()
        .map(|id| {
            let name = nodes
                .iter()
                .find(|n| n.id == id)
                .map(|n| n.name.clone())
                .unwrap_or_else(|| format!("File #{id}"));
            ManyToManyItem::new(id.to_string(), name)
        })
        .collect()
}
