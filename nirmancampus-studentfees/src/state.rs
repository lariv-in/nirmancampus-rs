use std::sync::Arc;

use sea_orm::DatabaseConnection;
use tokio::sync::RwLock;

use crate::{
    db::{connect_mysql, connection_url},
    preferences::load_preferences,
};

#[derive(Clone)]
pub struct StudentFeesState {
    pub app_db: DatabaseConnection,
    mysql: Arc<RwLock<Option<DatabaseConnection>>>,
}

impl StudentFeesState {
    pub fn new(app_db: DatabaseConnection) -> Self {
        Self {
            app_db,
            mysql: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn mysql(&self) -> anyhow::Result<DatabaseConnection> {
        {
            let guard = self.mysql.read().await;
            if let Some(conn) = guard.as_ref()
                && conn.ping().await.is_ok()
            {
                return Ok(conn.clone());
            }
        }
        self.reconnect().await
    }

    pub async fn reconnect(&self) -> anyhow::Result<DatabaseConnection> {
        let prefs = load_preferences(&self.app_db).await?;
        if prefs.host.trim().is_empty() || prefs.database.trim().is_empty() {
            anyhow::bail!("MySQL host and database are required. Set them in Preferences.");
        }
        let url = connection_url(&prefs);
        let conn = connect_mysql(&url).await?;
        *self.mysql.write().await = Some(conn.clone());
        Ok(conn)
    }
}
