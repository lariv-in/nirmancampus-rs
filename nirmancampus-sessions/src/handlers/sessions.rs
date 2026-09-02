use axum::{
    extract::{Path, Query},
    http::Uri,
    response::{IntoResponse, Redirect, Response},
};
use chrono::{Datelike, TimeZone, Utc};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder,
};
use serde::Deserialize;

use crate::{
    entities::admission_session::{self, Entity as SessionEntity},
    forms::SessionForm,
    keys::{
        SessionCreateModalKey, SessionDeleteModalKey, SessionEditModalKey, SessionSelectModalKey,
        SessionSelectTableKey, SessionTableKey,
    },
    routes::SessionsDetailRouteTag,
    state::SessionsState,
    templates::{
        ConfirmDeletePage, SessionDetailPage, SessionFormPage, SessionListPage, SessionRow,
        SessionSelectPage,
    },
};
use lariv_rs::{
    components::{ObjectList, SharedChromeFolder, SlotCtx, SwapKey},
    datetime::{format_date_in_tz, parse_date},
    html_form::HtmlFormBody,
    http::Cap,
    picker::respond_picker_select,
    plugins::users::middleware::RequireAuth,
    template::RenderAppPane,
    web::{
        html_built_page_or_app_layout, html_built_page_with_slots, respond_create_modal_done_fk,
        respond_edit_modal_done, Htmx, ModalFormQuery,
    },
};
use nirmancampus_common::{is_admin, path_and_query};

const PAGE_SIZE: u32 = 20;

#[derive(Debug, Deserialize, Default)]
pub struct SessionListQuery {
    #[serde(default, rename = "Name", alias = "name")]
    pub name: Option<String>,
    #[serde(default, rename = "Code", alias = "code")]
    pub code: Option<String>,
    #[serde(default)]
    pub page: Option<u32>,
    #[serde(default)]
    pub sort: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct SessionSelectQuery {
    #[serde(flatten)]
    pub filter: SessionListQuery,
    #[serde(default)]
    pub target_input: Option<String>,
}

fn date_to_utc(s: &str) -> Option<chrono::DateTime<Utc>> {
    parse_date(s).and_then(|d| d.and_hms_opt(0, 0, 0).map(|ndt| Utc.from_utc_datetime(&ndt)))
}

async fn generate_session_month_code(
    db: &sea_orm::DatabaseConnection,
    start: chrono::DateTime<Utc>,
    exclude_id: Option<i64>,
) -> Result<String, sea_orm::DbErr> {
    let month_start = start
        .date_naive()
        .with_day(1)
        .unwrap_or(start.date_naive());
    let month_start_dt = Utc.from_utc_datetime(
        &month_start.and_hms_opt(0, 0, 0).unwrap_or_default(),
    );
    let month_end_dt = if month_start.month() == 12 {
        Utc.with_ymd_and_hms(month_start.year() + 1, 1, 1, 0, 0, 0)
            .single()
            .unwrap_or(month_start_dt)
    } else {
        Utc.with_ymd_and_hms(month_start.year(), month_start.month() + 1, 1, 0, 0, 0)
            .single()
            .unwrap_or(month_start_dt)
    };
    let mut query = SessionEntity::find()
        .filter(admission_session::Column::DeletedAt.is_null())
        .filter(admission_session::Column::Start.gte(month_start_dt))
        .filter(admission_session::Column::Start.lt(month_end_dt));
    if let Some(id) = exclude_id {
        query = query.filter(admission_session::Column::Id.ne(id));
    }
    let count = query.count(db).await?;
    let prefix = start.format("%B").to_string().to_uppercase();
    let prefix: String = prefix.chars().take(4).collect();
    Ok(format!("{}{}-{}", prefix, start.format("%Y"), count + 1))
}

async fn find_session(
    db: &sea_orm::DatabaseConnection,
    id: i64,
) -> Option<admission_session::Model> {
    lariv_rs::web::opt_or_log(
        SessionEntity::find_by_id(id)
            .filter(admission_session::Column::DeletedAt.is_null())
            .one(db)
            .await,
        "db find one",
    )
}

async fn query_sessions(
    db: &sea_orm::DatabaseConnection,
    q: &SessionListQuery,
    tz: &str,
) -> ObjectList<SessionRow> {
    let mut query = SessionEntity::find().filter(admission_session::Column::DeletedAt.is_null());
    if let Some(name) = q.name.as_ref().filter(|n| !n.is_empty()) {
        query = query.filter(admission_session::Column::Name.contains(name));
    }
    if let Some(code) = q.code.as_ref().filter(|n| !n.is_empty()) {
        query = query.filter(admission_session::Column::Code.contains(code));
    }
    let sort = q.sort.as_deref().unwrap_or("").trim();
    query = match sort {
        s if s.eq_ignore_ascii_case("Name DESC") => {
            query.order_by_desc(admission_session::Column::Name)
        }
        s if s.eq_ignore_ascii_case("Start DESC") => {
            query.order_by_desc(admission_session::Column::Start)
        }
        _ => query.order_by_desc(admission_session::Column::Id),
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
        .map(|s| SessionRow {
            id: s.id,
            name: s.name().to_string(),
            code: s.code().to_string(),
            start: s
                .start
                .map(|d| format_date_in_tz(d, tz))
                .unwrap_or_default(),
            end: s.end.map(|d| format_date_in_tz(d, tz)).unwrap_or_default(),
            is_active: s.is_active(),
        })
        .collect();
    ObjectList::from_page(rows, page, PAGE_SIZE, total)
}

pub async fn list(
    Cap(state): Cap<SessionsState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    uri: Uri,
    Query(q): Query<SessionListQuery>,
) -> maud::Markup {
    let sessions = query_sessions(&state.db, &q, &ctx.timezone).await;
    let page = SessionListPage {
        sessions,
        filter_name: q.name.clone().unwrap_or_default(),
        filter_code: q.code.clone().unwrap_or_default(),
        path_and_query: path_and_query(&uri),
        sort: q.sort.clone().unwrap_or_default(),
        is_admin: is_admin(&ctx),
    };
    if htmx.targets::<SessionTableKey>() {
        return page.render_table();
    }
    if htmx.wants_main_content() {
        return page.render_main().into();
    }
    if htmx.wants_app_layout() {
        return page.render_pane().into();
    }
    html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx))
}

