use axum::{
    extract::{Path, Query},
    http::Uri,
    response::{IntoResponse, Redirect, Response},
};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, ConnectionTrait, DatabaseBackend,
    EntityTrait, FromQueryResult, PaginatorTrait, QueryFilter, QueryOrder, Statement,
};
use serde::Deserialize;

use crate::{
    entities::{
        student_application::{self, Entity as ApplicationEntity},
        student_application_document::{self, Entity as ApplicationDocumentEntity},
    },
    forms::ApplicationForm,
    keys::{
        ApplicationCreateModalKey, ApplicationDeleteModalKey, ApplicationEditModalKey,
        ApplicationSelectModalKey, ApplicationSelectTableKey, ApplicationTableKey,
    },
    routes::StudentApplicationsDetailRouteTag,
    state::StudentApplicationsState,
    templates::{
        ApplicationDetailPage, ApplicationFormPage, ApplicationListPage, ApplicationRow,
        ApplicationSelectPage, ConfirmDeletePage,
    },
};
use lariv_rs::{
    components::{ManyToManyItem, ObjectList, SharedChromeFolder, SlotCtx, SwapKey},
    datetime::{format_date, parse_date},
    html_form::HtmlFormBody,
    http::Cap,
    picker::respond_picker_select,
    plugins::users::middleware::RequireAuth,
    web::{
        html_built_page_or_app_layout, html_built_page_with_slots, respond_create_modal_done_fk,
        respond_edit_modal_done, Htmx, ModalFormQuery,
    },
};
use nirmancampus_common::{
    is_admin, is_unassigned, optional_string, parse_optional_i64, path_and_query, scope_created_by,
    vnode_items, vnode_name, vnode_name_opt,
};

const PAGE_SIZE: u32 = 20;

#[derive(Debug, Deserialize, Default)]
pub struct ApplicationListQuery {
    #[serde(default, rename = "Email", alias = "email")]
    pub email: Option<String>,
    #[serde(default, rename = "StudentName", alias = "student_name")]
    pub student_name: Option<String>,
    #[serde(default, rename = "Mobile", alias = "mobile")]
    pub mobile: Option<String>,
    #[serde(default)]
    pub page: Option<u32>,
    #[serde(default)]
    pub sort: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct ApplicationSelectQuery {
    #[serde(flatten)]
    pub filter: ApplicationListQuery,
    #[serde(default)]
    pub target_input: Option<String>,
}

fn can_view(ctx: &lariv_rs::plugins::users::state::AuthContext) -> bool {
    is_admin(ctx) || is_unassigned(ctx)
}

fn forbid_view(ctx: &lariv_rs::plugins::users::state::AuthContext) -> Option<Response> {
    if can_view(ctx) {
        None
    } else {
        Some(Redirect::to("/").into_response())
    }
}

fn forbid_admin(ctx: &lariv_rs::plugins::users::state::AuthContext) -> Option<Response> {
    if is_admin(ctx) {
        None
    } else {
        Some(Redirect::to("/student-applications/").into_response())
    }
}

#[derive(FromQueryResult)]
struct ProgramNameRow {
    name: Option<String>,
    university: Option<String>,
}

async fn program_label(db: &sea_orm::DatabaseConnection, id: i64) -> String {
    if id <= 0 {
        return String::new();
    }
    let backend = db.get_database_backend();
    let sql = match backend {
        DatabaseBackend::Postgres => {
            "SELECT name, university FROM programs WHERE id = $1 AND deleted_at IS NULL"
        }
        _ => "SELECT name, university FROM programs WHERE id = ? AND deleted_at IS NULL",
    };
    let row = ProgramNameRow::find_by_statement(Statement::from_sql_and_values(
        backend,
        sql,
        [id.into()],
    ))
    .one(db)
    .await
    .ok()
    .flatten();
    match row {
        Some(r) => {
            let name = r.name.unwrap_or_default();
            let uni = r.university.unwrap_or_default();
            if uni.is_empty() {
                name
            } else if name.is_empty() {
                uni
            } else {
                format!("{name} ({uni})")
            }
        }
        None => format!("Program #{id}"),
    }
}

async fn find_application(
    db: &sea_orm::DatabaseConnection,
    id: i64,
    ctx: &lariv_rs::plugins::users::state::AuthContext,
) -> Option<student_application::Model> {
    let query = ApplicationEntity::find_by_id(id).filter(student_application::Column::DeletedAt.is_null());
    let query = scope_created_by(query, ctx, student_application::Column::CreatedById);
    lariv_rs::web::opt_or_log(query.one(db).await, "db find one")
}

async fn load_doc_ids(db: &sea_orm::DatabaseConnection, application_id: i64) -> Vec<i64> {
    ApplicationDocumentEntity::find()
        .filter(student_application_document::Column::StudentApplicationId.eq(application_id))
        .all(db)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|d| d.v_node_id)
        .collect()
}

