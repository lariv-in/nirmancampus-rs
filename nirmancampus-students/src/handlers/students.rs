use axum::{
    extract::{Path, Query},
    http::Uri,
    response::{IntoResponse, Redirect, Response},
};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, sea_query::Expr,
};
use serde::Deserialize;

use crate::{
    entities::{
        student::{self, Entity as StudentEntity},
        student_document::{self, Entity as StudentDocumentEntity},
    },
    forms::StudentForm,
    keys::{
        StudentCreateModalKey, StudentDeleteModalKey, StudentEditModalKey, StudentSelectModalKey,
        StudentSelectTableKey, StudentTableKey,
    },
    routes::StudentsDetailRouteTag,
    state::StudentsState,
    student_detail_related::related_sections_html,
    templates::{
        ConfirmDeletePage, DocumentLink, StudentDetailPage, StudentFormPage, StudentListPage,
        StudentRow, StudentSelectPage,
    },
};
use lariv_rs::{
    components::{ManyToManyItem, ObjectList, SharedChromeFolder, SlotCtx, SwapKey},
    datetime::{format_date, parse_date},
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
use nirmancampus_common::{is_admin, is_student, optional_string, parse_optional_i64, path_and_query, vnode_items, vnode_name_opt};

const PAGE_SIZE: u32 = 20;

#[derive(Debug, Deserialize, Default)]
pub struct StudentListQuery {
    #[serde(default, rename = "Name", alias = "name")]
    pub name: Option<String>,
    #[serde(default, rename = "StudentNo", alias = "student_no")]
    pub student_no: Option<String>,
    #[serde(default, rename = "AadharCard", alias = "aadhar_card")]
    pub aadhar_card: Option<String>,
    #[serde(default, rename = "ABCId", alias = "abc_id")]
    pub abc_id: Option<String>,
    #[serde(default, rename = "Email", alias = "email")]
    pub email: Option<String>,
    #[serde(default, rename = "Phone", alias = "phone")]
    pub phone: Option<String>,
    #[serde(default, rename = "MotherName", alias = "mother_name")]
    pub mother_name: Option<String>,
    #[serde(default, rename = "FatherName", alias = "fathers_name")]
    pub fathers_name: Option<String>,
    #[serde(default, rename = "Category", alias = "category")]
    pub category: Option<String>,
    #[serde(default)]
    pub page: Option<u32>,
    #[serde(default)]
    pub sort: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct StudentSelectQuery {
    #[serde(flatten)]
    pub filter: StudentListQuery,
    #[serde(default)]
    pub target_input: Option<String>,
}

fn scope_students(
    query: sea_orm::Select<StudentEntity>,
    auth: &lariv_rs::plugins::users::state::AuthContext,
) -> sea_orm::Select<StudentEntity> {
    if is_admin(auth) {
        return query;
    }
    if is_student(auth) {
        let email = auth.user.email.trim().to_string();
        if email.is_empty() {
            return query.filter(Expr::cust("1 = 0"));
        }
        return query.filter(student::Column::Email.eq(email));
    }
    query.filter(Expr::cust("1 = 0"))
}

async fn find_student_scoped(
    db: &sea_orm::DatabaseConnection,
    id: i64,
    auth: &lariv_rs::plugins::users::state::AuthContext,
) -> Option<student::Model> {
    let query = StudentEntity::find_by_id(id).filter(student::Column::DeletedAt.is_null());
    let query = scope_students(query, auth);
    lariv_rs::web::opt_or_log(query.one(db).await, "db find one")
}

async fn load_document_ids(db: &sea_orm::DatabaseConnection, student_id: i64) -> Vec<i64> {
    StudentDocumentEntity::find()
        .filter(student_document::Column::StudentId.eq(student_id))
        .all(db)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|r| r.v_node_id)
        .collect()
}

async fn document_items_from_ids(
    db: &sea_orm::DatabaseConnection,
    ids: &[i64],
) -> Vec<ManyToManyItem> {
    vnode_items(db, ids).await
}

async fn load_document_items(db: &sea_orm::DatabaseConnection, student_id: i64) -> Vec<ManyToManyItem> {
    let ids = load_document_ids(db, student_id).await;
    document_items_from_ids(db, &ids).await
}

async fn load_document_links(db: &sea_orm::DatabaseConnection, student_id: i64) -> Vec<DocumentLink> {
    load_document_items(db, student_id)
        .await
        .into_iter()
        .filter_map(|item| {
            item.key.parse::<i64>().ok().map(|id| DocumentLink {
                id,
                name: item.value,
            })
        })
        .collect()
}

async fn sync_student_documents(
    db: &sea_orm::DatabaseConnection,
    student_id: i64,
    file_ids: &[i64],
) -> Result<(), sea_orm::DbErr> {
    StudentDocumentEntity::delete_many()
        .filter(student_document::Column::StudentId.eq(student_id))
        .exec(db)
        .await?;
    for &vnode_id in file_ids {
        student_document::ActiveModel {
            student_id: Set(student_id),
            v_node_id: Set(vnode_id),
        }
        .insert(db)
        .await?;
    }
    Ok(())
}

fn parse_dob(s: &str) -> Result<Option<chrono::NaiveDate>, String> {
    let t = s.trim();
    if t.is_empty() {
        return Ok(None);
    }
    parse_date(t).map(Some).ok_or_else(|| "Invalid date of birth".into())
}

fn photo_id_string(photo_id: Option<i64>) -> String {
    photo_id.filter(|&id| id > 0).map(|id| id.to_string()).unwrap_or_default()
}

async fn query_students(
    db: &sea_orm::DatabaseConnection,
    q: &StudentListQuery,
    auth: &lariv_rs::plugins::users::state::AuthContext,
) -> ObjectList<StudentRow> {
    let mut query = StudentEntity::find().filter(student::Column::DeletedAt.is_null());
    query = scope_students(query, auth);

    if let Some(name) = q.name.as_ref().filter(|n| !n.is_empty()) {
        query = query.filter(student::Column::Name.contains(name));
    }
    if let Some(student_no) = q.student_no.as_ref().filter(|n| !n.is_empty()) {
        query = query.filter(student::Column::StudentNo.contains(student_no));
    }
    if let Some(aadhar) = q.aadhar_card.as_ref().filter(|n| !n.is_empty()) {
        query = query.filter(student::Column::AadharCard.contains(aadhar));
    }
    if let Some(abc_id) = q.abc_id.as_ref().filter(|n| !n.is_empty()) {
        query = query.filter(student::Column::AbcId.contains(abc_id));
    }
    if let Some(email) = q.email.as_ref().filter(|n| !n.is_empty()) {
        query = query.filter(student::Column::Email.contains(email));
    }
    if let Some(phone) = q.phone.as_ref().filter(|n| !n.is_empty()) {
        query = query.filter(student::Column::Phone.contains(phone));
    }
    if let Some(mother) = q.mother_name.as_ref().filter(|n| !n.is_empty()) {
        query = query.filter(student::Column::MotherName.contains(mother));
    }
    if let Some(father) = q.fathers_name.as_ref().filter(|n| !n.is_empty()) {
        query = query.filter(student::Column::FathersName.contains(father));
    }
    if let Some(category) = q.category.as_ref().filter(|n| !n.is_empty()) {
        query = query.filter(student::Column::Category.contains(category));
    }

    let sort = q.sort.as_deref().unwrap_or("").trim();
    query = match sort {
        s if s.eq_ignore_ascii_case("Name DESC") => query.order_by_desc(student::Column::Name),
        s if s.eq_ignore_ascii_case("Name ASC") || s.eq_ignore_ascii_case("Name") => {
            query.order_by_asc(student::Column::Name)
        }
        s if s.eq_ignore_ascii_case("StudentNo DESC") => {
            query.order_by_desc(student::Column::StudentNo)
        }
        s if s.eq_ignore_ascii_case("StudentNo ASC") || s.eq_ignore_ascii_case("StudentNo") => {
            query.order_by_asc(student::Column::StudentNo)
        }
        s if s.eq_ignore_ascii_case("Email DESC") => query.order_by_desc(student::Column::Email),
        s if s.eq_ignore_ascii_case("Email ASC") || s.eq_ignore_ascii_case("Email") => {
            query.order_by_asc(student::Column::Email)
        }
        _ => query.order_by_desc(student::Column::Id),
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
        .map(|s| StudentRow {
            id: s.id,
            name: s.name().to_string(),
            student_no: s.student_no.clone(),
            aadhar_card: s.aadhar_card().to_string(),
            abc_id: s.abc_id().to_string(),
            email: s.email().to_string(),
            phone: s.phone().to_string(),
        })
        .collect();
    ObjectList::from_page(rows, page, PAGE_SIZE, total)
}

fn empty_form_page(q: &ModalFormQuery) -> StudentFormPage {
    StudentFormPage {
        id: 0,
        name: String::new(),
        student_no: String::new(),
        aadhar_card: String::new(),
        abc_id: String::new(),
        email: String::new(),
        phone: String::new(),
        dob: String::new(),
        mother_name: String::new(),
        fathers_name: String::new(),
        category: String::new(),
        address: String::new(),
        remarks: String::new(),
        handicapped: false,
        photo_id: String::new(),
        photo_display: String::new(),
        documents: Vec::new(),
        error: String::new(),
        form_name: q.form_name(),
        refresh_table: q.refresh_table(),
        target_input: q.target_input(),
    }
}

fn form_page_from_form(
    id: i64,
    form: StudentForm,
    photo_display: String,
    documents: Vec<ManyToManyItem>,
    error: String,
    q: &ModalFormQuery,
) -> StudentFormPage {
    StudentFormPage {
        id,
        name: form.name,
        student_no: form.student_no,
        aadhar_card: form.aadhar_card,
        abc_id: form.abc_id,
        email: form.email,
        phone: form.phone,
        dob: form.dob,
        mother_name: form.mother_name,
        fathers_name: form.fathers_name,
        category: form.category,
        address: form.address,
        remarks: form.remarks,
        handicapped: form.handicapped,
        photo_id: form.photo_id,
        photo_display,
        documents,
        error,
        form_name: q.form_name(),
        refresh_table: if id == 0 {
            q.refresh_table()
        } else {
            String::new()
        },
        target_input: if id == 0 {
            q.target_input()
        } else {
            String::new()
        },
    }
}

fn active_from_form(
    form: &StudentForm,
    dob: Option<chrono::NaiveDate>,
    now: chrono::DateTime<Utc>,
) -> student::ActiveModel {
    student::ActiveModel {
        id: Default::default(),
        created_at: Set(Some(now)),
        updated_at: Set(Some(now)),
        deleted_at: Set(None),
        name: Set(Some(form.name.clone())),
        email: Set(optional_string(&form.email)),
        phone: Set(optional_string(&form.phone)),
        student_no: Set(form.student_no.trim().to_string()),
        aadhar_card: Set(optional_string(&form.aadhar_card)),
        abc_id: Set(optional_string(&form.abc_id)),
        dob: Set(dob),
        mother_name: Set(optional_string(&form.mother_name)),
        fathers_name: Set(optional_string(&form.fathers_name)),
        category: Set(optional_string(&form.category)),
        handicapped: Set(Some(form.handicapped)),
        address: Set(optional_string(&form.address)),
        remarks: Set(optional_string(&form.remarks)),
        photo_id: Set(parse_optional_i64(&form.photo_id)),
    }
}

pub async fn list(
    Cap(state): Cap<StudentsState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    uri: Uri,
    Query(q): Query<StudentListQuery>,
) -> maud::Markup {
    let students = query_students(&state.db, &q, &ctx).await;
    let page = StudentListPage {
        students,
        filter_name: q.name.clone().unwrap_or_default(),
        filter_student_no: q.student_no.clone().unwrap_or_default(),
        filter_aadhar_card: q.aadhar_card.clone().unwrap_or_default(),
        filter_abc_id: q.abc_id.clone().unwrap_or_default(),
        filter_email: q.email.clone().unwrap_or_default(),
        filter_phone: q.phone.clone().unwrap_or_default(),
        filter_mother_name: q.mother_name.clone().unwrap_or_default(),
        filter_fathers_name: q.fathers_name.clone().unwrap_or_default(),
        filter_category: q.category.clone().unwrap_or_default(),
        path_and_query: path_and_query(&uri),
        sort: q.sort.clone().unwrap_or_default(),
        is_admin: is_admin(&ctx),
    };
    if htmx.targets::<StudentTableKey>() {
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
    Cap(state): Cap<StudentsState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Path(id): Path<i64>,
) -> Response {
    let Some(student) = find_student_scoped(&state.db, id, &ctx).await else {
        return Redirect::to("/students/").into_response();
    };
    let related_sections = related_sections_html(&state.db, student.id, &ctx).await;
    let photo_name = vnode_name_opt(&state.db, student.photo_id).await;
    let documents = load_document_links(&state.db, student.id).await;
    let page = StudentDetailPage {
        id: student.id,
        name: student.name().to_string(),
        student_no: student.student_no.clone(),
        aadhar_card: student.aadhar_card().to_string(),
        abc_id: student.abc_id().to_string(),
        email: student.email().to_string(),
        phone: student.phone().to_string(),
        dob: student.dob.map(format_date).unwrap_or_default(),
        mother_name: student.mother_name().to_string(),
        fathers_name: student.fathers_name().to_string(),
        category: student.category().to_string(),
        handicapped: student.handicapped(),
        address: student.address().to_string(),
        remarks: student.remarks().to_string(),
        photo_id: student.photo_id,
        photo_name,
        documents,
        is_admin: is_admin(&ctx),
        related_sections,
    };
    html_built_page_or_app_layout(&page, &htmx, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}

pub async fn create_get(
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    Query(q): Query<ModalFormQuery>,
) -> maud::Markup {
    if !is_admin(&ctx) {
        return maud::html! { p { "Forbidden" } };
    }
    html_built_page_with_slots(&empty_form_page(&q), &chrome, &SlotCtx::from_auth(&ctx))
}

pub async fn create_post(
    Cap(state): Cap<StudentsState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Query(q): Query<ModalFormQuery>,
    HtmlFormBody(form): HtmlFormBody<StudentForm>,
) -> Response {
    if !is_admin(&ctx) {
        return Redirect::to("/students/").into_response();
    }
    let dob = match parse_dob(&form.dob) {
        Ok(d) => d,
        Err(e) => {
            let photo_display = vnode_name_opt(&state.db, parse_optional_i64(&form.photo_id)).await;
            let documents = document_items_from_ids(&state.db, &form.documents).await;
            let page = form_page_from_form(0, form, photo_display, documents, e, &q);
            return html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx))
                .into_response();
        }
    };
    let now = Utc::now();
    let model = active_from_form(&form, dob, now);
    match model.insert(&state.db).await {
        Ok(saved) => {
            if let Err(e) = sync_student_documents(&state.db, saved.id, &form.documents).await {
                let photo_display = vnode_name_opt(&state.db, saved.photo_id).await;
                let documents = document_items_from_ids(&state.db, &form.documents).await;
                let page = form_page_from_form(0, form, photo_display, documents, e.to_string(), &q);
                return html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx))
                    .into_response();
            }
            respond_create_modal_done_fk::<StudentCreateModalKey>(
                &htmx,
                &q.refresh_table(),
                &StudentsDetailRouteTag::new(saved.id).url(),
                saved.id,
                saved.name(),
                &q.target_input(),
            )
        }
        Err(e) => {
            let photo_display = vnode_name_opt(&state.db, parse_optional_i64(&form.photo_id)).await;
            let documents = document_items_from_ids(&state.db, &form.documents).await;
            let page = form_page_from_form(0, form, photo_display, documents, e.to_string(), &q);
            html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
        }
    }
}

