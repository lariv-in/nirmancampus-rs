use axum::{
    body::Body,
    extract::{Path, Query},
    http::{header, HeaderMap, HeaderValue, StatusCode, Uri},
    response::{IntoResponse, Redirect, Response},
};
use chrono::{NaiveDate, Utc};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, sea_query::Expr,
};
use serde::Deserialize;

use crate::{
    academic_record_detail_related::related_sections_html,
    entities::{
        academic_record::{self, Entity as AcademicRecordEntity},
        academic_record_compulsory_course::{self, Entity as CompulsoryEntity},
        academic_record_optional_course::{self, Entity as OptionalEntity},
    },
    forms::AcademicRecordForm,
    keys::{
        AcademicRecordCreateModalKey, AcademicRecordDeleteModalKey, AcademicRecordEditModalKey,
        AcademicRecordSelectModalKey, AcademicRecordSelectTableKey, AcademicRecordTableKey,
        PsuSelectModalKey, PsuSelectTableKey,
    },
    routes::AcademicRecordsDetailRouteTag,
    state::AcademicRecordsState,
    templates::{
        AcademicRecordDetailPage, AcademicRecordFormPage, AcademicRecordListPage,
        AcademicRecordRow, AcademicRecordSelectPage, ConfirmDeletePage, CourseLink, PsuRow,
        PsuSelectPage,
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
    doc_export::{attachment_filename, export_pdf},
    env::{parse_environment_from_headers, selected_academic_record_session},
    is_admin, is_student, path_and_query, program_display,
    ui::SessionOption,
};
use nirmancampus_courses::entities::course::{self, Entity as CourseEntity};
use nirmancampus_programs::entities::{
    program::{self, Entity as ProgramEntity},
    program_structure_unit::{self, Entity as PsuEntity},
    program_structure_unit_compulsory_course::{self, Entity as PsuCompulsoryEntity},
    program_structure_unit_optional_course::{self, Entity as PsuOptionalEntity},
};
use nirmancampus_sessions::entities::admission_session::{self, Entity as SessionEntity};
use nirmancampus_students::entities::student::{self, Entity as StudentEntity};

const PAGE_SIZE: u32 = 20;
const DEFAULT_STATUS: &str = "Not Applied";

#[derive(Debug, Deserialize, Default)]
pub struct RecordListQuery {
    #[serde(default, rename = "Status", alias = "status")]
    pub status: Option<String>,
    #[serde(default, rename = "Term", alias = "term")]
    pub term: Option<String>,
    #[serde(default, rename = "ProgramID", alias = "program_id")]
    pub program_id: Option<i64>,
    #[serde(default)]
    pub page: Option<u32>,
}

#[derive(Debug, Deserialize, Default)]
pub struct RecordSelectQuery {
    #[serde(flatten)]
    pub filter: RecordListQuery,
    #[serde(default)]
    pub target_input: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct RecordCreateQuery {
    #[serde(flatten)]
    pub modal: ModalFormQuery,
    #[serde(default, rename = "StudentID", alias = "student_id")]
    pub student_id: Option<i64>,
}

#[derive(Debug, Deserialize, Default)]
pub struct PsuSelectQuery {
    #[serde(default, rename = "ProgramID", alias = "program_id")]
    pub program_id: Option<i64>,
    #[serde(default)]
    pub target_input: Option<String>,
    #[serde(default)]
    pub page: Option<u32>,
}

fn forbid_if_no_access(
    ctx: &lariv_rs::plugins::users::state::AuthContext,
) -> Option<Response> {
    if is_admin(ctx) || is_student(ctx) {
        None
    } else {
        Some(Redirect::to("/").into_response())
    }
}

fn forbid_non_admin(ctx: &lariv_rs::plugins::users::state::AuthContext) -> Option<Response> {
    if is_admin(ctx) {
        None
    } else {
        Some(Redirect::to("/").into_response())
    }
}

fn scope_records(
    query: sea_orm::Select<AcademicRecordEntity>,
    auth: &lariv_rs::plugins::users::state::AuthContext,
) -> sea_orm::Select<AcademicRecordEntity> {
    if is_admin(auth) {
        return query;
    }
    if is_student(auth) {
        let email = auth.user.email.trim().to_string();
        if email.is_empty() {
            return query.filter(Expr::cust("1 = 0"));
        }
        return query.filter(Expr::cust_with_values(
            "student_id IN (SELECT id FROM students WHERE email = ? AND deleted_at IS NULL)",
            [email],
        ));
    }
    query.filter(Expr::cust("1 = 0"))
}

pub async fn load_program_structure_unit(
    db: &sea_orm::DatabaseConnection,
    id: i64,
) -> Option<program_structure_unit::Model> {
    if id <= 0 {
        return None;
    }
    lariv_rs::web::opt_or_log(
        PsuEntity::find_by_id(id)
            .filter(program_structure_unit::Column::DeletedAt.is_null())
            .one(db)
            .await,
        "db find psu",
    )
}

pub fn term_label(psu: &program_structure_unit::Model) -> String {
    format!("Term {}", psu.term_number)
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
    match selected_academic_record_session(&env) {
        None => {
            let def = default_admission_session_id(db).await;
            (def, def)
        }
        Some(None) => (None, None),
        Some(Some(id)) => (Some(id), Some(id)),
    }
}

async fn load_student(db: &sea_orm::DatabaseConnection, id: i64) -> Option<student::Model> {
    if id <= 0 {
        return None;
    }
    lariv_rs::web::opt_or_log(
        StudentEntity::find_by_id(id)
            .filter(student::Column::DeletedAt.is_null())
            .one(db)
            .await,
        "db find student",
    )
}

async fn load_program(db: &sea_orm::DatabaseConnection, id: i64) -> Option<program::Model> {
    if id <= 0 {
        return None;
    }
    lariv_rs::web::opt_or_log(
        ProgramEntity::find_by_id(id)
            .filter(program::Column::DeletedAt.is_null())
            .one(db)
            .await,
        "db find program",
    )
}

async fn load_session(db: &sea_orm::DatabaseConnection, id: i64) -> Option<admission_session::Model> {
    if id <= 0 {
        return None;
    }
    lariv_rs::web::opt_or_log(
        SessionEntity::find_by_id(id)
            .filter(admission_session::Column::DeletedAt.is_null())
            .one(db)
            .await,
        "db find session",
    )
}

async fn load_courses(db: &sea_orm::DatabaseConnection, ids: &[i64]) -> Vec<course::Model> {
    if ids.is_empty() {
        return Vec::new();
    }
    CourseEntity::find()
        .filter(course::Column::DeletedAt.is_null())
        .filter(course::Column::Id.is_in(ids.iter().copied()))
        .all(db)
        .await
        .unwrap_or_default()
}

async fn load_compulsory_ids(db: &sea_orm::DatabaseConnection, record_id: i64) -> Vec<i64> {
    CompulsoryEntity::find()
        .filter(academic_record_compulsory_course::Column::AcademicRecordId.eq(record_id))
        .all(db)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|r| r.course_id)
        .collect()
}

async fn load_optional_ids(db: &sea_orm::DatabaseConnection, record_id: i64) -> Vec<i64> {
    OptionalEntity::find()
        .filter(academic_record_optional_course::Column::AcademicRecordId.eq(record_id))
        .all(db)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|r| r.course_id)
        .collect()
}