async fn replace_docs(
    db: &sea_orm::DatabaseConnection,
    application_id: i64,
    ids: &[i64],
) -> Result<(), sea_orm::DbErr> {
    ApplicationDocumentEntity::delete_many()
        .filter(student_application_document::Column::StudentApplicationId.eq(application_id))
        .exec(db)
        .await?;
    for &v_node_id in ids {
        if v_node_id <= 0 {
            continue;
        }
        student_application_document::ActiveModel {
            student_application_id: Set(application_id),
            v_node_id: Set(v_node_id),
        }
        .insert(db)
        .await?;
    }
    Ok(())
}

async fn doc_items(db: &sea_orm::DatabaseConnection, ids: &[i64]) -> Vec<ManyToManyItem> {
    vnode_items(db, ids).await
}

async fn query_applications(
    db: &sea_orm::DatabaseConnection,
    q: &ApplicationListQuery,
    ctx: &lariv_rs::plugins::users::state::AuthContext,
) -> ObjectList<ApplicationRow> {
    let mut query = ApplicationEntity::find().filter(student_application::Column::DeletedAt.is_null());
    query = scope_created_by(query, ctx, student_application::Column::CreatedById);
    if let Some(email) = q.email.as_ref().filter(|n| !n.is_empty()) {
        query = query.filter(student_application::Column::Email.contains(email));
    }
    if let Some(name) = q.student_name.as_ref().filter(|n| !n.is_empty()) {
        query = query.filter(student_application::Column::StudentName.contains(name));
    }
    if let Some(mobile) = q.mobile.as_ref().filter(|n| !n.is_empty()) {
        query = query.filter(student_application::Column::Mobile.contains(mobile));
    }
    query = query.order_by_desc(student_application::Column::Id);
    let page = q.page.unwrap_or(1).max(1);
    let paginator = query.paginate(db, PAGE_SIZE as u64);
    let total = paginator.num_items().await.unwrap_or(0);
    let models = paginator
        .fetch_page((page as u64).saturating_sub(1))
        .await
        .unwrap_or_default();
    let mut rows = Vec::with_capacity(models.len());
    for a in models {
        let program_display = program_label(db, a.program_id).await;
        rows.push(ApplicationRow {
            id: a.id,
            email: a.email().to_string(),
            program_display,
            student_name: a.student_name().to_string(),
            mobile: a.mobile().to_string(),
        });
    }
    ObjectList::from_page(rows, page, PAGE_SIZE, total)
}

fn empty_form(q: &ModalFormQuery) -> ApplicationFormPage {
    ApplicationFormPage {
        id: 0,
        program_id: 0,
        program_display: String::new(),
        student_name: String::new(),
        dob: String::new(),
        mother_name: String::new(),
        father_name: String::new(),
        category: String::new(),
        mobile: String::new(),
        email: String::new(),
        address: String::new(),
        photo_id: 0,
        photo_display: String::new(),
        documents: Vec::new(),
        error: String::new(),
        form_name: q.form_name(),
        refresh_table: q.refresh_table(),
        target_input: q.target_input(),
        can_delete: false,
    }
}

