use axum::{
    extract::{Path, Query},
    http::{HeaderMap, Uri},
    response::{IntoResponse, Redirect, Response},
};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, ConnectionTrait, DatabaseBackend,
    EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, Statement, sea_query::Expr,
};
use serde::Deserialize;

use crate::{
    entities::{
        exam_registration::{self, Entity as ExamEntity},
        exam_registration_asset::{self, Entity as ExamAssetEntity},
    },
    forms::ExamForm,
    keys::{
        ExamCreateModalKey, ExamDeleteModalKey, ExamEditModalKey, ExamSelectModalKey,
        ExamSelectTableKey, ExamTableKey,
    },
    routes::ExamRegistrationsDetailRouteTag,
    state::ExamRegistrationsState,
    templates::{
        AssetLink, ConfirmDeletePage, ExamDetailPage, ExamFormPage, ExamListPage, ExamRow,
        ExamSelectPage,
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
use nirmancampus_academicrecords::entities::academic_record::{
    self, Entity as AcademicRecordEntity,
};
use nirmancampus_common::{
    can_view_campus_records, exam_status_choice_pairs, format_inr, is_admin, is_student,
    path_and_query,
    env::{
        parse_environment_from_headers, selected_session, EXAM_REGISTRATIONS_SESSION_KEY,
    },
    ui::SessionOption,
    vnode_items,
};
use nirmancampus_courses::entities::course::{self, Entity as CourseEntity};
use nirmancampus_sessions::entities::admission_session::{self, Entity as SessionEntity};

const PAGE_SIZE: u32 = 20;
pub const STATUS_NOT_REGISTERED: &str = "not_registered";

#[derive(Debug, Deserialize, Default)]
pub struct ExamListQuery {
    #[serde(default, rename = "ExamTitle", alias = "exam_title")]
    pub exam_title: Option<String>,
    #[serde(default, rename = "RegistrationStatus", alias = "registration_status")]
    pub registration_status: Option<String>,
    #[serde(default, rename = "AcademicRecordID", alias = "academic_record_id")]
    pub academic_record_id: Option<i64>,
    #[serde(default)]
    pub page: Option<u32>,
    #[serde(default)]
    pub sort: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct ExamSelectQuery {
    #[serde(flatten)]
    pub filter: ExamListQuery,
    #[serde(default)]
    pub target_input: Option<String>,
}

pub fn forbid_non_admin(ctx: &lariv_rs::plugins::users::state::AuthContext) -> Option<Response> {
    if is_admin(ctx) {
        None
    } else {
        Some(Redirect::to("/").into_response())
    }
}

pub fn forbid_if_no_access(ctx: &lariv_rs::plugins::users::state::AuthContext) -> Option<Response> {
    if can_view_campus_records(ctx) {
        None
    } else {
        Some(Redirect::to("/").into_response())
    }
}

fn scope_by_role(
    query: sea_orm::Select<ExamEntity>,
    auth: &lariv_rs::plugins::users::state::AuthContext,
) -> sea_orm::Select<ExamEntity> {
    if is_admin(auth) {
        return query;
    }
    if is_student(auth) {
        let email = auth.user.email.trim().to_string();
        if email.is_empty() {
            return query.filter(Expr::cust("1 = 0"));
        }
        return query.filter(Expr::cust_with_values(
            "academic_record_id IN (SELECT id FROM academic_records WHERE deleted_at IS NULL AND student_id IN (SELECT id FROM students WHERE email = ? AND deleted_at IS NULL))",
            [email],
        ));
    }
    query.filter(Expr::cust("1 = 0"))
}

fn apply_session_filter(
    query: sea_orm::Select<ExamEntity>,
    session_id: Option<i64>,
) -> sea_orm::Select<ExamEntity> {
    match session_id {
        Some(id) if id > 0 => query.filter(Expr::cust_with_values(
            "academic_record_id IN (SELECT id FROM academic_records WHERE deleted_at IS NULL AND session_id = ?)",
            [id],
        )),
        _ => query,
    }
}

async fn default_admission_session_id(db: &sea_orm::DatabaseConnection) -> Option<i64> {
    if let Some(s) = lariv_rs::web::opt_or_log(
        SessionEntity::find()
            .filter(admission_session::Column::DeletedAt.is_null())
            .filter(admission_session::Column::IsActive.eq(true))
            .order_by_desc(admission_session::Column::Start)
            .one(db)
            .await,
        "db default active session",
    ) {
        return Some(s.id);
    }
    lariv_rs::web::opt_or_log(
        SessionEntity::find()
            .filter(admission_session::Column::DeletedAt.is_null())
            .order_by_desc(admission_session::Column::Start)
            .one(db)
            .await,
        "db default latest session",
    )
    .map(|s| s.id)
}

async fn list_session_options(db: &sea_orm::DatabaseConnection) -> Vec<SessionOption> {
    SessionEntity::find()
        .filter(admission_session::Column::DeletedAt.is_null())
        .order_by_desc(admission_session::Column::Start)
        .all(db)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|s| SessionOption {
            id: s.id,
            name: s.name().to_string(),
        })
        .collect()
}

async fn resolve_session_filter(
    db: &sea_orm::DatabaseConnection,
    headers: &HeaderMap,
) -> (Option<i64>, Option<i64>) {
    let env = parse_environment_from_headers(headers);
    match selected_session(&env, EXAM_REGISTRATIONS_SESSION_KEY) {
        None => {
            let def = default_admission_session_id(db).await;
            (def, def)
        }
        Some(None) => (None, None),
        Some(Some(id)) => (Some(id), Some(id)),
    }
}

pub fn status_label(key: &str) -> String {
    exam_status_choice_pairs()
        .into_iter()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v)
        .unwrap_or_else(|| key.to_string())
}

pub async fn load_course(db: &sea_orm::DatabaseConnection, id: i64) -> Option<course::Model> {
    if id <= 0 {
        return None;
    }
    lariv_rs::web::opt_or_log(
        CourseEntity::find_by_id(id)
            .filter(course::Column::DeletedAt.is_null())
            .one(db)
            .await,
        "db find course",
    )
}

pub async fn load_academic_record(
    db: &sea_orm::DatabaseConnection,
    id: i64,
) -> Option<academic_record::Model> {
    if id <= 0 {
        return None;
    }
    lariv_rs::web::opt_or_log(
        AcademicRecordEntity::find_by_id(id)
            .filter(academic_record::Column::DeletedAt.is_null())
            .one(db)
            .await,
        "db find academic record",
    )
}

pub fn record_label(id: i64) -> String {
    format!("Academic record #{id}")
}

pub async fn record_courses(
    db: &sea_orm::DatabaseConnection,
    academic_record_id: i64,
) -> Vec<course::Model> {
    let backend = db.get_database_backend();
    let sql = match backend {
        DatabaseBackend::Postgres => {
            "SELECT * FROM courses WHERE deleted_at IS NULL AND id IN (
                SELECT course_id FROM academic_record_compulsory_courses WHERE academic_record_id = $1
                UNION
                SELECT course_id FROM academic_record_optional_courses WHERE academic_record_id = $1
            )"
        }
        _ => {
            "SELECT * FROM courses WHERE deleted_at IS NULL AND id IN (
                SELECT course_id FROM academic_record_compulsory_courses WHERE academic_record_id = ?
                UNION
                SELECT course_id FROM academic_record_optional_courses WHERE academic_record_id = ?
            )"
        }
    };
    CourseEntity::find()
        .from_raw_sql(Statement::from_sql_and_values(
            backend,
            sql,
            [academic_record_id.into()],
        ))
        .all(db)
        .await
        .unwrap_or_default()
}