async fn psu_compulsory_ids(db: &sea_orm::DatabaseConnection, psu_id: i64) -> Vec<i64> {
    PsuCompulsoryEntity::find()
        .filter(program_structure_unit_compulsory_course::Column::ProgramStructureUnitId.eq(psu_id))
        .all(db)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|r| r.course_id)
        .collect()
}

async fn psu_optional_pool_ids(db: &sea_orm::DatabaseConnection, psu_id: i64) -> Vec<i64> {
    PsuOptionalEntity::find()
        .filter(program_structure_unit_optional_course::Column::ProgramStructureUnitId.eq(psu_id))
        .all(db)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|r| r.course_id)
        .collect()
}

fn course_items(courses: &[course::Model]) -> Vec<ManyToManyItem> {
    courses
        .iter()
        .map(|c| ManyToManyItem::new(c.id.to_string(), c.name().to_string()))
        .collect()
}

fn course_links(courses: &[course::Model]) -> Vec<CourseLink> {
    courses
        .iter()
        .map(|c| CourseLink {
            id: c.id,
            name: c.name().to_string(),
        })
        .collect()
}

async fn find_record_scoped(
    db: &sea_orm::DatabaseConnection,
    id: i64,
    auth: &lariv_rs::plugins::users::state::AuthContext,
) -> Option<academic_record::Model> {
    let query = AcademicRecordEntity::find_by_id(id)
        .filter(academic_record::Column::DeletedAt.is_null());
    let query = scope_records(query, auth);
    lariv_rs::web::opt_or_log(query.one(db).await, "db find academic record")
}