pub async fn detail(
    Cap(state): Cap<SessionsState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Path(id): Path<i64>,
) -> Response {
    let Some(session) = find_session(&state.db, id).await else {
        return Redirect::to("/sessions/").into_response();
    };
    let page = SessionDetailPage {
        id: session.id,
        name: session.name().to_string(),
        code: session.code().to_string(),
        start: session
            .start
            .map(|d| format_date_in_tz(d, &ctx.timezone))
            .unwrap_or_default(),
        end: session
            .end
            .map(|d| format_date_in_tz(d, &ctx.timezone))
            .unwrap_or_default(),
        is_active: session.is_active(),
        is_admin: is_admin(&ctx),
    };
    html_built_page_or_app_layout(&page, &htmx, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}

fn empty_form(q: &ModalFormQuery) -> SessionFormPage {
    SessionFormPage {
        id: 0,
        name: String::new(),
        code: String::new(),
        start: String::new(),
        end: String::new(),
        is_active: true,
        error: String::new(),
        form_name: q.form_name(),
        refresh_table: q.refresh_table(),
        target_input: q.target_input(),
    }
}

pub async fn create_get(
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    Query(q): Query<ModalFormQuery>,
) -> maud::Markup {
    html_built_page_with_slots(&empty_form(&q), &chrome, &SlotCtx::from_auth(&ctx))
}

pub async fn create_post(
    Cap(state): Cap<SessionsState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Query(q): Query<ModalFormQuery>,
    HtmlFormBody(form): HtmlFormBody<SessionForm>,
) -> Response {
    if !is_admin(&ctx) {
        return Redirect::to("/sessions/").into_response();
    }
    let start = date_to_utc(&form.start);
    let end = date_to_utc(&form.end);
    let mut code = form.code.trim().to_string();
    if code.is_empty() {
        if let Some(start) = start {
            match generate_session_month_code(&state.db, start, None).await {
                Ok(c) => code = c,
                Err(e) => {
                    let mut page = empty_form(&q);
                    page.name = form.name;
                    page.start = form.start;
                    page.end = form.end;
                    page.is_active = form.is_active;
                    page.error = e.to_string();
                    return html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx))
                        .into_response();
                }
            }
        }
    }
    let now = Utc::now();
    let model = admission_session::ActiveModel {
        id: Default::default(),
        created_at: Set(Some(now)),
        updated_at: Set(Some(now)),
        deleted_at: Set(None),
        name: Set(Some(form.name.clone())),
        code: Set(Some(code)),
        start: Set(start),
        end: Set(end),
        is_active: Set(Some(form.is_active)),
    };
    match model.insert(&state.db).await {
        Ok(saved) => respond_create_modal_done_fk::<SessionCreateModalKey>(
            &htmx,
            &q.refresh_table(),
            &SessionsDetailRouteTag::new(saved.id).url(),
            saved.id,
            saved.name(),
            &q.target_input(),
        ),
        Err(e) => {
            let mut page = empty_form(&q);
            page.name = form.name;
            page.code = form.code;
            page.start = form.start;
            page.end = form.end;
            page.is_active = form.is_active;
            page.error = e.to_string();
            html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
        }
    }
}