pub async fn edit_get(
    Cap(state): Cap<StudentsState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    Path(id): Path<i64>,
    Query(q): Query<ModalFormQuery>,
) -> Response {
    if !is_admin(&ctx) {
        return Redirect::to("/students/").into_response();
    }
    let Some(student) = find_student_scoped(&state.db, id, &ctx).await else {
        return Redirect::to("/students/").into_response();
    };
    let photo_display = vnode_name_opt(&state.db, student.photo_id).await;
    let documents = load_document_items(&state.db, student.id).await;
    let page = StudentFormPage {
        id: student.id,
        name: student.name().to_string(),
        student_no: student.student_no.clone(),
        aadhar_card: student.aadhar_card().to_string(),
        abc_id: student.abc_id().to_string(),
        email: student.email().to_string(),
        phone: student.phone().to_string(),
        dob: student.dob.map(format_date).unwrap_or_default(),
        mother_name: student.mother_name().to_string(),
        fathers_name: student.fathers_name().to_string(),
        category: student.category().to_string(),
        address: student.address().to_string(),
        remarks: student.remarks().to_string(),
        handicapped: student.handicapped(),
        photo_id: photo_id_string(student.photo_id),
        photo_display,
        documents,
        error: String::new(),
        form_name: q.form_name(),
        refresh_table: String::new(),
        target_input: String::new(),
    };
    html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}