struct DisplayBits {
    student_name: String,
    student_no: String,
    student_display: String,
    program_display: String,
    session_name: String,
    term: String,
    status: String,
    date: String,
    compulsory: Vec<CourseLink>,
    optional: Vec<CourseLink>,
}

async fn display_bits(db: &sea_orm::DatabaseConnection, row: &academic_record::Model) -> DisplayBits {
    let student = load_student(db, row.student_id).await;
    let (student_name, student_no, student_display) = match &student {
        Some(s) => (
            s.name().to_string(),
            s.student_no.clone(),
            format!("{} ({})", s.name(), s.student_no),
        ),
        None => (
            format!("Student #{}", row.student_id),
            String::new(),
            format!("Student #{}", row.student_id),
        ),
    };
    let program_display = match load_program(db, row.program_id).await {
        Some(p) => program_display(p.name(), &p.university),
        None => format!("Program #{}", row.program_id),
    };
    let session_name = load_session(db, row.session_id)
        .await
        .map(|s| s.name().to_string())
        .unwrap_or_else(|| format!("Session #{}", row.session_id));
    let term = load_program_structure_unit(db, row.program_structure_unit_id)
        .await
        .map(|u| term_label(&u))
        .unwrap_or_default();
    let compulsory_ids = load_compulsory_ids(db, row.id).await;
    let optional_ids = load_optional_ids(db, row.id).await;
    let compulsory = course_links(&load_courses(db, &compulsory_ids).await);
    let optional = course_links(&load_courses(db, &optional_ids).await);
    DisplayBits {
        student_name,
        student_no,
        student_display,
        program_display,
        session_name,
        term,
        status: row.status().to_string(),
        date: row.date.map(format_date).unwrap_or_default(),
        compulsory,
        optional,
    }
}

async fn query_records(
    db: &sea_orm::DatabaseConnection,
    auth: &lariv_rs::plugins::users::state::AuthContext,
    q: &RecordListQuery,
    session_id: Option<i64>,
) -> ObjectList<AcademicRecordRow> {
    let mut query = AcademicRecordEntity::find().filter(academic_record::Column::DeletedAt.is_null());
    query = scope_records(query, auth);
    if let Some(sid) = session_id {
        query = query.filter(academic_record::Column::SessionId.eq(sid));
    }
    if let Some(status) = q.status.as_ref().filter(|s| !s.is_empty()) {
        query = query.filter(academic_record::Column::Status.eq(status.clone()));
    }
    if let Some(pid) = q.program_id.filter(|id| *id > 0) {
        query = query.filter(academic_record::Column::ProgramId.eq(pid));
    }
    if let Some(term) = q.term.as_ref().filter(|t| !t.is_empty())
        && let Ok(n) = term.trim().parse::<i64>()
    {
        let psu_ids: Vec<i64> = PsuEntity::find()
            .filter(program_structure_unit::Column::DeletedAt.is_null())
            .filter(program_structure_unit::Column::TermNumber.eq(n))
            .all(db)
            .await
            .unwrap_or_default()
            .into_iter()
            .map(|u| u.id)
            .collect();
        query = query.filter(academic_record::Column::ProgramStructureUnitId.is_in(psu_ids));
    }
    query = query.order_by_desc(academic_record::Column::Id);
    let page = q.page.unwrap_or(1).max(1);
    let paginator = query.paginate(db, PAGE_SIZE as u64);
    let total = paginator.num_items().await.unwrap_or(0);
    let models = paginator
        .fetch_page((page as u64).saturating_sub(1))
        .await
        .unwrap_or_default();
    let mut rows = Vec::with_capacity(models.len());
    for r in models {
        let bits = display_bits(db, &r).await;
        rows.push(AcademicRecordRow {
            id: r.id,
            student_display: bits.student_display,
            program_display: bits.program_display,
            term: bits.term,
            session_name: bits.session_name,
            status: bits.status,
        });
    }
    ObjectList::from_page(rows, page, PAGE_SIZE, total)
}

fn validate_optional(
    optional: &[i64],
    psu: &program_structure_unit::Model,
    pool: &[i64],
) -> Result<(), String> {
    let want = psu.optional_course_count();
    if optional.len() as i64 != want {
        return Err(format!(
            "select exactly {want} optional course(s) for this program term"
        ));
    }
    for id in optional {
        if !pool.contains(id) {
            return Err("select optional courses from the selected program pool".into());
        }
    }
    Ok(())
}