pub async fn edit_get(
    Cap(state): Cap<SessionsState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    Path(id): Path<i64>,
    Query(q): Query<ModalFormQuery>,
) -> Response {
    let Some(session) = find_session(&state.db, id).await else {
        return Redirect::to("/sessions/").into_response();
    };
    let page = SessionFormPage {
        id: session.id,
        name: session.name().to_string(),
        code: session.code().to_string(),
        start: session
            .start
            .map(|d| format_date_in_tz(d, &ctx.timezone))
            .unwrap_or_default(),
        end: session
            .end
            .map(|d| format_date_in_tz(d, &ctx.timezone))
            .unwrap_or_default(),
        is_active: session.is_active(),
        error: String::new(),
        form_name: q.form_name(),
        refresh_table: String::new(),
        target_input: String::new(),
    };
    html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}

pub async fn edit_post(
    Cap(state): Cap<SessionsState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Path(id): Path<i64>,
    Query(q): Query<ModalFormQuery>,
    HtmlFormBody(form): HtmlFormBody<SessionForm>,
) -> Response {
    if !is_admin(&ctx) {
        return Redirect::to("/sessions/").into_response();
    }
    let Some(existing) = find_session(&state.db, id).await else {
        return Redirect::to("/sessions/").into_response();
    };
    let start = date_to_utc(&form.start);
    let end = date_to_utc(&form.end);
    let mut code = form.code.trim().to_string();
    if code.is_empty() {
        if let Some(start) = start {
            if let Ok(c) = generate_session_month_code(&state.db, start, Some(id)).await {
                code = c;
            }
        }
    }
    let now = Utc::now();
    let model = admission_session::ActiveModel {
        id: Set(existing.id),
        created_at: Set(existing.created_at),
        updated_at: Set(Some(now)),
        deleted_at: Set(existing.deleted_at),
        name: Set(Some(form.name.clone())),
        code: Set(Some(code)),
        start: Set(start),
        end: Set(end),
        is_active: Set(Some(form.is_active)),
    };
    match model.update(&state.db).await {
        Ok(_) => respond_edit_modal_done::<SessionEditModalKey>(
            &htmx,
            &SessionsDetailRouteTag::new(id).url(),
        ),
        Err(e) => {
            let page = SessionFormPage {
                id,
                name: form.name,
                code: form.code,
                start: form.start,
                end: form.end,
                is_active: form.is_active,
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
) -> maud::Markup {
    let page = ConfirmDeletePage {
        modal_uid: SessionDeleteModalKey::ID.to_string(),
        message: format!("Delete session #{id}?"),
        id,
        error: String::new(),
    };
    html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx))
}

pub async fn delete_post(
    Cap(state): Cap<SessionsState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    Path(id): Path<i64>,
) -> Response {
    if !is_admin(&ctx) {
        return Redirect::to("/sessions/").into_response();
    }
    let Some(existing) = find_session(&state.db, id).await else {
        return Redirect::to("/sessions/").into_response();
    };
    let now = Utc::now();
    let model = admission_session::ActiveModel {
        id: Set(existing.id),
        deleted_at: Set(Some(now)),
        ..Default::default()
    };
    match model.update(&state.db).await {
        Ok(_) => Redirect::to("/sessions/").into_response(),
        Err(e) => {
            let page = ConfirmDeletePage {
                modal_uid: SessionDeleteModalKey::ID.to_string(),
                message: format!("Delete session #{id}?"),
                id,
                error: e.to_string(),
            };
            html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
        }
    }
}

pub async fn select(
    Cap(state): Cap<SessionsState>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    uri: Uri,
    Query(q): Query<SessionSelectQuery>,
) -> maud::Markup {
    let sessions = query_sessions(&state.db, &q.filter, &ctx.timezone).await;
    let page = SessionSelectPage {
        sessions,
        filter_name: q.filter.name.clone().unwrap_or_default(),
        filter_code: q.filter.code.clone().unwrap_or_default(),
        path_and_query: path_and_query(&uri),
        target_input: q.target_input.unwrap_or_default(),
    };
    respond_picker_select::<SessionSelectTableKey, SessionSelectModalKey, _>(&htmx, &page)
}