pub async fn list(
    Cap(state): Cap<StudentApplicationsState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    uri: Uri,
    Query(q): Query<ApplicationListQuery>,
) -> Response {
    if let Some(resp) = forbid_view(&ctx) {
        return resp;
    }
    let applications = query_applications(&state.db, &q, &ctx).await;
    let page = ApplicationListPage {
        applications,
        filter_email: q.email.clone().unwrap_or_default(),
        filter_student_name: q.student_name.clone().unwrap_or_default(),
        filter_mobile: q.mobile.clone().unwrap_or_default(),
        path_and_query: path_and_query(&uri),
        can_create: can_view(&ctx),
    };
    if htmx.targets::<ApplicationTableKey>() {
        return page.render_table().into_response();
    }
    html_built_page_or_app_layout(&page, &htmx, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}

pub async fn detail(
    Cap(state): Cap<StudentApplicationsState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Path(id): Path<i64>,
) -> Response {
    if let Some(resp) = forbid_view(&ctx) {
        return resp;
    }
    let Some(row) = find_application(&state.db, id, &ctx).await else {
        return Redirect::to("/student-applications/").into_response();
    };
    let page = ApplicationDetailPage {
        id: row.id,
        program_id: row.program_id,
        program_display: program_label(&state.db, row.program_id).await,
        student_name: row.student_name().to_string(),
        email: row.email().to_string(),
        dob: row.dob.map(format_date).unwrap_or_default(),
        mother_name: row.mother_name().to_string(),
        father_name: row.father_name().to_string(),
        category: row.category().to_string(),
        mobile: row.mobile().to_string(),
        address: row.address().to_string(),
        photo_id: row.photo_id.unwrap_or(0),
        photo_name: vnode_name_opt(&state.db, row.photo_id).await,
        documents: doc_items(&state.db, &load_doc_ids(&state.db, id).await).await,
        is_admin: is_admin(&ctx),
    };
    html_built_page_or_app_layout(&page, &htmx, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}

pub async fn create_get(
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    Query(q): Query<ModalFormQuery>,
) -> Response {
    if let Some(resp) = forbid_view(&ctx) {
        return resp;
    }
    html_built_page_with_slots(&empty_form(&q), &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}

fn photo_opt(id: i64) -> Option<i64> {
    parse_optional_i64(&id.to_string())
}

pub async fn create_post(
    Cap(state): Cap<StudentApplicationsState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Query(q): Query<ModalFormQuery>,
    HtmlFormBody(form): HtmlFormBody<ApplicationForm>,
) -> Response {
    if let Some(resp) = forbid_view(&ctx) {
        return resp;
    }
    let now = Utc::now();
    let model = student_application::ActiveModel {
        id: Default::default(),
        created_at: Set(Some(now)),
        updated_at: Set(Some(now)),
        deleted_at: Set(None),
        program_id: Set(form.program_id),
        created_by_id: Set(Some(ctx.user.id)),
        student_name: Set(form.student_name.clone()),
        email: Set(optional_string(&form.email)),
        dob: Set(parse_date(&form.dob)),
        mother_name: Set(optional_string(&form.mother_name)),
        father_name: Set(optional_string(&form.father_name)),
        category: Set(optional_string(&form.category)),
        address: Set(optional_string(&form.address)),
        mobile: Set(optional_string(&form.mobile)),
        photo_id: Set(photo_opt(form.photo_id)),
    };
    match model.insert(&state.db).await {
        Ok(saved) => {
            if let Err(e) = replace_docs(&state.db, saved.id, &form.documents).await {
                tracing::error!(error = %e, id = saved.id, "failed to save application documents");
            }
            respond_create_modal_done_fk::<ApplicationCreateModalKey>(
                &htmx,
                &q.refresh_table(),
                &StudentApplicationsDetailRouteTag::new(saved.id).url(),
                saved.id,
                saved.student_name(),
                &q.target_input(),
            )
        }
        Err(e) => {
            let mut page = empty_form(&q);
            page.program_id = form.program_id;
            page.program_display = program_label(&state.db, form.program_id).await;
            page.student_name = form.student_name;
            page.dob = form.dob;
            page.mother_name = form.mother_name;
            page.father_name = form.father_name;
            page.category = form.category;
            page.mobile = form.mobile;
            page.email = form.email;
            page.address = form.address;
            page.photo_id = form.photo_id;
            page.photo_display = vnode_name(&state.db, form.photo_id).await;
            page.documents = doc_items(&state.db, &form.documents).await;
            page.error = e.to_string();
            html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
        }
    }
}

pub async fn edit_get(
    Cap(state): Cap<StudentApplicationsState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    Path(id): Path<i64>,
    Query(q): Query<ModalFormQuery>,
) -> Response {
    if let Some(resp) = forbid_admin(&ctx) {
        return resp;
    }
    let Some(row) = find_application(&state.db, id, &ctx).await else {
        return Redirect::to("/student-applications/").into_response();
    };
    let docs = load_doc_ids(&state.db, id).await;
    let page = ApplicationFormPage {
        id: row.id,
        program_id: row.program_id,
        program_display: program_label(&state.db, row.program_id).await,
        student_name: row.student_name().to_string(),
        dob: row.dob.map(format_date).unwrap_or_default(),
        mother_name: row.mother_name().to_string(),
        father_name: row.father_name().to_string(),
        category: row.category().to_string(),
        mobile: row.mobile().to_string(),
        email: row.email().to_string(),
        address: row.address().to_string(),
        photo_id: row.photo_id.unwrap_or(0),
        photo_display: vnode_name_opt(&state.db, row.photo_id).await,
        documents: doc_items(&state.db, &docs).await,
        error: String::new(),
        form_name: q.form_name(),
        refresh_table: String::new(),
        target_input: String::new(),
        can_delete: true,
    };
    html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}

pub async fn edit_post(
    Cap(state): Cap<StudentApplicationsState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Path(id): Path<i64>,
    Query(q): Query<ModalFormQuery>,
    HtmlFormBody(form): HtmlFormBody<ApplicationForm>,
) -> Response {
    if let Some(resp) = forbid_admin(&ctx) {
        return resp;
    }
    let Some(existing) = find_application(&state.db, id, &ctx).await else {
        return Redirect::to("/student-applications/").into_response();
    };
    let now = Utc::now();
    let model = student_application::ActiveModel {
        id: Set(existing.id),
        created_at: Set(existing.created_at),
        updated_at: Set(Some(now)),
        deleted_at: Set(existing.deleted_at),
        program_id: Set(form.program_id),
        created_by_id: Set(existing.created_by_id),
        student_name: Set(form.student_name.clone()),
        email: Set(optional_string(&form.email)),
        dob: Set(parse_date(&form.dob)),
        mother_name: Set(optional_string(&form.mother_name)),
        father_name: Set(optional_string(&form.father_name)),
        category: Set(optional_string(&form.category)),
        address: Set(optional_string(&form.address)),
        mobile: Set(optional_string(&form.mobile)),
        photo_id: Set(photo_opt(form.photo_id)),
    };
    match model.update(&state.db).await {
        Ok(_) => {
            if let Err(e) = replace_docs(&state.db, id, &form.documents).await {
                tracing::error!(error = %e, id, "failed to save application documents");
            }
            respond_edit_modal_done::<ApplicationEditModalKey>(
                &htmx,
                &StudentApplicationsDetailRouteTag::new(id).url(),
            )
        }
        Err(e) => {
            let page = ApplicationFormPage {
                id,
                program_id: form.program_id,
                program_display: program_label(&state.db, form.program_id).await,
                student_name: form.student_name,
                dob: form.dob,
                mother_name: form.mother_name,
                father_name: form.father_name,
                category: form.category,
                mobile: form.mobile,
                email: form.email,
                address: form.address,
                photo_id: form.photo_id,
                photo_display: vnode_name(&state.db, form.photo_id).await,
                documents: doc_items(&state.db, &form.documents).await,
                error: e.to_string(),
                form_name: q.form_name(),
                refresh_table: String::new(),
                target_input: String::new(),
                can_delete: true,
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
    if let Some(resp) = forbid_admin(&ctx) {
        return resp;
    }
    let page = ConfirmDeletePage {
        modal_uid: ApplicationDeleteModalKey::ID.to_string(),
        message: format!("Delete application #{id}?"),
        form_name: "studentapplications.ApplicationDeleteForm".into(),
        id,
        error: String::new(),
    };
    html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}

pub async fn delete_post(
    Cap(state): Cap<StudentApplicationsState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    Path(id): Path<i64>,
) -> Response {
    if let Some(resp) = forbid_admin(&ctx) {
        return resp;
    }
    let Some(existing) = find_application(&state.db, id, &ctx).await else {
        return Redirect::to("/student-applications/").into_response();
    };
    let now = Utc::now();
    let model = student_application::ActiveModel {
        id: Set(existing.id),
        deleted_at: Set(Some(now)),
        ..Default::default()
    };
    match model.update(&state.db).await {
        Ok(_) => Redirect::to("/student-applications/").into_response(),
        Err(e) => {
            let page = ConfirmDeletePage {
                modal_uid: ApplicationDeleteModalKey::ID.to_string(),
                message: format!("Delete application #{id}?"),
                form_name: "studentapplications.ApplicationDeleteForm".into(),
                id,
                error: e.to_string(),
            };
            html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
        }
    }
}

pub async fn select(
    Cap(state): Cap<StudentApplicationsState>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    uri: Uri,
    Query(q): Query<ApplicationSelectQuery>,
) -> Response {
    if let Some(resp) = forbid_view(&ctx) {
        return resp;
    }
    let applications = query_applications(&state.db, &q.filter, &ctx).await;
    let page = ApplicationSelectPage {
        applications,
        filter_student_name: q.filter.student_name.clone().unwrap_or_default(),
        path_and_query: path_and_query(&uri),
        target_input: q.target_input.unwrap_or_default(),
    };
    respond_picker_select::<ApplicationSelectTableKey, ApplicationSelectModalKey, _>(&htmx, &page)
        .into_response()
}