async fn replace_optional(
    db: &sea_orm::DatabaseConnection,
    record_id: i64,
    ids: &[i64],
) -> Result<(), sea_orm::DbErr> {
    OptionalEntity::delete_many()
        .filter(academic_record_optional_course::Column::AcademicRecordId.eq(record_id))
        .exec(db)
        .await?;
    for &course_id in ids {
        if course_id <= 0 {
            continue;
        }
        academic_record_optional_course::ActiveModel {
            academic_record_id: Set(record_id),
            course_id: Set(course_id),
        }
        .insert(db)
        .await?;
    }
    Ok(())
}

async fn insert_compulsory(
    db: &sea_orm::DatabaseConnection,
    record_id: i64,
    ids: &[i64],
) -> Result<(), sea_orm::DbErr> {
    for &course_id in ids {
        if course_id <= 0 {
            continue;
        }
        academic_record_compulsory_course::ActiveModel {
            academic_record_id: Set(record_id),
            course_id: Set(course_id),
        }
        .insert(db)
        .await?;
    }
    Ok(())
}

fn optional_courses_multi_select_url(pool_ids: &[i64]) -> String {
    let joined = pool_ids
        .iter()
        .map(|id| id.to_string())
        .collect::<Vec<_>>()
        .join(",");
    format!("/courses/multi-select/?pool_course_ids={joined}")
}

async fn optional_pool_url_for_psu(db: &sea_orm::DatabaseConnection, psu_id: i64) -> String {
    if psu_id <= 0 {
        return optional_courses_multi_select_url(&[]);
    }
    let ids = psu_optional_pool_ids(db, psu_id).await;
    optional_courses_multi_select_url(&ids)
}

fn empty_form(q: &ModalFormQuery, student_id: i64) -> AcademicRecordFormPage {
    AcademicRecordFormPage {
        id: 0,
        session_id: 0,
        session_display: String::new(),
        student_id,
        student_display: String::new(),
        program_id: 0,
        program_display: String::new(),
        status: DEFAULT_STATUS.into(),
        date: format_date(chrono::Local::now().date_naive()),
        program_structure_unit_id: 0,
        term_display: String::new(),
        optional_course_count: String::new(),
        compulsory: Vec::new(),
        optional: Vec::new(),
        optional_pool_url: optional_courses_multi_select_url(&[]),
        error: String::new(),
        form_name: q.form_name(),
        refresh_table: q.refresh_table(),
        target_input: q.target_input(),
        locked_identity: false,
    }
}

async fn fill_form(
    db: &sea_orm::DatabaseConnection,
    page: &mut AcademicRecordFormPage,
    form: &AcademicRecordForm,
    error: String,
    compulsory: Vec<ManyToManyItem>,
    optional: Vec<ManyToManyItem>,
    optional_course_count: String,
) {
    page.session_id = form.session_id;
    page.student_id = form.student_id;
    page.program_id = form.program_id;
    page.status = form.status.clone();
    page.date = form.date.clone();
    page.program_structure_unit_id = form.program_structure_unit_id;
    page.compulsory = compulsory;
    page.optional = optional;
    page.optional_course_count = optional_course_count;
    page.error = error;
    page.optional_pool_url = optional_pool_url_for_psu(db, page.program_structure_unit_id).await;
}

