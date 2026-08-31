use axum::{
    extract::{Path, Query},
    http::{HeaderMap, Uri},
    response::{IntoResponse, Redirect, Response},
};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, ConnectionTrait, DatabaseBackend, EntityTrait,
    PaginatorTrait, QueryFilter, QueryOrder, Statement, sea_query::Expr,
};
use serde::Deserialize;

use crate::{
    entities::{
        assignment_submission::{self, Entity as AssignmentEntity},
        assignment_submission_asset::{self, Entity as AssignmentAssetEntity},
    },
    forms::AssignmentForm,
    keys::{
        AssignmentCreateModalKey, AssignmentDeleteModalKey, AssignmentEditModalKey,
        AssignmentTableKey,
    },
    routes::AssignmentSubmissionsDetailRouteTag,
    state::AssignmentSubmissionsState,
    templates::{
        AssetLink, AssignmentDetailPage, AssignmentFormPage, AssignmentListPage, AssignmentRow,
        ConfirmDeletePage,
    },
};
use lariv_rs::{
    components::{ManyToManyItem, ObjectList, SharedChromeFolder, SlotCtx, SwapKey},
    html_form::HtmlFormBody,
    http::Cap,
    plugins::users::middleware::RequireAuth,
    web::{
        Htmx, ModalFormQuery, html_built_page_or_app_layout, html_built_page_with_slots,
        respond_create_modal_done_fk, respond_edit_modal_done,
    },
};
use nirmancampus_academicrecords::entities::academic_record::{
    self, Entity as AcademicRecordEntity,
};
use nirmancampus_common::{
    assignment_status_choice_pairs, can_view_campus_records, is_admin, is_student, path_and_query,
    env::{
        parse_environment_from_headers, selected_session, ASSIGNMENT_SUBMISSIONS_SESSION_KEY,
    },
    ui::SessionOption,
    vnode_items,
};
use nirmancampus_courses::entities::course::{self, Entity as CourseEntity};
use nirmancampus_sessions::entities::admission_session::{self, Entity as SessionEntity};

const PAGE_SIZE: u32 = 20;
pub const STATUS_NOT_MARKED: &str = "not_marked";
pub const STATUS_MARKED: &str = "marked";

#[derive(Debug, Deserialize, Default)]
pub struct AssignmentListQuery {
    #[serde(default, rename = "AssignmentTitle", alias = "assignment_title")]
    pub assignment_title: Option<String>,
    #[serde(default, rename = "SubmissionStatus", alias = "submission_status")]
    pub submission_status: Option<String>,
    #[serde(default, rename = "AcademicRecordID", alias = "academic_record_id")]
    pub academic_record_id: Option<i64>,
    #[serde(default)]
    pub page: Option<u32>,
    #[serde(default)]
    pub sort: Option<String>,
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
    query: sea_orm::Select<AssignmentEntity>,
    auth: &lariv_rs::plugins::users::state::AuthContext,
) -> sea_orm::Select<AssignmentEntity> {
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
    query: sea_orm::Select<AssignmentEntity>,
    session_id: Option<i64>,
) -> sea_orm::Select<AssignmentEntity> {
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
    match selected_session(&env, ASSIGNMENT_SUBMISSIONS_SESSION_KEY) {
        None => {
            let def = default_admission_session_id(db).await;
            (def, def)
        }
        Some(None) => (None, None),
        Some(Some(id)) => (Some(id), Some(id)),
    }
}

pub fn status_label(key: &str) -> String {
    assignment_status_choice_pairs()
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

pub async fn find_assignment(
    db: &sea_orm::DatabaseConnection,
    id: i64,
) -> Option<assignment_submission::Model> {
    lariv_rs::web::opt_or_log(
        AssignmentEntity::find_by_id(id)
            .filter(assignment_submission::Column::DeletedAt.is_null())
            .one(db)
            .await,
        "db find one",
    )
}

async fn find_assignment_scoped(
    db: &sea_orm::DatabaseConnection,
    id: i64,
    auth: &lariv_rs::plugins::users::state::AuthContext,
) -> Option<assignment_submission::Model> {
    let query =
        AssignmentEntity::find_by_id(id).filter(assignment_submission::Column::DeletedAt.is_null());
    let query = scope_by_role(query, auth);
    lariv_rs::web::opt_or_log(query.one(db).await, "db find one")
}

pub async fn load_academic_record_visible(
    db: &sea_orm::DatabaseConnection,
    id: i64,
    auth: &lariv_rs::plugins::users::state::AuthContext,
) -> Option<academic_record::Model> {
    let mut query = AcademicRecordEntity::find_by_id(id)
        .filter(academic_record::Column::DeletedAt.is_null());
    if is_admin(auth) {
        return lariv_rs::web::opt_or_log(query.one(db).await, "db find academic record");
    }
    if is_student(auth) {
        let email = auth.user.email.trim().to_string();
        if email.is_empty() {
            return None;
        }
        query = query.filter(Expr::cust_with_values(
            "student_id IN (SELECT id FROM students WHERE email = ? AND deleted_at IS NULL)",
            [email],
        ));
        return lariv_rs::web::opt_or_log(query.one(db).await, "db find academic record");
    }
    None
}

async fn load_asset_ids(db: &sea_orm::DatabaseConnection, assignment_id: i64) -> Vec<i64> {
    AssignmentAssetEntity::find()
        .filter(assignment_submission_asset::Column::AssignmentSubmissionId.eq(assignment_id))
        .all(db)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|a| a.v_node_id)
        .collect()
}

