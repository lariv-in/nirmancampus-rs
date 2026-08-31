use axum::{
    extract::{Path, Query},
    http::Uri,
    response::{IntoResponse, Redirect, Response},
};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder,
};
use serde::Deserialize;

use crate::{
    entities::important_link::{self, Entity as ImportantLinkEntity},
    forms::ImportantLinkForm,
    handlers::{file_opt, forbid_non_admin},
    keys::{
        ImportantLinkCreateModalKey, ImportantLinkDeleteModalKey, ImportantLinkEditModalKey,
        ImportantLinksTableKey,
    },
    routes::WebsiteImportantLinksDetailRouteTag,
    state::WebsiteState,
    templates::{
        ConfirmDeletePage, ImportantLinkDetailPage, ImportantLinkFormPage, ImportantLinkListPage,
        ImportantLinkRow,
    },
};
use lariv_rs::{
    components::{ObjectList, SharedChromeFolder, SlotCtx, SwapKey},
    html_form::HtmlFormBody,
    http::Cap,
    plugins::users::middleware::RequireAuth,
    web::{
        Htmx, ModalFormQuery, html_built_page_or_app_layout, html_built_page_with_slots,
        respond_create_modal_done_fk, respond_edit_modal_done,
    },
};
use nirmancampus_common::ui::yes_no;
use nirmancampus_common::{is_admin, path_and_query, vnode_name, vnode_name_opt};

const PAGE_SIZE: u32 = 20;

#[derive(Debug, Deserialize, Default)]
pub struct ImportantLinkListQuery {
    #[serde(default, rename = "Title", alias = "title")]
    pub title: Option<String>,
    #[serde(default)]
    pub page: Option<u32>,
}

async fn find_link(db: &sea_orm::DatabaseConnection, id: i64) -> Option<important_link::Model> {
    lariv_rs::web::opt_or_log(
        ImportantLinkEntity::find_by_id(id)
            .filter(important_link::Column::DeletedAt.is_null())
            .one(db)
            .await,
        "db find important link",
    )
}

async fn query_links(
    db: &sea_orm::DatabaseConnection,
    q: &ImportantLinkListQuery,
) -> ObjectList<ImportantLinkRow> {
    let mut query = ImportantLinkEntity::find().filter(important_link::Column::DeletedAt.is_null());
    if let Some(title) = q.title.as_ref().filter(|n| !n.is_empty()) {
        query = query.filter(important_link::Column::Title.contains(title));
    }
    query = query
        .order_by_asc(important_link::Column::Order)
        .order_by_asc(important_link::Column::Id);
    let page = q.page.unwrap_or(1).max(1);
    let paginator = query.paginate(db, PAGE_SIZE as u64);
    let total = paginator.num_items().await.unwrap_or(0);
    let models = paginator
        .fetch_page((page as u64).saturating_sub(1))
        .await
        .unwrap_or_default();
    let rows = models
        .into_iter()
        .map(|l| ImportantLinkRow {
            id: l.id,
            title: l.title.clone(),
            order: l.order,
            is_link: yes_no(l.is_link()).to_string(),
            link: l.link().to_string(),
        })
        .collect();
    ObjectList::from_page(rows, page, PAGE_SIZE, total)
}

fn empty_form(q: &ModalFormQuery) -> ImportantLinkFormPage {
    ImportantLinkFormPage {
        id: 0,
        title: String::new(),
        order: 0,
        is_link: false,
        link: String::new(),
        file_id: 0,
        file_display: String::new(),
        error: String::new(),
        form_name: q.form_name(),
        refresh_table: q.refresh_table(),
        target_input: q.target_input(),
    }
}

async fn fill_form(
    db: &sea_orm::DatabaseConnection,
    page: &mut ImportantLinkFormPage,
    form: &ImportantLinkForm,
    error: String,
) {
    page.title = form.title.clone();
    page.order = form.order;
    page.is_link = form.is_link;
    page.link = form.link.clone();
    page.file_id = form.file_id;
    page.file_display = vnode_name(db, form.file_id).await;
    page.error = error;
}