pub async fn edit_post(
    Cap(state): Cap<StudentsState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Path(id): Path<i64>,
    Query(q): Query<ModalFormQuery>,
    HtmlFormBody(form): HtmlFormBody<StudentForm>,
) -> Response {
    if !is_admin(&ctx) {
        return Redirect::to("/students/").into_response();
    }
    let Some(existing) = find_student_scoped(&state.db, id, &ctx).await else {
        return Redirect::to("/students/").into_response();
    };
    let dob = match parse_dob(&form.dob) {
        Ok(d) => d,
        Err(e) => {
            let photo_display = vnode_name_opt(&state.db, parse_optional_i64(&form.photo_id)).await;
            let documents = document_items_from_ids(&state.db, &form.documents).await;
            let page = form_page_from_form(id, form, photo_display, documents, e, &q);
            return html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx))
                .into_response();
        }
    };
    let now = Utc::now();
    let mut model = active_from_form(&form, dob, now);
    model.id = Set(existing.id);
    model.created_at = Set(existing.created_at);
    model.deleted_at = Set(existing.deleted_at);
    match model.update(&state.db).await {
        Ok(_) => {
            if let Err(e) = sync_student_documents(&state.db, id, &form.documents).await {
                let photo_display = vnode_name_opt(&state.db, parse_optional_i64(&form.photo_id)).await;
                let documents = document_items_from_ids(&state.db, &form.documents).await;
                let page = form_page_from_form(id, form, photo_display, documents, e.to_string(), &q);
                return html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx))
                    .into_response();
            }
            respond_edit_modal_done::<StudentEditModalKey>(
                &htmx,
                &StudentsDetailRouteTag::new(id).url(),
            )
        }
        Err(e) => {
            let photo_display = vnode_name_opt(&state.db, parse_optional_i64(&form.photo_id)).await;
            let documents = document_items_from_ids(&state.db, &form.documents).await;
            let page = form_page_from_form(id, form, photo_display, documents, e.to_string(), &q);
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
        modal_uid: StudentDeleteModalKey::ID.to_string(),
        message: format!("Delete student #{id}?"),
        form_name: "students.StudentDeleteForm".into(),
        id,
        error: String::new(),
    };
    html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx))
}