async fn load_asset_links(
    db: &sea_orm::DatabaseConnection,
    assignment_id: i64,
) -> Vec<AssetLink> {
    let ids = load_asset_ids(db, assignment_id).await;
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

async fn replace_assets(
    db: &sea_orm::DatabaseConnection,
    assignment_id: i64,
    ids: &[i64],
) -> Result<(), sea_orm::DbErr> {
    AssignmentAssetEntity::delete_many()
        .filter(assignment_submission_asset::Column::AssignmentSubmissionId.eq(assignment_id))
        .exec(db)
        .await?;
    for &v_node_id in ids {
        if v_node_id <= 0 {
            continue;
        }
        assignment_submission_asset::ActiveModel {
            assignment_submission_id: Set(assignment_id),
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

async fn query_assignments(
    db: &sea_orm::DatabaseConnection,
    q: &AssignmentListQuery,
    auth: &lariv_rs::plugins::users::state::AuthContext,
    session_id: Option<i64>,
) -> ObjectList<AssignmentRow> {
    let mut query =
        AssignmentEntity::find().filter(assignment_submission::Column::DeletedAt.is_null());
    query = scope_by_role(query, auth);
    query = apply_session_filter(query, session_id);
    if let Some(title) = q.assignment_title.as_ref().filter(|n| !n.is_empty()) {
        query = query.filter(assignment_submission::Column::AssignmentTitle.contains(title));
    }
    if let Some(status) = q.submission_status.as_ref().filter(|n| !n.is_empty()) {
        query = query.filter(assignment_submission::Column::SubmissionStatus.eq(status.clone()));
    }
    if let Some(aid) = q.academic_record_id.filter(|id| *id > 0) {
        query = query.filter(assignment_submission::Column::AcademicRecordId.eq(aid));
    }
    query = query
        .order_by_desc(assignment_submission::Column::CreatedAt)
        .order_by_desc(assignment_submission::Column::Id);
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
        rows.push(AssignmentRow {
            id: e.id,
            assignment_title: e.assignment_title().to_string(),
            course_name,
            submission_status: status_label(&e.submission_status),
            academic_record_display: record_label(e.academic_record_id),
        });
    }
    ObjectList::from_page(rows, page, PAGE_SIZE, total)
}

fn empty_form(q: &ModalFormQuery) -> AssignmentFormPage {
    AssignmentFormPage {
        id: 0,
        assignment_title: String::new(),
        submission_status: STATUS_NOT_MARKED.into(),
        max_marks: 0,
        marks: 0,
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

async fn fill_form(
    db: &sea_orm::DatabaseConnection,
    page: &mut AssignmentFormPage,
    form: &AssignmentForm,
    error: String,
) {
    page.assignment_title = form.assignment_title.clone();
    page.submission_status = form.submission_status.clone();
    page.max_marks = form.max_marks;
    page.marks = form.marks;
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

pub fn academic_record_detail_url(academic_record_id: i64) -> String {
    format!("/academic-records/{academic_record_id}/")
}

pub async fn list(
    Cap(state): Cap<AssignmentSubmissionsState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    headers: HeaderMap,
    uri: Uri,
    Query(q): Query<AssignmentListQuery>,
) -> Response {
    if let Some(resp) = forbid_if_no_access(&ctx) {
        return resp;
    }
    let (filter_session, selected_session) = resolve_session_filter(&state.db, &headers).await;
    let assignments = query_assignments(&state.db, &q, &ctx, filter_session).await;
    let page = AssignmentListPage {
        assignments,
        filter_assignment_title: q.assignment_title.clone().unwrap_or_default(),
        filter_submission_status: q.submission_status.clone().unwrap_or_default(),
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
    if htmx.targets::<AssignmentTableKey>() {
        return page.render_table().into_response();
    }
    html_built_page_or_app_layout(&page, &htmx, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}

pub async fn detail(
    Cap(state): Cap<AssignmentSubmissionsState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Path(id): Path<i64>,
) -> Response {
    if let Some(resp) = forbid_if_no_access(&ctx) {
        return resp;
    }
    let Some(row) = find_assignment_scoped(&state.db, id, &ctx).await else {
        return Redirect::to("/assignment-submissions/").into_response();
    };
    let course_name = load_course(&state.db, row.course_id)
        .await
        .map(|c| c.name().to_string())
        .unwrap_or_default();
    let page = AssignmentDetailPage {
        id: row.id,
        assignment_title: row.assignment_title().to_string(),
        submission_status: status_label(&row.submission_status),
        max_marks: row.max_marks,
        marks: row.marks,
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
    Cap(state): Cap<AssignmentSubmissionsState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Query(q): Query<ModalFormQuery>,
    HtmlFormBody(form): HtmlFormBody<AssignmentForm>,
) -> Response {
    if let Some(resp) = forbid_non_admin(&ctx) {
        return resp;
    }
    let now = Utc::now();
    let status = if form.submission_status.trim().is_empty() {
        STATUS_NOT_MARKED.to_string()
    } else {
        form.submission_status.clone()
    };
    let model = assignment_submission::ActiveModel {
        id: Default::default(),
        created_at: Set(Some(now)),
        updated_at: Set(Some(now)),
        deleted_at: Set(None),
        assignment_title: Set(form.assignment_title.clone()),
        max_marks: Set(form.max_marks),
        submission_status: Set(status),
        marks: Set(form.marks),
        course_id: Set(form.course_id),
        academic_record_id: Set(form.academic_record_id),
    };
    match model.insert(&state.db).await {
        Ok(saved) => {
            let _ = replace_assets(&state.db, saved.id, &form.assets).await;
            respond_create_modal_done_fk::<AssignmentCreateModalKey>(
                &htmx,
                &q.refresh_table(),
                &AssignmentSubmissionsDetailRouteTag::new(saved.id).url(),
                saved.id,
                saved.assignment_title(),
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

pub async fn edit_get(
    Cap(state): Cap<AssignmentSubmissionsState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    Path(id): Path<i64>,
    Query(q): Query<ModalFormQuery>,
) -> Response {
    if let Some(resp) = forbid_non_admin(&ctx) {
        return resp;
    }
    let Some(row) = find_assignment(&state.db, id).await else {
        return Redirect::to("/assignment-submissions/").into_response();
    };
    let ids = load_asset_ids(&state.db, id).await;
    let course_display = load_course(&state.db, row.course_id)
        .await
        .map(|c| c.name().to_string())
        .unwrap_or_default();
    let page = AssignmentFormPage {
        id: row.id,
        assignment_title: row.assignment_title().to_string(),
        submission_status: row.submission_status().to_string(),
        max_marks: row.max_marks,
        marks: row.marks,
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
    Cap(state): Cap<AssignmentSubmissionsState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Path(id): Path<i64>,
    Query(q): Query<ModalFormQuery>,
    HtmlFormBody(form): HtmlFormBody<AssignmentForm>,
) -> Response {
    if let Some(resp) = forbid_non_admin(&ctx) {
        return resp;
    }
    let Some(existing) = find_assignment(&state.db, id).await else {
        return Redirect::to("/assignment-submissions/").into_response();
    };
    let now = Utc::now();
    let status = if form.submission_status.trim().is_empty() {
        existing.submission_status.clone()
    } else {
        form.submission_status.clone()
    };
    let model = assignment_submission::ActiveModel {
        id: Set(existing.id),
        created_at: Set(existing.created_at),
        updated_at: Set(Some(now)),
        deleted_at: Set(existing.deleted_at),
        assignment_title: Set(form.assignment_title.clone()),
        max_marks: Set(form.max_marks),
        submission_status: Set(status),
        marks: Set(form.marks),
        course_id: Set(form.course_id),
        academic_record_id: Set(form.academic_record_id),
    };
    match model.update(&state.db).await {
        Ok(_) => {
            let _ = replace_assets(&state.db, id, &form.assets).await;
            respond_edit_modal_done::<AssignmentEditModalKey>(
                &htmx,
                &AssignmentSubmissionsDetailRouteTag::new(id).url(),
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
        modal_uid: AssignmentDeleteModalKey::ID.to_string(),
        message: format!("Delete assignment submission #{id}?"),
        form_name: "assignmentsubmissions.DeleteForm".into(),
        id,
        error: String::new(),
    };
    html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}

pub async fn delete_post(
    Cap(state): Cap<AssignmentSubmissionsState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    Path(id): Path<i64>,
) -> Response {
    if let Some(resp) = forbid_non_admin(&ctx) {
        return resp;
    }
    let Some(existing) = find_assignment(&state.db, id).await else {
        return Redirect::to("/assignment-submissions/").into_response();
    };
    let now = Utc::now();
    let model = assignment_submission::ActiveModel {
        id: Set(existing.id),
        deleted_at: Set(Some(now)),
        ..Default::default()
    };
    match model.update(&state.db).await {
        Ok(_) => Redirect::to("/assignment-submissions/").into_response(),
        Err(e) => {
            let page = ConfirmDeletePage {
                modal_uid: AssignmentDeleteModalKey::ID.to_string(),
                message: format!("Delete assignment submission #{id}?"),
                form_name: "assignmentsubmissions.DeleteForm".into(),
                id,
                error: e.to_string(),
            };
            html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
        }
    }
}