pub async fn list(
    Cap(state): Cap<WebsiteState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    uri: Uri,
    Query(q): Query<ImportantLinkListQuery>,
) -> Response {
    if let Some(resp) = forbid_non_admin(&ctx) {
        return resp;
    }
    let links = query_links(&state.db, &q).await;
    let page = ImportantLinkListPage {
        links,
        filter_title: q.title.clone().unwrap_or_default(),
        path_and_query: path_and_query(&uri),
        is_admin: is_admin(&ctx),
    };
    if htmx.targets::<ImportantLinksTableKey>() {
        return page.render_table().into_response();
    }
    html_built_page_or_app_layout(&page, &htmx, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}

pub async fn detail(
    Cap(state): Cap<WebsiteState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Path(id): Path<i64>,
) -> Response {
    if let Some(resp) = forbid_non_admin(&ctx) {
        return resp;
    }
    let Some(row) = find_link(&state.db, id).await else {
        return Redirect::to("/website/important-links/").into_response();
    };
    let page = ImportantLinkDetailPage {
        id: row.id,
        title: row.title.clone(),
        order: row.order,
        is_link: row.is_link(),
        link: row.link().to_string(),
        file_id: row.file_id.unwrap_or(0),
        file_name: vnode_name_opt(&state.db, row.file_id).await,
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
    html_built_page_with_slots(&empty_form(&q), &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}

pub async fn create_post(
    Cap(state): Cap<WebsiteState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Query(q): Query<ModalFormQuery>,
    HtmlFormBody(form): HtmlFormBody<ImportantLinkForm>,
) -> Response {
    if let Some(resp) = forbid_non_admin(&ctx) {
        return resp;
    }
    let now = Utc::now();
    let model = important_link::ActiveModel {
        id: Default::default(),
        created_at: Set(Some(now)),
        updated_at: Set(Some(now)),
        deleted_at: Set(None),
        title: Set(form.title.clone()),
        order: Set(form.order),
        is_link: Set(Some(form.is_link)),
        link: Set(Some(form.link.clone())),
        file_id: Set(file_opt(form.file_id)),
    };
    match model.insert(&state.db).await {
        Ok(saved) => respond_create_modal_done_fk::<ImportantLinkCreateModalKey>(
            &htmx,
            &q.refresh_table(),
            &WebsiteImportantLinksDetailRouteTag::new(saved.id).url(),
            saved.id,
            &saved.title,
            &q.target_input(),
        ),
        Err(e) => {
            let mut page = empty_form(&q);
            fill_form(&state.db, &mut page, &form, e.to_string()).await;
            html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
        }
    }
}

pub async fn edit_get(
    Cap(state): Cap<WebsiteState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    Path(id): Path<i64>,
    Query(q): Query<ModalFormQuery>,
) -> Response {
    if let Some(resp) = forbid_non_admin(&ctx) {
        return resp;
    }
    let Some(row) = find_link(&state.db, id).await else {
        return Redirect::to("/website/important-links/").into_response();
    };
    let page = ImportantLinkFormPage {
        id: row.id,
        title: row.title.clone(),
        order: row.order,
        is_link: row.is_link(),
        link: row.link().to_string(),
        file_id: row.file_id.unwrap_or(0),
        file_display: vnode_name_opt(&state.db, row.file_id).await,
        error: String::new(),
        form_name: q.form_name(),
        refresh_table: String::new(),
        target_input: String::new(),
    };
    html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}

pub async fn edit_post(
    Cap(state): Cap<WebsiteState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Path(id): Path<i64>,
    Query(q): Query<ModalFormQuery>,
    HtmlFormBody(form): HtmlFormBody<ImportantLinkForm>,
) -> Response {
    if let Some(resp) = forbid_non_admin(&ctx) {
        return resp;
    }
    let Some(existing) = find_link(&state.db, id).await else {
        return Redirect::to("/website/important-links/").into_response();
    };
    let now = Utc::now();
    let model = important_link::ActiveModel {
        id: Set(existing.id),
        created_at: Set(existing.created_at),
        updated_at: Set(Some(now)),
        deleted_at: Set(existing.deleted_at),
        title: Set(form.title.clone()),
        order: Set(form.order),
        is_link: Set(Some(form.is_link)),
        link: Set(Some(form.link.clone())),
        file_id: Set(file_opt(form.file_id)),
    };
    match model.update(&state.db).await {
        Ok(_) => respond_edit_modal_done::<ImportantLinkEditModalKey>(
            &htmx,
            &WebsiteImportantLinksDetailRouteTag::new(id).url(),
        ),
        Err(e) => {
            let mut page = empty_form(&q);
            page.id = id;
            fill_form(&state.db, &mut page, &form, e.to_string()).await;
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
        modal_uid: ImportantLinkDeleteModalKey::ID.to_string(),
        message: format!("Delete important link #{id}?"),
        form_name: "nirmancampus_website.ImportantLinksDeleteForm".into(),
        id,
        post_url: crate::routes::WebsiteImportantLinksDeletePostRouteTag::new(id).url(),
        error: String::new(),
    };
    html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}

pub async fn delete_post(
    Cap(state): Cap<WebsiteState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    Path(id): Path<i64>,
) -> Response {
    if let Some(resp) = forbid_non_admin(&ctx) {
        return resp;
    }
    let Some(existing) = find_link(&state.db, id).await else {
        return Redirect::to("/website/important-links/").into_response();
    };
    let now = Utc::now();
    let model = important_link::ActiveModel {
        id: Set(existing.id),
        deleted_at: Set(Some(now)),
        ..Default::default()
    };
    match model.update(&state.db).await {
        Ok(_) => Redirect::to("/website/important-links/").into_response(),
        Err(e) => {
            let page = ConfirmDeletePage {
                modal_uid: ImportantLinkDeleteModalKey::ID.to_string(),
                message: format!("Delete important link #{id}?"),
                form_name: "nirmancampus_website.ImportantLinksDeleteForm".into(),
                id,
                post_url: crate::routes::WebsiteImportantLinksDeletePostRouteTag::new(id).url(),
                error: e.to_string(),
            };
            html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
        }
    }
}