pub async fn list(
    Cap(state): Cap<AcademicRecordsState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    uri: Uri,
    headers: HeaderMap,
    Query(q): Query<RecordListQuery>,
) -> Response {
    if let Some(resp) = forbid_if_no_access(&ctx) {
        return resp;
    }
    let (filter_session, selected_session) = resolve_session_filter(&state.db, &headers).await;
    let records = query_records(&state.db, &ctx, &q, filter_session).await;
    let program_display = if let Some(pid) = q.program_id.filter(|id| *id > 0) {
        load_program(&state.db, pid)
            .await
            .map(|p| program_display(p.name(), &p.university))
            .unwrap_or_default()
    } else {
        String::new()
    };
    let page = AcademicRecordListPage {
        records,
        filter_status: q.status.clone().unwrap_or_default(),
        filter_term: q.term.clone().unwrap_or_default(),
        filter_program_id: q.program_id.unwrap_or(0),
        filter_program_display: program_display,
        path_and_query: path_and_query(&uri),
        is_admin: is_admin(&ctx),
        sessions: list_session_options(&state.db).await,
        selected_session_id: selected_session.unwrap_or(0),
    };
    if htmx.targets::<AcademicRecordTableKey>() {
        return page.render_table().into_response();
    }
    html_built_page_or_app_layout(&page, &htmx, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}

pub async fn detail(
    Cap(state): Cap<AcademicRecordsState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Path(id): Path<i64>,
) -> Response {
    if let Some(resp) = forbid_if_no_access(&ctx) {
        return resp;
    }
    let Some(row) = find_record_scoped(&state.db, id, &ctx).await else {
        return Redirect::to("/academic-records/").into_response();
    };
    let bits = display_bits(&state.db, &row).await;
    let related_sections = related_sections_html(&state.db, row.id, &ctx).await;
    let page = AcademicRecordDetailPage {
        id: row.id,
        student_id: row.student_id,
        student_name: bits.student_name,
        student_no: bits.student_no,
        program_id: row.program_id,
        program_display: bits.program_display,
        session_id: row.session_id,
        session_name: bits.session_name,
        status: bits.status,
        date: bits.date,
        term: bits.term,
        compulsory: bits.compulsory,
        optional: bits.optional,
        related_sections,
        is_admin: is_admin(&ctx),
    };
    html_built_page_or_app_layout(&page, &htmx, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}

pub async fn create_get(
    Cap(state): Cap<AcademicRecordsState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    Query(q): Query<RecordCreateQuery>,
) -> Response {
    if let Some(resp) = forbid_non_admin(&ctx) {
        return resp;
    }
    let student_id = q.student_id.filter(|id| *id > 0).unwrap_or(0);
    let mut page = empty_form(&q.modal, student_id);
    if student_id > 0
        && let Some(s) = load_student(&state.db, student_id).await
    {
        page.student_display = format!("{} ({})", s.name(), s.student_no);
    }
    html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}

async fn form_displays(
    db: &sea_orm::DatabaseConnection,
    form: &AcademicRecordForm,
) -> (String, String, String, String, Vec<ManyToManyItem>, String) {
    let session_display = load_session(db, form.session_id)
        .await
        .map(|s| s.name().to_string())
        .unwrap_or_default();
    let student_display = load_student(db, form.student_id)
        .await
        .map(|s| format!("{} ({})", s.name(), s.student_no))
        .unwrap_or_default();
    let program_display = load_program(db, form.program_id)
        .await
        .map(|p| program_display(p.name(), &p.university))
        .unwrap_or_default();
    let psu = load_program_structure_unit(db, form.program_structure_unit_id).await;
    let term_display = psu.as_ref().map(term_label).unwrap_or_default();
    let count = psu
        .as_ref()
        .map(|u| u.optional_course_count().to_string())
        .unwrap_or_else(|| "—".into());
    let compulsory_ids = if let Some(u) = &psu {
        psu_compulsory_ids(db, u.id).await
    } else {
        Vec::new()
    };
    let compulsory = course_items(&load_courses(db, &compulsory_ids).await);
    (
        session_display,
        student_display,
        program_display,
        term_display,
        compulsory,
        count,
    )
}

pub async fn create_post(
    Cap(state): Cap<AcademicRecordsState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Query(q): Query<ModalFormQuery>,
    HtmlFormBody(form): HtmlFormBody<AcademicRecordForm>,
) -> Response {
    if let Some(resp) = forbid_non_admin(&ctx) {
        return resp;
    }
    let mut page = empty_form(&q, form.student_id);
    let (session_display, student_display, program_display, term_display, compulsory, count) =
        form_displays(&state.db, &form).await;
    page.session_display = session_display;
    page.student_display = student_display;
    page.program_display = program_display;
    page.term_display = term_display;
    page.optional = course_items(&load_courses(&state.db, &form.optional_courses).await);
    let optional_items = page.optional.clone();

    if form.program_id <= 0 {
        fill_form(
            &state.db,
            &mut page,
            &form,
            "select a program".into(),
            compulsory,
            optional_items.clone(),
            count,
        )
        .await;
        return html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx)).into_response();
    }
    let Some(psu) = load_program_structure_unit(&state.db, form.program_structure_unit_id).await
    else {
        fill_form(
            &state.db,
            &mut page,
            &form,
            "select a term".into(),
            compulsory,
            optional_items.clone(),
            count,
        )
        .await;
        return html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx)).into_response();
    };
    if psu.program_id != form.program_id {
        fill_form(
            &state.db,
            &mut page,
            &form,
            "select a valid term for this program".into(),
            compulsory,
            optional_items.clone(),
            "—".into(),
        )
        .await;
        return html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx)).into_response();
    }
    let pool = psu_optional_pool_ids(&state.db, psu.id).await;
    if let Err(e) = validate_optional(&form.optional_courses, &psu, &pool) {
        fill_form(
            &state.db,
            &mut page,
            &form,
            e,
            compulsory,
            optional_items.clone(),
            psu.optional_course_count().to_string(),
        )
        .await;
        return html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx)).into_response();
    }
    if form.student_id <= 0 || form.session_id <= 0 {
        fill_form(
            &state.db,
            &mut page,
            &form,
            "select a student and admission session".into(),
            compulsory,
            optional_items.clone(),
            psu.optional_course_count().to_string(),
        )
        .await;
        return html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx)).into_response();
    }
    let status = if form.status.trim().is_empty() {
        DEFAULT_STATUS.to_string()
    } else {
        form.status.clone()
    };
    let date: Option<NaiveDate> = parse_date(&form.date).or_else(|| Some(chrono::Local::now().date_naive()));
    let now = Utc::now();
    let compulsory_ids = psu_compulsory_ids(&state.db, psu.id).await;
    let model = academic_record::ActiveModel {
        id: Default::default(),
        created_at: Set(Some(now)),
        updated_at: Set(Some(now)),
        deleted_at: Set(None),
        student_id: Set(form.student_id),
        program_id: Set(form.program_id),
        session_id: Set(form.session_id),
        program_structure_unit_id: Set(psu.id),
        date: Set(date),
        status: Set(status),
    };
    match model.insert(&state.db).await {
        Ok(saved) => {
            let _ = insert_compulsory(&state.db, saved.id, &compulsory_ids).await;
            let _ = replace_optional(&state.db, saved.id, &form.optional_courses).await;
            respond_create_modal_done_fk::<AcademicRecordCreateModalKey>(
                &htmx,
                &q.refresh_table(),
                &AcademicRecordsDetailRouteTag::new(saved.id).url(),
                saved.id,
                &format!("Academic record #{}", saved.id),
                &q.target_input(),
            )
        }
        Err(e) => {
            fill_form(
                &state.db,
                &mut page,
                &form,
                e.to_string(),
                compulsory,
                optional_items.clone(),
                psu.optional_course_count().to_string(),
            )
            .await;
            html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
        }
    }
}

