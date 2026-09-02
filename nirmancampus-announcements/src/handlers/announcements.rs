use axum::{
    extract::{Path, Query},
    http::Uri,
    response::{IntoResponse, Redirect, Response},
};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, Condition, EntityTrait, PaginatorTrait,
    QueryFilter, QueryOrder, sea_query::Expr,
};
use serde::Deserialize;

use crate::{
    entities::{
        announcement::{self, Entity as AnnouncementEntity},
        announcement_asset::{self, Entity as AnnouncementAssetEntity},
    },
    forms::AnnouncementForm,
    keys::{
        AnnouncementCreateModalKey, AnnouncementDeleteModalKey, AnnouncementEditModalKey,
        AnnouncementSelectModalKey, AnnouncementSelectTableKey, AnnouncementTableKey,
    },
    routes::AnnouncementsDetailRouteTag,
    state::AnnouncementsState,
    templates::{
        AnnouncementDetailPage, AnnouncementFormPage, AnnouncementListPage, AnnouncementRow,
        AnnouncementSelectPage, ConfirmDeletePage,
    },
};
use lariv_rs::{
    components::{ManyToManyItem, ObjectList, SharedChromeFolder, SlotCtx, SwapKey},
    html_form::HtmlFormBody,
    http::Cap,
    picker::respond_picker_select,
    plugins::users::middleware::RequireAuth,
    web::{
        html_built_page_or_app_layout, html_built_page_with_slots, respond_create_modal_done_fk,
        respond_edit_modal_done, Htmx, ModalFormQuery,
    },
};
use nirmancampus_common::{can_view_campus_records, is_admin, is_student, optional_string, path_and_query, vnode_items};

const PAGE_SIZE: u32 = 20;

#[derive(Debug, Deserialize, Default)]
pub struct AnnouncementListQuery {
    #[serde(default, rename = "Title", alias = "title")]
    pub title: Option<String>,
    #[serde(default, rename = "Description", alias = "description")]
    pub description: Option<String>,
    #[serde(default)]
    pub page: Option<u32>,
    #[serde(default)]
    pub sort: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct AnnouncementSelectQuery {
    #[serde(flatten)]
    pub filter: AnnouncementListQuery,
    #[serde(default)]
    pub target_input: Option<String>,
}

fn forbid_non_admin(ctx: &lariv_rs::plugins::users::state::AuthContext) -> Option<Response> {
    if is_admin(ctx) {
        None
    } else {
        Some(Redirect::to("/").into_response())
    }
}

fn forbid_if_no_access(ctx: &lariv_rs::plugins::users::state::AuthContext) -> Option<Response> {
    if can_view_campus_records(ctx) {
        None
    } else {
        Some(Redirect::to("/").into_response())
    }
}

fn scope_announcements(
    query: sea_orm::Select<AnnouncementEntity>,
    auth: &lariv_rs::plugins::users::state::AuthContext,
) -> sea_orm::Select<AnnouncementEntity> {
    if is_admin(auth) {
        return query;
    }
    if is_student(auth) {
        let now = Utc::now();
        return query.filter(announcement::Column::ReleaseAt.lte(now)).filter(
            Condition::any()
                .add(announcement::Column::ExpiryAt.is_null())
                .add(announcement::Column::ExpiryAt.gt(now)),
        );
    }
    query.filter(Expr::cust("1 = 0"))
}

async fn find_announcement(
    db: &sea_orm::DatabaseConnection,
    id: i64,
) -> Option<announcement::Model> {
    lariv_rs::web::opt_or_log(
        AnnouncementEntity::find_by_id(id)
            .filter(announcement::Column::DeletedAt.is_null())
            .one(db)
            .await,
        "db find one",
    )
}

async fn load_asset_ids(db: &sea_orm::DatabaseConnection, announcement_id: i64) -> Vec<i64> {
    AnnouncementAssetEntity::find()
        .filter(announcement_asset::Column::AnnouncementId.eq(announcement_id))
        .all(db)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|a| a.v_node_id)
        .collect()
}

async fn replace_assets(
    db: &sea_orm::DatabaseConnection,
    announcement_id: i64,
    ids: &[i64],
) -> Result<(), sea_orm::DbErr> {
    AnnouncementAssetEntity::delete_many()
        .filter(announcement_asset::Column::AnnouncementId.eq(announcement_id))
        .exec(db)
        .await?;
    for &v_node_id in ids {
        if v_node_id <= 0 {
            continue;
        }
        announcement_asset::ActiveModel {
            announcement_id: Set(announcement_id),
            v_node_id: Set(v_node_id),
        }
        .insert(db)
        .await?;
    }
    Ok(())
}

