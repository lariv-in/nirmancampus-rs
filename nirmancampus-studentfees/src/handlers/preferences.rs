use axum::response::{IntoResponse, Response};

use crate::{
    forms::PreferencesForm,
    preferences::{load_preferences, save_preferences},
    state::StudentFeesState,
    templates::FeePreferencesPage,
};
use lariv_rs::{
    components::{SharedChromeFolder, SlotCtx},
    html_form::HtmlFormBody,
    http::Cap,
    plugins::users::middleware::RequireAuth,
    web::{Htmx, html_built_page_or_app_layout},
};

use super::forbid_non_admin;

fn page_from_prefs(
    prefs: &crate::entities::preferences::Model,
    error: String,
    message: String,
) -> FeePreferencesPage {
    FeePreferencesPage {
        host: prefs.host.clone(),
        port: prefs.port.to_string(),
        username: prefs.username.clone(),
        password: prefs.password.clone(),
        database: prefs.database.clone(),
        error,
        message,
    }
}

pub async fn get(
    Cap(state): Cap<StudentFeesState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
) -> Response {
    if let Some(resp) = forbid_non_admin(&ctx) {
        return resp;
    }
    let prefs = load_preferences(&state.app_db)
        .await
        .unwrap_or_else(|_| crate::entities::preferences::Model {
            id: 1,
            host: String::new(),
            port: 3306,
            username: String::new(),
            password: String::new(),
            database: String::new(),
        });
    let page = page_from_prefs(&prefs, String::new(), String::new());
    html_built_page_or_app_layout(&page, &htmx, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}

pub async fn post(
    Cap(state): Cap<StudentFeesState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    HtmlFormBody(form): HtmlFormBody<PreferencesForm>,
) -> Response {
    if let Some(resp) = forbid_non_admin(&ctx) {
        return resp;
    }
    let port = if form.port <= 0 { 3306 } else { form.port as i32 };
    let prefs = match save_preferences(
        &state.app_db,
        form.host.trim().to_string(),
        port,
        form.username.trim().to_string(),
        form.password.clone(),
        form.database.trim().to_string(),
    )
    .await
    {
        Ok(prefs) => prefs,
        Err(e) => {
            let page = FeePreferencesPage {
                host: form.host,
                port: port.to_string(),
                username: form.username,
                password: form.password,
                database: form.database,
                error: e.to_string(),
                message: String::new(),
            };
            return html_built_page_or_app_layout(&page, &htmx, &chrome, &SlotCtx::from_auth(&ctx))
                .into_response();
        }
    };
    let (error, message) = match state.reconnect().await {
        Ok(_) => (String::new(), "Connected to MySQL.".to_string()),
        Err(e) => (e.to_string(), String::new()),
    };
    let page = page_from_prefs(&prefs, error, message);
    html_built_page_or_app_layout(&page, &htmx, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}