pub async fn edit_get(
    Cap(state): Cap<AcademicRecordsState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    Path(id): Path<i64>,
    Query(q): Query<ModalFormQuery>,
) -> Response {
    if let Some(resp) = forbid_non_admin(&ctx) {
        return resp;
    }
    let Some(row) = find_record_scoped(&state.db, id, &ctx).await else {
        return Redirect::to("/academic-records/").into_response();
    };
    let bits = display_bits(&state.db, &row).await;
    let psu = load_program_structure_unit(&state.db, row.program_structure_unit_id).await;
    let optional_ids = load_optional_ids(&state.db, row.id).await;
    let compulsory_ids = load_compulsory_ids(&state.db, row.id).await;
    let page = AcademicRecordFormPage {
        id: row.id,
        session_id: row.session_id,
        session_display: bits.session_name,
        student_id: row.student_id,
        student_display: bits.student_display,
        program_id: row.program_id,
        program_display: bits.program_display,
        status: row.status().to_string(),
        date: row.date.map(format_date).unwrap_or_default(),
        program_structure_unit_id: row.program_structure_unit_id,
        term_display: bits.term,
        optional_course_count: psu
            .as_ref()
            .map(|u| u.optional_course_count().to_string())
            .unwrap_or_else(|| "—".into()),
        compulsory: course_items(&load_courses(&state.db, &compulsory_ids).await),
        optional: course_items(&load_courses(&state.db, &optional_ids).await),
        optional_pool_url: optional_pool_url_for_psu(&state.db, row.program_structure_unit_id).await,
        error: String::new(),
        form_name: q.form_name(),
        refresh_table: String::new(),
        target_input: String::new(),
        locked_identity: true,
    };
    html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}