async fn asset_items(db: &sea_orm::DatabaseConnection, ids: &[i64]) -> Vec<ManyToManyItem> {
    vnode_items(db, ids).await
}

async fn query_announcements(
    db: &sea_orm::DatabaseConnection,
    q: &AnnouncementListQuery,
    tz: &str,
    auth: &lariv_rs::plugins::users::state::AuthContext,
) -> ObjectList<AnnouncementRow> {
    let mut query = AnnouncementEntity::find().filter(announcement::Column::DeletedAt.is_null());
    query = scope_announcements(query, auth);
    if let Some(title) = q.title.as_ref().filter(|n| !n.is_empty()) {
        query = query.filter(announcement::Column::Title.contains(title));
    }
    if let Some(description) = q.description.as_ref().filter(|n| !n.is_empty()) {
        query = query.filter(announcement::Column::Description.contains(description));
    }
    let sort = q.sort.as_deref().unwrap_or("").trim();
    query = match sort {
        s if s.eq_ignore_ascii_case("Title DESC") => {
            query.order_by_desc(announcement::Column::Title)
        }
        s if s.eq_ignore_ascii_case("Title ASC") || s.eq_ignore_ascii_case("Title") => {
            query.order_by_asc(announcement::Column::Title)
        }
        s if s.eq_ignore_ascii_case("ReleaseAt DESC") => {
            query.order_by_desc(announcement::Column::ReleaseAt)
        }
        _ => query.order_by_desc(announcement::Column::Id),
    };
    let page = q.page.unwrap_or(1).max(1);
    let paginator = query.paginate(db, PAGE_SIZE as u64);
    let total = paginator.num_items().await.unwrap_or(0);
    let models = paginator
        .fetch_page((page as u64).saturating_sub(1))
        .await
        .unwrap_or_default();
    let rows = models
        .into_iter()
        .map(|a| AnnouncementRow {
            id: a.id,
            title: a.title().to_string(),
            url: a.url().to_string(),
            release_at: a
                .release_at
                .map(|d| format_dt(d, tz))
                .unwrap_or_default(),
            expiry_at: a.expiry_at.map(|d| format_dt(d, tz)).unwrap_or_default(),
        })
        .collect();
    ObjectList::from_page(rows, page, PAGE_SIZE, total)
}

fn format_dt(dt: chrono::DateTime<Utc>, tz: &str) -> String {
    lariv_rs::datetime::DatetimeLabel::short(dt, tz).into_string()
}

fn parse_release(
    ctx: &lariv_rs::plugins::users::state::AuthContext,
    raw: &str,
) -> chrono::DateTime<Utc> {
    ctx.parse_datetime_local_input(raw)
        .unwrap_or_else(Utc::now)
}

fn empty_form(q: &ModalFormQuery, ctx: &lariv_rs::plugins::users::state::AuthContext) -> AnnouncementFormPage {
    AnnouncementFormPage {
        id: 0,
        title: String::new(),
        description: String::new(),
        url: String::new(),
        release_at: ctx.datetime_local_input(Utc::now()).into_string(),
        expiry_at: String::new(),
        assets: Vec::new(),
        error: String::new(),
        form_name: q.form_name(),
        refresh_table: q.refresh_table(),
        target_input: q.target_input(),
    }
}

