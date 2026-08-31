use axum::{
    extract::Path,
    response::{IntoResponse, Response},
};
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter};

use crate::{
    entities::contact_page_settings::{self, Entity as SettingsEntity},
    forms::ContactPageSettingsForm,
    handlers::{file_opt, forbid_non_admin},
    state::{CONTACT_PAGE_SETTINGS_ID, WebsiteState},
    templates::ContactPageSettingsFormPage,
};
use lariv_rs::{
    components::{SharedChromeFolder, SlotCtx},
    html_form::HtmlFormBody,
    http::Cap,
    plugins::users::middleware::RequireAuth,
    web::{Htmx, html_built_page_or_app_layout, html_built_page_with_slots},
};
use nirmancampus_common::vnode_name;

async fn load_or_seed(db: &sea_orm::DatabaseConnection) -> contact_page_settings::Model {
    if let Some(row) = lariv_rs::web::opt_or_log(
        SettingsEntity::find_by_id(CONTACT_PAGE_SETTINGS_ID)
            .filter(contact_page_settings::Column::DeletedAt.is_null())
            .one(db)
            .await,
        "db find contact page settings",
    ) {
        return row;
    }
    let now = Utc::now();
    let model = contact_page_settings::ActiveModel {
        id: Set(CONTACT_PAGE_SETTINGS_ID),
        created_at: Set(Some(now)),
        updated_at: Set(Some(now)),
        deleted_at: Set(None),
        essential_committees_list_file_id: Set(None),
    };
    model
        .insert(db)
        .await
        .unwrap_or(contact_page_settings::Model {
            id: CONTACT_PAGE_SETTINGS_ID,
            created_at: Some(now),
            updated_at: Some(now),
            deleted_at: None,
            essential_committees_list_file_id: None,
        })
}

async fn form_page(
    db: &sea_orm::DatabaseConnection,
    row: &contact_page_settings::Model,
    error: String,
) -> ContactPageSettingsFormPage {
    let file_id = row.essential_committees_list_file_id.unwrap_or(0);
    ContactPageSettingsFormPage {
        id: row.id,
        file_id,
        file_display: vnode_name(db, file_id).await,
        error,
    }
}

pub async fn detail(
    Cap(state): Cap<WebsiteState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Path(_id): Path<i64>,
) -> Response {
    if let Some(resp) = forbid_non_admin(&ctx) {
        return resp;
    }
    let row = load_or_seed(&state.db).await;
    let page = form_page(&state.db, &row, String::new()).await;
    html_built_page_or_app_layout(&page, &htmx, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}

pub async fn edit_get(
    Cap(state): Cap<WebsiteState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Path(_id): Path<i64>,
) -> Response {
    if let Some(resp) = forbid_non_admin(&ctx) {
        return resp;
    }
    let row = load_or_seed(&state.db).await;
    let page = form_page(&state.db, &row, String::new()).await;
    html_built_page_or_app_layout(&page, &htmx, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}

pub async fn edit_post(
    Cap(state): Cap<WebsiteState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Path(_id): Path<i64>,
    HtmlFormBody(form): HtmlFormBody<ContactPageSettingsForm>,
) -> Response {
    if let Some(resp) = forbid_non_admin(&ctx) {
        return resp;
    }
    let existing = load_or_seed(&state.db).await;
    let now = Utc::now();
    let model = contact_page_settings::ActiveModel {
        id: Set(existing.id),
        created_at: Set(existing.created_at),
        updated_at: Set(Some(now)),
        deleted_at: Set(existing.deleted_at),
        essential_committees_list_file_id: Set(file_opt(form.essential_committees_list_file_id)),
    };
    match model.update(&state.db).await {
        Ok(_) => htmx.redirect("/website/contact-page/settings/1/"),
        Err(e) => {
            let mut page = form_page(&state.db, &existing, e.to_string()).await;
            page.file_id = form.essential_committees_list_file_id;
            page.file_display = vnode_name(&state.db, form.essential_committees_list_file_id).await;
            html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
        }
    }
}