async fn find_exam(db: &sea_orm::DatabaseConnection, id: i64) -> Option<exam_registration::Model> {
    lariv_rs::web::opt_or_log(
        ExamEntity::find_by_id(id)
            .filter(exam_registration::Column::DeletedAt.is_null())
            .one(db)
            .await,
        "db find one",
    )
}

async fn load_asset_links(
    db: &sea_orm::DatabaseConnection,
    exam_id: i64,
) -> Vec<AssetLink> {
    let ids = load_asset_ids(db, exam_id).await;
    vnode_items(db, &ids)
        .await
        .into_iter()
        .filter_map(|item| {
            item.key.parse::<i64>().ok().map(|id| AssetLink {
                id,
                name: item.value,
            })
        })
        .collect()
}

async fn load_asset_ids(db: &sea_orm::DatabaseConnection, exam_id: i64) -> Vec<i64> {
    ExamAssetEntity::find()
        .filter(exam_registration_asset::Column::ExamRegistrationId.eq(exam_id))
        .all(db)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|a| a.v_node_id)
        .collect()
}

async fn replace_assets(
    db: &sea_orm::DatabaseConnection,
    exam_id: i64,
    ids: &[i64],
) -> Result<(), sea_orm::DbErr> {
    ExamAssetEntity::delete_many()
        .filter(exam_registration_asset::Column::ExamRegistrationId.eq(exam_id))
        .exec(db)
        .await?;
    for &v_node_id in ids {
        if v_node_id <= 0 {
            continue;
        }
        exam_registration_asset::ActiveModel {
            exam_registration_id: Set(exam_id),
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

async fn find_exam_scoped(
    db: &sea_orm::DatabaseConnection,
    id: i64,
    auth: &lariv_rs::plugins::users::state::AuthContext,
) -> Option<exam_registration::Model> {
    let query = ExamEntity::find_by_id(id).filter(exam_registration::Column::DeletedAt.is_null());
    let query = scope_by_role(query, auth);
    lariv_rs::web::opt_or_log(query.one(db).await, "db find one")
}

async fn query_exams(
    db: &sea_orm::DatabaseConnection,
    q: &ExamListQuery,
    auth: &lariv_rs::plugins::users::state::AuthContext,
    session_id: Option<i64>,
) -> ObjectList<ExamRow> {
    let mut query = ExamEntity::find().filter(exam_registration::Column::DeletedAt.is_null());
    query = scope_by_role(query, auth);
    query = apply_session_filter(query, session_id);
    if let Some(title) = q.exam_title.as_ref().filter(|n| !n.is_empty()) {
        query = query.filter(exam_registration::Column::ExamTitle.contains(title));
    }
    if let Some(status) = q.registration_status.as_ref().filter(|n| !n.is_empty()) {
        query = query.filter(exam_registration::Column::RegistrationStatus.eq(status.clone()));
    }
    if let Some(aid) = q.academic_record_id.filter(|id| *id > 0) {
        query = query.filter(exam_registration::Column::AcademicRecordId.eq(aid));
    }
    query = query
        .order_by_desc(exam_registration::Column::CreatedAt)
        .order_by_desc(exam_registration::Column::Id);
    let page = q.page.unwrap_or(1).max(1);
    let paginator = query.paginate(db, PAGE_SIZE as u64);
    let total = paginator.num_items().await.unwrap_or(0);
    let models = paginator
        .fetch_page((page as u64).saturating_sub(1))
        .await
        .unwrap_or_default();
    let mut rows = Vec::with_capacity(models.len());
    for e in models {
        let course_name = load_course(db, e.course_id)
            .await
            .map(|c| c.name().to_string())
            .unwrap_or_default();
        rows.push(ExamRow {
            id: e.id,
            exam_title: e.exam_title().to_string(),
            course_name,
            registration_status: status_label(&e.registration_status),
            academic_record_display: record_label(e.academic_record_id),
        });
    }
    ObjectList::from_page(rows, page, PAGE_SIZE, total)
}

fn empty_form(q: &ModalFormQuery) -> ExamFormPage {
    ExamFormPage {
        id: 0,
        exam_title: String::new(),
        registration_status: STATUS_NOT_REGISTERED.into(),
        max_marks: 0,
        marks: 0,
        fee: 0,
        course_id: 0,
        course_display: String::new(),
        academic_record_id: 0,
        academic_record_display: String::new(),
        assets: Vec::new(),
        error: String::new(),
        form_name: q.form_name(),
        refresh_table: q.refresh_table(),
        target_input: q.target_input(),
    }
}

pub async fn list(
    Cap(state): Cap<ExamRegistrationsState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    headers: HeaderMap,
    uri: Uri,
    Query(q): Query<ExamListQuery>,
) -> Response {
    if let Some(resp) = forbid_if_no_access(&ctx) {
        return resp;
    }
    let (filter_session, selected_session) = resolve_session_filter(&state.db, &headers).await;
    let exams = query_exams(&state.db, &q, &ctx, filter_session).await;
    let page = ExamListPage {
        exams,
        filter_exam_title: q.exam_title.clone().unwrap_or_default(),
        filter_registration_status: q.registration_status.clone().unwrap_or_default(),
        filter_academic_record_id: q.academic_record_id.unwrap_or(0),
        filter_academic_record_display: q
            .academic_record_id
            .filter(|id| *id > 0)
            .map(record_label)
            .unwrap_or_default(),
        path_and_query: path_and_query(&uri),
        is_admin: is_admin(&ctx),
        sessions: list_session_options(&state.db).await,
        selected_session_id: selected_session.unwrap_or(0),
    };
    if htmx.targets::<ExamTableKey>() {
        return page.render_table().into_response();
    }
    html_built_page_or_app_layout(&page, &htmx, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}

pub async fn detail(
    Cap(state): Cap<ExamRegistrationsState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Path(id): Path<i64>,
) -> Response {
    if let Some(resp) = forbid_if_no_access(&ctx) {
        return resp;
    }
    let Some(row) = find_exam_scoped(&state.db, id, &ctx).await else {
        return Redirect::to("/exam-registrations/").into_response();
    };
    let course_name = load_course(&state.db, row.course_id)
        .await
        .map(|c| c.name().to_string())
        .unwrap_or_default();
    let page = ExamDetailPage {
        id: row.id,
        exam_title: row.exam_title().to_string(),
        registration_status: status_label(&row.registration_status),
        max_marks: row.max_marks,
        marks: row.marks,
        fee: format_inr(row.fee),
        course_name,
        course_id: row.course_id,
        academic_record_display: record_label(row.academic_record_id),
        academic_record_id: row.academic_record_id,
        assets: load_asset_links(&state.db, row.id).await,
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
    Cap(state): Cap<ExamRegistrationsState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Query(q): Query<ModalFormQuery>,
    HtmlFormBody(form): HtmlFormBody<ExamForm>,
) -> Response {
    if let Some(resp) = forbid_non_admin(&ctx) {
        return resp;
    }
    let now = Utc::now();
    let status = if form.registration_status.trim().is_empty() {
        STATUS_NOT_REGISTERED.to_string()
    } else {
        form.registration_status.clone()
    };
    let model = exam_registration::ActiveModel {
        id: Default::default(),
        created_at: Set(Some(now)),
        updated_at: Set(Some(now)),
        deleted_at: Set(None),
        exam_title: Set(form.exam_title.clone()),
        max_marks: Set(form.max_marks),
        registration_status: Set(status),
        marks: Set(form.marks),
        fee: Set(form.fee),
        course_id: Set(form.course_id),
        academic_record_id: Set(form.academic_record_id),
    };
    match model.insert(&state.db).await {
        Ok(saved) => {
            let _ = replace_assets(&state.db, saved.id, &form.assets).await;
            respond_create_modal_done_fk::<ExamCreateModalKey>(
                &htmx,
                &q.refresh_table(),
                &ExamRegistrationsDetailRouteTag::new(saved.id).url(),
                saved.id,
                saved.exam_title(),
                &q.target_input(),
            )
        }
        Err(e) => {
            let mut page = empty_form(&q);
            fill_form(&state.db, &mut page, &form, e.to_string()).await;
            html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
        }
    }
}

async fn fill_form(
    db: &sea_orm::DatabaseConnection,
    page: &mut ExamFormPage,
    form: &ExamForm,
    error: String,
) {
    page.exam_title = form.exam_title.clone();
    page.registration_status = form.registration_status.clone();
    page.max_marks = form.max_marks;
    page.marks = form.marks;
    page.fee = form.fee;
    page.course_id = form.course_id;
    page.course_display = load_course(db, form.course_id)
        .await
        .map(|c| c.name().to_string())
        .unwrap_or_default();
    page.academic_record_id = form.academic_record_id;
    page.academic_record_display = if form.academic_record_id > 0 {
        record_label(form.academic_record_id)
    } else {
        String::new()
    };
    page.assets = asset_items(db, &form.assets).await;
    page.error = error;
}

pub async fn edit_get(
    Cap(state): Cap<ExamRegistrationsState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    Path(id): Path<i64>,
    Query(q): Query<ModalFormQuery>,
) -> Response {
    if let Some(resp) = forbid_non_admin(&ctx) {
        return resp;
    }
    let Some(row) = find_exam(&state.db, id).await else {
        return Redirect::to("/exam-registrations/").into_response();
    };
    let ids = load_asset_ids(&state.db, id).await;
    let course_display = load_course(&state.db, row.course_id)
        .await
        .map(|c| c.name().to_string())
        .unwrap_or_default();
    let page = ExamFormPage {
        id: row.id,
        exam_title: row.exam_title().to_string(),
        registration_status: row.registration_status().to_string(),
        max_marks: row.max_marks,
        marks: row.marks,
        fee: row.fee,
        course_id: row.course_id,
        course_display,
        academic_record_id: row.academic_record_id,
        academic_record_display: record_label(row.academic_record_id),
        assets: asset_items(&state.db, &ids).await,
        error: String::new(),
        form_name: q.form_name(),
        refresh_table: String::new(),
        target_input: String::new(),
    };
    html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}

pub async fn edit_post(
    Cap(state): Cap<ExamRegistrationsState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Path(id): Path<i64>,
    Query(q): Query<ModalFormQuery>,
    HtmlFormBody(form): HtmlFormBody<ExamForm>,
) -> Response {
    if let Some(resp) = forbid_non_admin(&ctx) {
        return resp;
    }
    let Some(existing) = find_exam(&state.db, id).await else {
        return Redirect::to("/exam-registrations/").into_response();
    };
    let now = Utc::now();
    let status = if form.registration_status.trim().is_empty() {
        existing.registration_status.clone()
    } else {
        form.registration_status.clone()
    };
    let model = exam_registration::ActiveModel {
        id: Set(existing.id),
        created_at: Set(existing.created_at),
        updated_at: Set(Some(now)),
        deleted_at: Set(existing.deleted_at),
        exam_title: Set(form.exam_title.clone()),
        max_marks: Set(form.max_marks),
        registration_status: Set(status),
        marks: Set(form.marks),
        fee: Set(form.fee),
        course_id: Set(form.course_id),
        academic_record_id: Set(form.academic_record_id),
    };
    match model.update(&state.db).await {
        Ok(_) => {
            let _ = replace_assets(&state.db, id, &form.assets).await;
            respond_edit_modal_done::<ExamEditModalKey>(
                &htmx,
                &ExamRegistrationsDetailRouteTag::new(id).url(),
            )
        }
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
        modal_uid: ExamDeleteModalKey::ID.to_string(),
        message: format!("Delete exam registration #{id}?"),
        form_name: "examregistrations.DeleteForm".into(),
        id,
        error: String::new(),
    };
    html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}

pub async fn delete_post(
    Cap(state): Cap<ExamRegistrationsState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    Path(id): Path<i64>,
) -> Response {
    if let Some(resp) = forbid_non_admin(&ctx) {
        return resp;
    }
    let Some(existing) = find_exam(&state.db, id).await else {
        return Redirect::to("/exam-registrations/").into_response();
    };
    let now = Utc::now();
    let model = exam_registration::ActiveModel {
        id: Set(existing.id),
        deleted_at: Set(Some(now)),
        ..Default::default()
    };
    match model.update(&state.db).await {
        Ok(_) => Redirect::to("/exam-registrations/").into_response(),
        Err(e) => {
            let page = ConfirmDeletePage {
                modal_uid: ExamDeleteModalKey::ID.to_string(),
                message: format!("Delete exam registration #{id}?"),
                form_name: "examregistrations.DeleteForm".into(),
                id,
                error: e.to_string(),
            };
            html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
        }
    }
}

pub async fn select(
    Cap(state): Cap<ExamRegistrationsState>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    uri: Uri,
    Query(q): Query<ExamSelectQuery>,
) -> Response {
    if let Some(resp) = forbid_if_no_access(&ctx) {
        return resp;
    }
    let exams = query_exams(&state.db, &q.filter, &ctx, None).await;
    let page = ExamSelectPage {
        exams,
        filter_exam_title: q.filter.exam_title.clone().unwrap_or_default(),
        path_and_query: path_and_query(&uri),
        target_input: q.target_input.unwrap_or_default(),
    };
    respond_picker_select::<ExamSelectTableKey, ExamSelectModalKey, _>(&htmx, &page).into_response()
}