pub async fn list(
    Cap(state): Cap<AnnouncementsState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    uri: Uri,
    Query(q): Query<AnnouncementListQuery>,
) -> Response {
    if let Some(resp) = forbid_if_no_access(&ctx) {
        return resp;
    }
    let announcements = query_announcements(&state.db, &q, &ctx.timezone, &ctx).await;
    let page = AnnouncementListPage {
        announcements,
        filter_title: q.title.clone().unwrap_or_default(),
        filter_description: q.description.clone().unwrap_or_default(),
        path_and_query: path_and_query(&uri),
        sort: q.sort.clone().unwrap_or_default(),
        is_admin: is_admin(&ctx),
    };
    if htmx.targets::<AnnouncementTableKey>() {
        return page.render_table().into_response();
    }
    html_built_page_or_app_layout(&page, &htmx, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}

pub async fn detail(
    Cap(state): Cap<AnnouncementsState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Path(id): Path<i64>,
) -> Response {
    if let Some(resp) = forbid_if_no_access(&ctx) {
        return resp;
    }
    let Some(row) = find_announcement(&state.db, id).await else {
        return Redirect::to("/announcements/").into_response();
    };
    if !is_admin(&ctx) {
        let now = Utc::now();
        let released = row.release_at.map(|t| t <= now).unwrap_or(false);
        let unexpired = row.expiry_at.map(|t| t > now).unwrap_or(true);
        if !released || !unexpired {
            return Redirect::to("/announcements/").into_response();
        }
    }
    let page = AnnouncementDetailPage {
        id: row.id,
        title: row.title().to_string(),
        description: row.description().to_string(),
        url: row.url().to_string(),
        release_at: row
            .release_at
            .map(|d| format_dt(d, &ctx.timezone))
            .unwrap_or_default(),
        expiry_at: row
            .expiry_at
            .map(|d| format_dt(d, &ctx.timezone))
            .unwrap_or_default(),
        assets: asset_items(&state.db, &load_asset_ids(&state.db, id).await).await,
        is_admin: is_admin(&ctx),
    };
    html_built_page_or_app_layout(&page, &htmx, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}

pub async fn create_get(
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    Query(q): Query<ModalFormQuery>,
) -> Response {
    if let Some(resp) = forbid_non_admin(&ctx) {
        return resp;
    }
    html_built_page_with_slots(&empty_form(&q, &ctx), &chrome, &SlotCtx::from_auth(&ctx))
        .into_response()
}

pub async fn create_post(
    Cap(state): Cap<AnnouncementsState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Query(q): Query<ModalFormQuery>,
    HtmlFormBody(form): HtmlFormBody<AnnouncementForm>,
) -> Response {
    if let Some(resp) = forbid_non_admin(&ctx) {
        return resp;
    }
    let now = Utc::now();
    let release_at = if form.release_at.trim().is_empty() {
        now
    } else {
        parse_release(&ctx, &form.release_at)
    };
    let expiry_at = ctx.parse_datetime_local_input(&form.expiry_at);
    let model = announcement::ActiveModel {
        id: Default::default(),
        created_at: Set(Some(now)),
        updated_at: Set(Some(now)),
        deleted_at: Set(None),
        title: Set(form.title.clone()),
        description: Set(form.description.clone()),
        url: Set(optional_string(&form.url)),
        created_by_id: Set(Some(ctx.user.id)),
        release_at: Set(Some(release_at)),
        expiry_at: Set(expiry_at),
    };
    match model.insert(&state.db).await {
        Ok(saved) => {
            if let Err(e) = replace_assets(&state.db, saved.id, &form.assets).await {
                tracing::error!(error = %e, id = saved.id, "failed to save announcement assets");
            }
            respond_create_modal_done_fk::<AnnouncementCreateModalKey>(
                &htmx,
                &q.refresh_table(),
                &AnnouncementsDetailRouteTag::new(saved.id).url(),
                saved.id,
                saved.title(),
                &q.target_input(),
            )
        }
        Err(e) => {
            let mut page = empty_form(&q, &ctx);
            page.title = form.title;
            page.description = form.description;
            page.url = form.url;
            page.release_at = form.release_at;
            page.expiry_at = form.expiry_at;
            page.assets = asset_items(&state.db, &form.assets).await;
            page.error = e.to_string();
            html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
        }
    }
}

pub async fn edit_get(
    Cap(state): Cap<AnnouncementsState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    Path(id): Path<i64>,
    Query(q): Query<ModalFormQuery>,
) -> Response {
    if let Some(resp) = forbid_non_admin(&ctx) {
        return resp;
    }
    let Some(row) = find_announcement(&state.db, id).await else {
        return Redirect::to("/announcements/").into_response();
    };
    let ids = load_asset_ids(&state.db, id).await;
    let page = AnnouncementFormPage {
        id: row.id,
        title: row.title().to_string(),
        description: row.description().to_string(),
        url: row.url().to_string(),
        release_at: row
            .release_at
            .map(|d| ctx.datetime_local_input(d).into_string())
            .unwrap_or_default(),
        expiry_at: row
            .expiry_at
            .map(|d| ctx.datetime_local_input(d).into_string())
            .unwrap_or_default(),
        assets: asset_items(&state.db, &ids).await,
        error: String::new(),
        form_name: q.form_name(),
        refresh_table: String::new(),
        target_input: String::new(),
    };
    html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}

pub async fn edit_post(
    Cap(state): Cap<AnnouncementsState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Path(id): Path<i64>,
    Query(q): Query<ModalFormQuery>,
    HtmlFormBody(form): HtmlFormBody<AnnouncementForm>,
) -> Response {
    if let Some(resp) = forbid_non_admin(&ctx) {
        return resp;
    }
    let Some(existing) = find_announcement(&state.db, id).await else {
        return Redirect::to("/announcements/").into_response();
    };
    let now = Utc::now();
    let release_at = if form.release_at.trim().is_empty() {
        existing.release_at.unwrap_or(now)
    } else {
        parse_release(&ctx, &form.release_at)
    };
    let expiry_at = ctx.parse_datetime_local_input(&form.expiry_at);
    let model = announcement::ActiveModel {
        id: Set(existing.id),
        created_at: Set(existing.created_at),
        updated_at: Set(Some(now)),
        deleted_at: Set(existing.deleted_at),
        title: Set(form.title.clone()),
        description: Set(form.description.clone()),
        url: Set(optional_string(&form.url)),
        created_by_id: Set(existing.created_by_id),
        release_at: Set(Some(release_at)),
        expiry_at: Set(expiry_at),
    };
    match model.update(&state.db).await {
        Ok(_) => {
            if let Err(e) = replace_assets(&state.db, id, &form.assets).await {
                tracing::error!(error = %e, id, "failed to save announcement assets");
            }
            respond_edit_modal_done::<AnnouncementEditModalKey>(
                &htmx,
                &AnnouncementsDetailRouteTag::new(id).url(),
            )
        }
        Err(e) => {
            let page = AnnouncementFormPage {
                id,
                title: form.title,
                description: form.description,
                url: form.url,
                release_at: form.release_at,
                expiry_at: form.expiry_at,
                assets: asset_items(&state.db, &form.assets).await,
                error: e.to_string(),
                form_name: q.form_name(),
                refresh_table: String::new(),
                target_input: String::new(),
            };
            html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
        }
    }
}

pub async fn delete_get(
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    Path(id): Path<i64>,
) -> Response {
    if let Some(resp) = forbid_non_admin(&ctx) {
        return resp;
    }
    let page = ConfirmDeletePage {
        modal_uid: AnnouncementDeleteModalKey::ID.to_string(),
        message: format!("Delete announcement #{id}?"),
        form_name: "announcements.AnnouncementDeleteForm".into(),
        id,
        error: String::new(),
    };
    html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}

pub async fn delete_post(
    Cap(state): Cap<AnnouncementsState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    Path(id): Path<i64>,
) -> Response {
    if let Some(resp) = forbid_non_admin(&ctx) {
        return resp;
    }
    let Some(existing) = find_announcement(&state.db, id).await else {
        return Redirect::to("/announcements/").into_response();
    };
    let now = Utc::now();
    let model = announcement::ActiveModel {
        id: Set(existing.id),
        deleted_at: Set(Some(now)),
        ..Default::default()
    };
    match model.update(&state.db).await {
        Ok(_) => Redirect::to("/announcements/").into_response(),
        Err(e) => {
            let page = ConfirmDeletePage {
                modal_uid: AnnouncementDeleteModalKey::ID.to_string(),
                message: format!("Delete announcement #{id}?"),
                form_name: "announcements.AnnouncementDeleteForm".into(),
                id,
                error: e.to_string(),
            };
            html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
        }
    }
}

pub async fn select(
    Cap(state): Cap<AnnouncementsState>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    uri: Uri,
    Query(q): Query<AnnouncementSelectQuery>,
) -> Response {
    if let Some(resp) = forbid_if_no_access(&ctx) {
        return resp;
    }
    let announcements = query_announcements(&state.db, &q.filter, &ctx.timezone, &ctx).await;
    let page = AnnouncementSelectPage {
        announcements,
        filter_title: q.filter.title.clone().unwrap_or_default(),
        path_and_query: path_and_query(&uri),
        target_input: q.target_input.unwrap_or_default(),
    };
    respond_picker_select::<AnnouncementSelectTableKey, AnnouncementSelectModalKey, _>(&htmx, &page)
        .into_response()
}
