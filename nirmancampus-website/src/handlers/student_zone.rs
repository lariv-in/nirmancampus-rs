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
    entities::{
        student_zone_item::{self, Entity as ItemEntity},
        student_zone_section::{self, Entity as SectionEntity},
    },
    forms::{StudentZoneItemForm, StudentZoneSectionForm},
    handlers::{file_opt, forbid_non_admin},
    keys::{
        StudentZoneItemCreateModalKey, StudentZoneItemDeleteModalKey, StudentZoneItemEditModalKey,
        StudentZoneItemTableKey, StudentZoneSectionCreateModalKey,
        StudentZoneSectionDeleteModalKey, StudentZoneSectionEditModalKey,
        StudentZoneSectionSelectModalKey, StudentZoneSectionSelectTableKey,
        StudentZoneSectionTableKey,
    },
    routes::{
        WebsiteStudentZoneItemsDeletePostRouteTag, WebsiteStudentZoneItemsDetailRouteTag,
        WebsiteStudentZoneSectionsDeletePostRouteTag, WebsiteStudentZoneSectionsDetailRouteTag,
    },
    state::WebsiteState,
    templates::{
        ConfirmDeletePage, StudentZoneItemDetailPage, StudentZoneItemFormPage,
        StudentZoneItemListPage, StudentZoneItemRow, StudentZoneSectionDetailPage,
        StudentZoneSectionFormPage, StudentZoneSectionListPage, StudentZoneSectionRow,
        StudentZoneSectionSelectPage,
    },
};
use lariv_rs::{
    components::{ObjectList, SharedChromeFolder, SlotCtx, SwapKey},
    html_form::HtmlFormBody,
    http::Cap,
    picker::respond_picker_select,
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
pub struct TitleListQuery {
    #[serde(default, rename = "Title", alias = "title")]
    pub title: Option<String>,
    #[serde(default)]
    pub page: Option<u32>,
}

#[derive(Debug, Deserialize, Default)]
pub struct SectionSelectQuery {
    #[serde(flatten)]
    pub filter: TitleListQuery,
    #[serde(default)]
    pub target_input: Option<String>,
}

async fn find_section(
    db: &sea_orm::DatabaseConnection,
    id: i64,
) -> Option<student_zone_section::Model> {
    lariv_rs::web::opt_or_log(
        SectionEntity::find_by_id(id)
            .filter(student_zone_section::Column::DeletedAt.is_null())
            .one(db)
            .await,
        "db find student zone section",
    )
}

async fn find_item(db: &sea_orm::DatabaseConnection, id: i64) -> Option<student_zone_item::Model> {
    lariv_rs::web::opt_or_log(
        ItemEntity::find_by_id(id)
            .filter(student_zone_item::Column::DeletedAt.is_null())
            .one(db)
            .await,
        "db find student zone item",
    )
}

async fn query_sections(
    db: &sea_orm::DatabaseConnection,
    q: &TitleListQuery,
) -> ObjectList<StudentZoneSectionRow> {
    let mut query = SectionEntity::find().filter(student_zone_section::Column::DeletedAt.is_null());
    if let Some(title) = q.title.as_ref().filter(|n| !n.is_empty()) {
        query = query.filter(student_zone_section::Column::Title.contains(title));
    }
    query = query
        .order_by_asc(student_zone_section::Column::Order)
        .order_by_asc(student_zone_section::Column::Id);
    let page = q.page.unwrap_or(1).max(1);
    let paginator = query.paginate(db, PAGE_SIZE as u64);
    let total = paginator.num_items().await.unwrap_or(0);
    let models = paginator
        .fetch_page((page as u64).saturating_sub(1))
        .await
        .unwrap_or_default();
    let rows = models
        .into_iter()
        .map(|s| StudentZoneSectionRow {
            id: s.id,
            title: s.title.clone(),
            order: s.order,
        })
        .collect();
    ObjectList::from_page(rows, page, PAGE_SIZE, total)
}

async fn section_display(db: &sea_orm::DatabaseConnection, id: i64) -> String {
    find_section(db, id)
        .await
        .map(|s| s.title)
        .unwrap_or_default()
}

async fn query_items(
    db: &sea_orm::DatabaseConnection,
    q: &TitleListQuery,
) -> ObjectList<StudentZoneItemRow> {
    let mut query = ItemEntity::find().filter(student_zone_item::Column::DeletedAt.is_null());
    if let Some(title) = q.title.as_ref().filter(|n| !n.is_empty()) {
        query = query.filter(student_zone_item::Column::Title.contains(title));
    }
    query = query.order_by_desc(student_zone_item::Column::Id);
    let page = q.page.unwrap_or(1).max(1);
    let paginator = query.paginate(db, PAGE_SIZE as u64);
    let total = paginator.num_items().await.unwrap_or(0);
    let models = paginator
        .fetch_page((page as u64).saturating_sub(1))
        .await
        .unwrap_or_default();
    let mut rows = Vec::with_capacity(models.len());
    for item in models {
        let section = match item.student_zone_section_id {
            Some(sid) => section_display(db, sid).await,
            None => String::new(),
        };
        rows.push(StudentZoneItemRow {
            id: item.id,
            title: item.title.clone(),
            is_link: yes_no(item.is_link()).to_string(),
            section,
        });
    }
    ObjectList::from_page(rows, page, PAGE_SIZE, total)
}

fn empty_section_form(q: &ModalFormQuery) -> StudentZoneSectionFormPage {
    StudentZoneSectionFormPage {
        id: 0,
        title: String::new(),
        order: 0,
        error: String::new(),
        form_name: q.form_name(),
        refresh_table: q.refresh_table(),
        target_input: q.target_input(),
    }
}

fn empty_item_form(q: &ModalFormQuery) -> StudentZoneItemFormPage {
    StudentZoneItemFormPage {
        id: 0,
        title: String::new(),
        is_link: false,
        link: String::new(),
        file_id: 0,
        file_display: String::new(),
        student_zone_section_id: 0,
        section_display: String::new(),
        error: String::new(),
        form_name: q.form_name(),
        refresh_table: q.refresh_table(),
        target_input: q.target_input(),
    }
}

pub async fn section_list(
    Cap(state): Cap<WebsiteState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    uri: Uri,
    Query(q): Query<TitleListQuery>,
) -> Response {
    if let Some(resp) = forbid_non_admin(&ctx) {
        return resp;
    }
    let sections = query_sections(&state.db, &q).await;
    let page = StudentZoneSectionListPage {
        sections,
        filter_title: q.title.clone().unwrap_or_default(),
        path_and_query: path_and_query(&uri),
        is_admin: is_admin(&ctx),
    };
    if htmx.targets::<StudentZoneSectionTableKey>() {
        return page.render_table().into_response();
    }
    html_built_page_or_app_layout(&page, &htmx, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}

pub async fn section_detail(
    Cap(state): Cap<WebsiteState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Path(id): Path<i64>,
) -> Response {
    if let Some(resp) = forbid_non_admin(&ctx) {
        return resp;
    }
    let Some(row) = find_section(&state.db, id).await else {
        return Redirect::to("/website/student-zone/").into_response();
    };
    let page = StudentZoneSectionDetailPage {
        id: row.id,
        title: row.title.clone(),
        order: row.order,
        is_admin: is_admin(&ctx),
    };
    html_built_page_or_app_layout(&page, &htmx, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}

pub async fn section_create_get(
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    Query(q): Query<ModalFormQuery>,
) -> Response {
    if let Some(resp) = forbid_non_admin(&ctx) {
        return resp;
    }
    html_built_page_with_slots(&empty_section_form(&q), &chrome, &SlotCtx::from_auth(&ctx))
        .into_response()
}

pub async fn section_create_post(
    Cap(state): Cap<WebsiteState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Query(q): Query<ModalFormQuery>,
    HtmlFormBody(form): HtmlFormBody<StudentZoneSectionForm>,
) -> Response {
    if let Some(resp) = forbid_non_admin(&ctx) {
        return resp;
    }
    let now = Utc::now();
    let model = student_zone_section::ActiveModel {
        id: Default::default(),
        created_at: Set(Some(now)),
        updated_at: Set(Some(now)),
        deleted_at: Set(None),
        title: Set(form.title.clone()),
        order: Set(form.order),
    };
    match model.insert(&state.db).await {
        Ok(saved) => respond_create_modal_done_fk::<StudentZoneSectionCreateModalKey>(
            &htmx,
            &q.refresh_table(),
            &WebsiteStudentZoneSectionsDetailRouteTag::new(saved.id).url(),
            saved.id,
            &saved.title,
            &q.target_input(),
        ),
        Err(e) => {
            let mut page = empty_section_form(&q);
            page.title = form.title;
            page.order = form.order;
            page.error = e.to_string();
            html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
        }
    }
}

pub async fn section_edit_get(
    Cap(state): Cap<WebsiteState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    Path(id): Path<i64>,
    Query(q): Query<ModalFormQuery>,
) -> Response {
    if let Some(resp) = forbid_non_admin(&ctx) {
        return resp;
    }
    let Some(row) = find_section(&state.db, id).await else {
        return Redirect::to("/website/student-zone/").into_response();
    };
    let page = StudentZoneSectionFormPage {
        id: row.id,
        title: row.title.clone(),
        order: row.order,
        error: String::new(),
        form_name: q.form_name(),
        refresh_table: String::new(),
        target_input: String::new(),
    };
    html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}

pub async fn section_edit_post(
    Cap(state): Cap<WebsiteState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Path(id): Path<i64>,
    Query(q): Query<ModalFormQuery>,
    HtmlFormBody(form): HtmlFormBody<StudentZoneSectionForm>,
) -> Response {
    if let Some(resp) = forbid_non_admin(&ctx) {
        return resp;
    }
    let Some(existing) = find_section(&state.db, id).await else {
        return Redirect::to("/website/student-zone/").into_response();
    };
    let now = Utc::now();
    let model = student_zone_section::ActiveModel {
        id: Set(existing.id),
        created_at: Set(existing.created_at),
        updated_at: Set(Some(now)),
        deleted_at: Set(existing.deleted_at),
        title: Set(form.title.clone()),
        order: Set(form.order),
    };
    match model.update(&state.db).await {
        Ok(_) => respond_edit_modal_done::<StudentZoneSectionEditModalKey>(
            &htmx,
            &WebsiteStudentZoneSectionsDetailRouteTag::new(id).url(),
        ),
        Err(e) => {
            let mut page = empty_section_form(&q);
            page.id = id;
            page.title = form.title;
            page.order = form.order;
            page.error = e.to_string();
            html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
        }
    }
}

pub async fn section_delete_get(
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    Path(id): Path<i64>,
) -> Response {
    if let Some(resp) = forbid_non_admin(&ctx) {
        return resp;
    }
    let page = ConfirmDeletePage {
        modal_uid: StudentZoneSectionDeleteModalKey::ID.to_string(),
        message: format!("Delete student zone section #{id}?"),
        form_name: "nirmancampus_website.StudentZoneSectionDeleteForm".into(),
        id,
        post_url: WebsiteStudentZoneSectionsDeletePostRouteTag::new(id).url(),
        error: String::new(),
    };
    html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}

pub async fn section_delete_post(
    Cap(state): Cap<WebsiteState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    Path(id): Path<i64>,
) -> Response {
    if let Some(resp) = forbid_non_admin(&ctx) {
        return resp;
    }
    let Some(existing) = find_section(&state.db, id).await else {
        return Redirect::to("/website/student-zone/").into_response();
    };
    let now = Utc::now();
    let model = student_zone_section::ActiveModel {
        id: Set(existing.id),
        deleted_at: Set(Some(now)),
        ..Default::default()
    };
    match model.update(&state.db).await {
        Ok(_) => Redirect::to("/website/student-zone/").into_response(),
        Err(e) => {
            let page = ConfirmDeletePage {
                modal_uid: StudentZoneSectionDeleteModalKey::ID.to_string(),
                message: format!("Delete student zone section #{id}?"),
                form_name: "nirmancampus_website.StudentZoneSectionDeleteForm".into(),
                id,
                post_url: WebsiteStudentZoneSectionsDeletePostRouteTag::new(id).url(),
                error: e.to_string(),
            };
            html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
        }
    }
}

pub async fn section_select(
    Cap(state): Cap<WebsiteState>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    uri: Uri,
    Query(q): Query<SectionSelectQuery>,
) -> Response {
    if let Some(resp) = forbid_non_admin(&ctx) {
        return resp;
    }
    let sections = query_sections(&state.db, &q.filter).await;
    let page = StudentZoneSectionSelectPage {
        sections,
        filter_title: q.filter.title.clone().unwrap_or_default(),
        path_and_query: path_and_query(&uri),
        target_input: q.target_input.unwrap_or_default(),
    };
    respond_picker_select::<StudentZoneSectionSelectTableKey, StudentZoneSectionSelectModalKey, _>(
        &htmx, &page,
    )
    .into_response()
}

pub async fn item_list(
    Cap(state): Cap<WebsiteState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    uri: Uri,
    Query(q): Query<TitleListQuery>,
) -> Response {
    if let Some(resp) = forbid_non_admin(&ctx) {
        return resp;
    }
    let items = query_items(&state.db, &q).await;
    let page = StudentZoneItemListPage {
        items,
        filter_title: q.title.clone().unwrap_or_default(),
        path_and_query: path_and_query(&uri),
        is_admin: is_admin(&ctx),
    };
    if htmx.targets::<StudentZoneItemTableKey>() {
        return page.render_table().into_response();
    }
    html_built_page_or_app_layout(&page, &htmx, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}

pub async fn item_detail(
    Cap(state): Cap<WebsiteState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Path(id): Path<i64>,
) -> Response {
    if let Some(resp) = forbid_non_admin(&ctx) {
        return resp;
    }
    let Some(row) = find_item(&state.db, id).await else {
        return Redirect::to("/website/student-zone/items/").into_response();
    };
    let section = match row.student_zone_section_id {
        Some(sid) => section_display(&state.db, sid).await,
        None => String::new(),
    };
    let page = StudentZoneItemDetailPage {
        id: row.id,
        title: row.title.clone(),
        is_link: row.is_link(),
        link: row.link().to_string(),
        file_id: row.file_id.unwrap_or(0),
        file_name: vnode_name_opt(&state.db, row.file_id).await,
        section,
        section_id: row.student_zone_section_id.unwrap_or(0),
        is_admin: is_admin(&ctx),
    };
    html_built_page_or_app_layout(&page, &htmx, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}

pub async fn item_create_get(
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    Query(q): Query<ModalFormQuery>,
) -> Response {
    if let Some(resp) = forbid_non_admin(&ctx) {
        return resp;
    }
    html_built_page_with_slots(&empty_item_form(&q), &chrome, &SlotCtx::from_auth(&ctx))
        .into_response()
}

async fn fill_item(
    db: &sea_orm::DatabaseConnection,
    page: &mut StudentZoneItemFormPage,
    form: &StudentZoneItemForm,
    error: String,
) {
    page.title = form.title.clone();
    page.is_link = form.is_link;
    page.link = form.link.clone();
    page.file_id = form.file_id;
    page.file_display = vnode_name(db, form.file_id).await;
    page.student_zone_section_id = form.student_zone_section_id;
    page.section_display = if form.student_zone_section_id > 0 {
        section_display(db, form.student_zone_section_id).await
    } else {
        String::new()
    };
    page.error = error;
}

pub async fn item_create_post(
    Cap(state): Cap<WebsiteState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Query(q): Query<ModalFormQuery>,
    HtmlFormBody(form): HtmlFormBody<StudentZoneItemForm>,
) -> Response {
    if let Some(resp) = forbid_non_admin(&ctx) {
        return resp;
    }
    let now = Utc::now();
    let model = student_zone_item::ActiveModel {
        id: Default::default(),
        created_at: Set(Some(now)),
        updated_at: Set(Some(now)),
        deleted_at: Set(None),
        title: Set(form.title.clone()),
        is_link: Set(Some(form.is_link)),
        link: Set(Some(form.link.clone())),
        file_id: Set(file_opt(form.file_id)),
        student_zone_section_id: Set(file_opt(form.student_zone_section_id)),
    };
    match model.insert(&state.db).await {
        Ok(saved) => respond_create_modal_done_fk::<StudentZoneItemCreateModalKey>(
            &htmx,
            &q.refresh_table(),
            &WebsiteStudentZoneItemsDetailRouteTag::new(saved.id).url(),
            saved.id,
            &saved.title,
            &q.target_input(),
        ),
        Err(e) => {
            let mut page = empty_item_form(&q);
            fill_item(&state.db, &mut page, &form, e.to_string()).await;
            html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
        }
    }
}

pub async fn item_edit_get(
    Cap(state): Cap<WebsiteState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    Path(id): Path<i64>,
    Query(q): Query<ModalFormQuery>,
) -> Response {
    if let Some(resp) = forbid_non_admin(&ctx) {
        return resp;
    }
    let Some(row) = find_item(&state.db, id).await else {
        return Redirect::to("/website/student-zone/items/").into_response();
    };
    let section_id = row.student_zone_section_id.unwrap_or(0);
    let page = StudentZoneItemFormPage {
        id: row.id,
        title: row.title.clone(),
        is_link: row.is_link(),
        link: row.link().to_string(),
        file_id: row.file_id.unwrap_or(0),
        file_display: vnode_name_opt(&state.db, row.file_id).await,
        student_zone_section_id: section_id,
        section_display: if section_id > 0 {
            section_display(&state.db, section_id).await
        } else {
            String::new()
        },
        error: String::new(),
        form_name: q.form_name(),
        refresh_table: String::new(),
        target_input: String::new(),
    };
    html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}

pub async fn item_edit_post(
    Cap(state): Cap<WebsiteState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Path(id): Path<i64>,
    Query(q): Query<ModalFormQuery>,
    HtmlFormBody(form): HtmlFormBody<StudentZoneItemForm>,
) -> Response {
    if let Some(resp) = forbid_non_admin(&ctx) {
        return resp;
    }
    let Some(existing) = find_item(&state.db, id).await else {
        return Redirect::to("/website/student-zone/items/").into_response();
    };
    let now = Utc::now();
    let model = student_zone_item::ActiveModel {
        id: Set(existing.id),
        created_at: Set(existing.created_at),
        updated_at: Set(Some(now)),
        deleted_at: Set(existing.deleted_at),
        title: Set(form.title.clone()),
        is_link: Set(Some(form.is_link)),
        link: Set(Some(form.link.clone())),
        file_id: Set(file_opt(form.file_id)),
        student_zone_section_id: Set(file_opt(form.student_zone_section_id)),
    };
    match model.update(&state.db).await {
        Ok(_) => respond_edit_modal_done::<StudentZoneItemEditModalKey>(
            &htmx,
            &WebsiteStudentZoneItemsDetailRouteTag::new(id).url(),
        ),
        Err(e) => {
            let mut page = empty_item_form(&q);
            page.id = id;
            fill_item(&state.db, &mut page, &form, e.to_string()).await;
            html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
        }
    }
}

pub async fn item_delete_get(
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    Path(id): Path<i64>,
) -> Response {
    if let Some(resp) = forbid_non_admin(&ctx) {
        return resp;
    }
    let page = ConfirmDeletePage {
        modal_uid: StudentZoneItemDeleteModalKey::ID.to_string(),
        message: format!("Delete student zone item #{id}?"),
        form_name: "nirmancampus_website.StudentZoneItemDeleteForm".into(),
        id,
        post_url: WebsiteStudentZoneItemsDeletePostRouteTag::new(id).url(),
        error: String::new(),
    };
    html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}

pub async fn item_delete_post(
    Cap(state): Cap<WebsiteState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    Path(id): Path<i64>,
) -> Response {
    if let Some(resp) = forbid_non_admin(&ctx) {
        return resp;
    }
    let Some(existing) = find_item(&state.db, id).await else {
        return Redirect::to("/website/student-zone/items/").into_response();
    };
    let now = Utc::now();
    let model = student_zone_item::ActiveModel {
        id: Set(existing.id),
        deleted_at: Set(Some(now)),
        ..Default::default()
    };
    match model.update(&state.db).await {
        Ok(_) => Redirect::to("/website/student-zone/items/").into_response(),
        Err(e) => {
            let page = ConfirmDeletePage {
                modal_uid: StudentZoneItemDeleteModalKey::ID.to_string(),
                message: format!("Delete student zone item #{id}?"),
                form_name: "nirmancampus_website.StudentZoneItemDeleteForm".into(),
                id,
                post_url: WebsiteStudentZoneItemsDeletePostRouteTag::new(id).url(),
                error: e.to_string(),
            };
            html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
        }
    }
}