pub async fn edit_post(
    Cap(state): Cap<AcademicRecordsState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Path(id): Path<i64>,
    Query(q): Query<ModalFormQuery>,
    HtmlFormBody(form): HtmlFormBody<AcademicRecordForm>,
) -> Response {
    if let Some(resp) = forbid_non_admin(&ctx) {
        return resp;
    }
    let Some(existing) = find_record_scoped(&state.db, id, &ctx).await else {
        return Redirect::to("/academic-records/").into_response();
    };
    let Some(psu) =
        load_program_structure_unit(&state.db, existing.program_structure_unit_id).await
    else {
        return Redirect::to("/academic-records/").into_response();
    };
    let pool = psu_optional_pool_ids(&state.db, psu.id).await;
    let compulsory_ids = load_compulsory_ids(&state.db, id).await;
    let mut page = empty_form(&q, existing.student_id);
    page.id = id;
    page.locked_identity = true;
    page.compulsory = course_items(&load_courses(&state.db, &compulsory_ids).await);
    page.optional = course_items(&load_courses(&state.db, &form.optional_courses).await);
    let compulsory_items = page.compulsory.clone();
    let optional_items = page.optional.clone();
    if let Err(e) = validate_optional(&form.optional_courses, &psu, &pool) {
        fill_form(
            &state.db,
            &mut page,
            &form,
            e,
            compulsory_items.clone(),
            optional_items.clone(),
            psu.optional_course_count().to_string(),
        )
        .await;
        return html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx)).into_response();
    }
    let status = if form.status.trim().is_empty() {
        existing.status.clone()
    } else {
        form.status.clone()
    };
    let date = parse_date(&form.date).or(existing.date);
    let now = Utc::now();
    let model = academic_record::ActiveModel {
        id: Set(existing.id),
        created_at: Set(existing.created_at),
        updated_at: Set(Some(now)),
        deleted_at: Set(existing.deleted_at),
        student_id: Set(existing.student_id),
        program_id: Set(existing.program_id),
        session_id: Set(existing.session_id),
        program_structure_unit_id: Set(existing.program_structure_unit_id),
        date: Set(date),
        status: Set(status),
    };
    match model.update(&state.db).await {
        Ok(_) => {
            let _ = replace_optional(&state.db, id, &form.optional_courses).await;
            respond_edit_modal_done::<AcademicRecordEditModalKey>(
                &htmx,
                &AcademicRecordsDetailRouteTag::new(id).url(),
            )
        }
        Err(e) => {
            fill_form(
                &state.db,
                &mut page,
                &form,
                e.to_string(),
                compulsory_items.clone(),
                optional_items.clone(),
                psu.optional_course_count().to_string(),
            )
            .await;
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
        modal_uid: AcademicRecordDeleteModalKey::ID.to_string(),
        message: "Are you sure you want to delete this academic record?".into(),
        form_name: "academicrecords.AcademicRecordDeleteForm".into(),
        id,
        error: String::new(),
    };
    html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}

pub async fn delete_post(
    Cap(state): Cap<AcademicRecordsState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    Path(id): Path<i64>,
) -> Response {
    if let Some(resp) = forbid_non_admin(&ctx) {
        return resp;
    }
    let Some(existing) = find_record_scoped(&state.db, id, &ctx).await else {
        return Redirect::to("/academic-records/").into_response();
    };
    let now = Utc::now();
    let model = academic_record::ActiveModel {
        id: Set(existing.id),
        deleted_at: Set(Some(now)),
        ..Default::default()
    };
    match model.update(&state.db).await {
        Ok(_) => Redirect::to("/academic-records/").into_response(),
        Err(e) => {
            let page = ConfirmDeletePage {
                modal_uid: AcademicRecordDeleteModalKey::ID.to_string(),
                message: "Are you sure you want to delete this academic record?".into(),
                form_name: "academicrecords.AcademicRecordDeleteForm".into(),
                id,
                error: e.to_string(),
            };
            html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
        }
    }
}

pub async fn select(
    Cap(state): Cap<AcademicRecordsState>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    uri: Uri,
    headers: HeaderMap,
    Query(q): Query<RecordSelectQuery>,
) -> Response {
    if let Some(resp) = forbid_if_no_access(&ctx) {
        return resp;
    }
    let (filter_session, _) = resolve_session_filter(&state.db, &headers).await;
    let records = query_records(&state.db, &ctx, &q.filter, filter_session).await;
    let page = AcademicRecordSelectPage {
        records,
        filter_status: q.filter.status.clone().unwrap_or_default(),
        path_and_query: path_and_query(&uri),
        target_input: q.target_input.unwrap_or_default(),
    };
    respond_picker_select::<AcademicRecordSelectTableKey, AcademicRecordSelectModalKey, _>(
        &htmx, &page,
    )
    .into_response()
}