pub async fn delete_post(
    Cap(state): Cap<StudentsState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    Path(id): Path<i64>,
) -> Response {
    if !is_admin(&ctx) {
        return Redirect::to("/students/").into_response();
    }
    let Some(existing) = find_student_scoped(&state.db, id, &ctx).await else {
        return Redirect::to("/students/").into_response();
    };
    let now = Utc::now();
    let model = student::ActiveModel {
        id: Set(existing.id),
        deleted_at: Set(Some(now)),
        ..Default::default()
    };
    match model.update(&state.db).await {
        Ok(_) => Redirect::to("/students/").into_response(),
        Err(e) => {
            tracing::error!(error = %e, id, "failed to delete student");
            let page = ConfirmDeletePage {
                modal_uid: StudentDeleteModalKey::ID.to_string(),
                message: format!("Delete student #{id}?"),
                form_name: "students.StudentDeleteForm".into(),
                id,
                error: e.to_string(),
            };
            html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
        }
    }
}

pub async fn select(
    Cap(state): Cap<StudentsState>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    uri: Uri,
    Query(q): Query<StudentSelectQuery>,
) -> maud::Markup {
    let students = query_students(&state.db, &q.filter, &ctx).await;
    let page = StudentSelectPage {
        students,
        filter_name: q.filter.name.clone().unwrap_or_default(),
        filter_student_no: q.filter.student_no.clone().unwrap_or_default(),
        filter_phone: q.filter.phone.clone().unwrap_or_default(),
        path_and_query: path_and_query(&uri),
        sort: q.filter.sort.clone().unwrap_or_default(),
        target_input: q.target_input.unwrap_or_default(),
    };
    respond_picker_select::<StudentSelectTableKey, StudentSelectModalKey, _>(&htmx, &page)
}