pub async fn psu_select(
    Cap(state): Cap<AcademicRecordsState>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    uri: Uri,
    Query(q): Query<PsuSelectQuery>,
) -> Response {
    if let Some(resp) = forbid_if_no_access(&ctx) {
        return resp;
    }
    let mut query = PsuEntity::find().filter(program_structure_unit::Column::DeletedAt.is_null());
    if let Some(pid) = q.program_id.filter(|id| *id > 0) {
        query = query.filter(program_structure_unit::Column::ProgramId.eq(pid));
    }
    query = query.order_by_asc(program_structure_unit::Column::TermNumber);
    let page_n = q.page.unwrap_or(1).max(1);
    let paginator = query.paginate(&state.db, PAGE_SIZE as u64);
    let total = paginator.num_items().await.unwrap_or(0);
    let models = paginator
        .fetch_page((page_n as u64).saturating_sub(1))
        .await
        .unwrap_or_default();
    let rows: Vec<PsuRow> = models
        .into_iter()
        .map(|u| PsuRow {
            id: u.id,
            term: term_label(&u),
            optional_count: u.optional_course_count().to_string(),
        })
        .collect();
    let page = PsuSelectPage {
        units: ObjectList::from_page(rows, page_n, PAGE_SIZE, total),
        program_id: q.program_id.unwrap_or(0),
        path_and_query: path_and_query(&uri),
        target_input: q.target_input.unwrap_or_default(),
    };
    respond_picker_select::<PsuSelectTableKey, PsuSelectModalKey, _>(&htmx, &page).into_response()
}

fn md_cell(s: &str) -> String {
    let t = s.replace('|', "\\|").trim().to_string();
    if t.is_empty() {
        "—".into()
    } else {
        t
    }
}

pub async fn download_pdf(
    Cap(state): Cap<AcademicRecordsState>,
    RequireAuth(ctx): RequireAuth,
    Path(id): Path<i64>,
) -> Response {
    if let Some(resp) = forbid_if_no_access(&ctx) {
        return resp;
    }
    let Some(row) = find_record_scoped(&state.db, id, &ctx).await else {
        return Redirect::to("/academic-records/").into_response();
    };
    let bits = display_bits(&state.db, &row).await;
    let compulsory = bits
        .compulsory
        .iter()
        .map(|c| c.name.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    let optional = bits
        .optional
        .iter()
        .map(|c| c.name.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    let issued = ctx.format_datetime(Utc::now()).into_string();
    let md = format!(
        r#"# Academic record

**Issued:** {issued}

## Student

| Field | Value |
|---|---|
| Name | {name} |
| Enrolment no. | {no} |

## Enrolment

| Field | Value |
|---|---|
| Program | {program} |
| Admission session | {session} |
| Term | {term} |
| Status | {status} |
| Admission date | {date} |
| Compulsory courses | {compulsory} |
| Optional courses | {optional} |
"#,
        issued = md_cell(&issued),
        name = md_cell(&bits.student_name),
        no = md_cell(&bits.student_no),
        program = md_cell(&bits.program_display),
        session = md_cell(&bits.session_name),
        term = md_cell(&bits.term),
        status = md_cell(&bits.status),
        date = md_cell(&bits.date),
        compulsory = md_cell(&compulsory),
        optional = md_cell(&optional),
    );
    let base = if bits.student_no.is_empty() {
        format!("academic-record-{id}")
    } else {
        bits.student_no.clone()
    };
    match export_pdf(&md).await {
        Ok(bytes) => file_response(
            "application/pdf",
            &attachment_filename(&format!("{base}-academic-record-{id}"), "pdf"),
            bytes,
        ),
        Err(e) => {
            tracing::warn!(error = %e, id, "PDF export failed; falling back to markdown");
            file_response(
                "text/markdown; charset=utf-8",
                &attachment_filename(&format!("{base}-academic-record-{id}"), "md"),
                md.into_bytes(),
            )
        }
    }
}

fn file_response(content_type: &str, filename: &str, bytes: Vec<u8>) -> Response {
    let mut resp = Response::new(Body::from(bytes));
    *resp.status_mut() = StatusCode::OK;
    resp.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(content_type)
            .unwrap_or(HeaderValue::from_static("application/octet-stream")),
    );
    if let Ok(value) = HeaderValue::from_str(&format!("attachment; filename=\"{filename}\"")) {
        resp.headers_mut().insert(header::CONTENT_DISPOSITION, value);
    }
    resp
}
