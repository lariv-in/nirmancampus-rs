use axum::{
    extract::{Multipart, Path, Query},
    http::Uri,
    response::{IntoResponse, Redirect, Response},
};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, Condition, EntityTrait, PaginatorTrait,
    QueryFilter, QueryOrder,
};
use serde::Deserialize;

use crate::{
    entities::fee::{self, Entity as FeeEntity},
    forms::{FeeForm, FeeUploadForm},
    keys::{FeeCreateModalKey, FeeDeleteModalKey, FeeEditModalKey, FeeTableKey},
    parse::{flag_bool, flag_from_bool, format_dod_form, opt_str, parse_dod, parse_optional_text},
    tblfee_xlsx::{parse_tblfee_xlsx, upsert_rows},
    routes::StudentFeesDetailRouteTag,
    state::StudentFeesState,
    templates::{ConfirmDeletePage, FeeDetailPage, FeeFormPage, FeeListPage, FeeRow},
};
use lariv_rs::{
    components::{ObjectList, SharedChromeFolder, SlotCtx, SwapKey},
    html_form::{HtmlForm, HtmlFormBody},
    http::Cap,
    plugins::users::middleware::RequireAuth,
    web::{
        Htmx, ModalFormQuery, html_built_page_or_app_layout, html_built_page_with_slots,
        respond_create_modal_done_fk, respond_edit_modal_done,
    },
};
use nirmancampus_common::{is_admin, path_and_query};

use super::forbid_non_admin;

const PAGE_SIZE: u32 = 20;
const MAX_UPLOAD_BYTES: usize = lariv_rs::http::REQUEST_BODY_LIMIT_BYTES;

#[derive(Debug, Deserialize, Default)]
pub struct FeeListQuery {
    #[serde(default, rename = "Search", alias = "search")]
    pub search: Option<String>,
    #[serde(default)]
    pub page: Option<u32>,
}

fn search_condition(q: &str) -> Condition {
    let mut cond = Condition::any()
        .add(fee::Column::AdmSession.contains(q))
        .add(fee::Column::AdmYear.contains(q))
        .add(fee::Column::Submit.contains(q))
        .add(fee::Column::Prog.contains(q))
        .add(fee::Column::Enroll.contains(q))
        .add(fee::Column::Student.contains(q))
        .add(fee::Column::YearSem.contains(q))
        .add(fee::Column::Category.contains(q))
        .add(fee::Column::Dob.contains(q))
        .add(fee::Column::Contact.contains(q))
        .add(fee::Column::Deposit.contains(q))
        .add(fee::Column::Nsd.contains(q))
        .add(fee::Column::Fee.contains(q))
        .add(fee::Column::Courses.contains(q))
        .add(fee::Column::Remarks.contains(q))
        .add(fee::Column::DepositBy.contains(q))
        .add(fee::Column::Ts.contains(q))
        .add(fee::Column::Medium.contains(q))
        .add(fee::Column::MotherName.contains(q))
        .add(fee::Column::FatherName.contains(q))
        .add(fee::Column::Username.contains(q))
        .add(fee::Column::ControlId.contains(q))
        .add(fee::Column::Descrepency.contains(q))
        .add(fee::Column::University.contains(q))
        .add(fee::Column::PaymentMode.contains(q))
        .add(fee::Column::TransId.contains(q))
        .add(fee::Column::Bank.contains(q))
        .add(fee::Column::Rm.contains(q));
    if let Ok(id) = q.parse::<i32>() {
        cond = cond.add(fee::Column::Id.eq(id));
    }
    let flag = match q.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" => Some(1i8),
        "0" | "false" | "no" => Some(0i8),
        _ => None,
    };
    if let Some(v) = flag {
        cond = cond
            .add(fee::Column::IsReconciled.eq(v))
            .add(fee::Column::OnlineExported.eq(v));
    }
    cond
}

fn empty_list() -> ObjectList<FeeRow> {
    ObjectList::from_page(Vec::new(), 1, PAGE_SIZE, 0)
}

async fn query_rows(db: &sea_orm::DatabaseConnection, q: &FeeListQuery) -> ObjectList<FeeRow> {
    let mut query = FeeEntity::find();
    if let Some(search) = q
        .search
        .as_ref()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
    {
        query = query.filter(search_condition(&search));
    }
    query = query.order_by_desc(fee::Column::Id);
    let page = q.page.unwrap_or(1).max(1);
    let paginator = query.paginate(db, PAGE_SIZE as u64);
    let total = paginator.num_items().await.unwrap_or(0);
    let models = paginator
        .fetch_page((page as u64).saturating_sub(1))
        .await
        .unwrap_or_default();
    let rows = models.iter().map(FeeRow::from_model).collect();
    ObjectList::from_page(rows, page, PAGE_SIZE, total)
}

fn list_page(
    records: ObjectList<FeeRow>,
    q: &FeeListQuery,
    uri: &Uri,
    is_admin: bool,
    connection_error: String,
    sync_message: String,
    sync_error: String,
) -> FeeListPage {
    FeeListPage {
        records,
        filter_search: q.search.clone().unwrap_or_default(),
        path_and_query: path_and_query(uri),
        is_admin,
        connection_error,
        sync_message,
        sync_error,
    }
}

async fn find_row(db: &sea_orm::DatabaseConnection, id: i64) -> Option<fee::Model> {
    let id = i32::try_from(id).ok().filter(|n| *n > 0)?;
    lariv_rs::web::opt_or_log(FeeEntity::find_by_id(id).one(db).await, "db find tblfee")
}

fn receipt_id(raw: i64) -> Result<i32, String> {
    let id = i32::try_from(raw).map_err(|_| "Receipt ID is out of range".to_string())?;
    if id <= 0 {
        return Err("Receipt ID is required".into());
    }
    Ok(id)
}

fn form_to_active(id: i32, form: &FeeForm) -> fee::ActiveModel {
    fee::ActiveModel {
        id: Set(id),
        adm_session: Set(parse_optional_text(&form.adm_session)),
        adm_year: Set(parse_optional_text(&form.adm_year)),
        dod: Set(parse_dod(&form.dod)),
        submit: Set(parse_optional_text(&form.submit)),
        prog: Set(parse_optional_text(&form.prog)),
        enroll: Set(parse_optional_text(&form.enroll)),
        student: Set(parse_optional_text(&form.student)),
        year_sem: Set(parse_optional_text(&form.year_sem)),
        category: Set(parse_optional_text(&form.category)),
        dob: Set(parse_optional_text(&form.dob)),
        contact: Set(parse_optional_text(&form.contact)),
        deposit: Set(parse_optional_text(&form.deposit)),
        nsd: Set(parse_optional_text(&form.nsd)),
        fee: Set(parse_optional_text(&form.fee)),
        courses: Set(parse_optional_text(&form.courses)),
        remarks: Set(parse_optional_text(&form.remarks)),
        deposit_by: Set(parse_optional_text(&form.deposit_by)),
        ts: Set(parse_optional_text(&form.ts)),
        medium: Set(parse_optional_text(&form.medium)),
        mother_name: Set(parse_optional_text(&form.mother_name)),
        father_name: Set(parse_optional_text(&form.father_name)),
        username: Set(parse_optional_text(&form.username)),
        control_id: Set(parse_optional_text(&form.control_id)),
        descrepency: Set(parse_optional_text(&form.descrepency)),
        university: Set(parse_optional_text(&form.university)),
        payment_mode: Set(parse_optional_text(&form.payment_mode)),
        trans_id: Set(parse_optional_text(&form.trans_id)),
        bank: Set(parse_optional_text(&form.bank)),
        rm: Set(parse_optional_text(&form.rm)),
        is_reconciled: Set(flag_from_bool(form.is_reconciled)),
        online_exported: Set(flag_from_bool(form.online_exported)),
    }
}

fn model_to_form(row: &fee::Model) -> FeeForm {
    FeeForm {
        id: i64::from(row.id),
        adm_session: opt_str(&row.adm_session).to_string(),
        adm_year: opt_str(&row.adm_year).to_string(),
        dod: format_dod_form(row.dod),
        submit: opt_str(&row.submit).to_string(),
        prog: opt_str(&row.prog).to_string(),
        enroll: opt_str(&row.enroll).to_string(),
        student: opt_str(&row.student).to_string(),
        year_sem: opt_str(&row.year_sem).to_string(),
        category: opt_str(&row.category).to_string(),
        dob: opt_str(&row.dob).to_string(),
        contact: opt_str(&row.contact).to_string(),
        deposit: opt_str(&row.deposit).to_string(),
        nsd: opt_str(&row.nsd).to_string(),
        fee: opt_str(&row.fee).to_string(),
        courses: opt_str(&row.courses).to_string(),
        remarks: opt_str(&row.remarks).to_string(),
        deposit_by: opt_str(&row.deposit_by).to_string(),
        ts: opt_str(&row.ts).to_string(),
        medium: opt_str(&row.medium).to_string(),
        mother_name: opt_str(&row.mother_name).to_string(),
        father_name: opt_str(&row.father_name).to_string(),
        username: opt_str(&row.username).to_string(),
        control_id: opt_str(&row.control_id).to_string(),
        descrepency: opt_str(&row.descrepency).to_string(),
        university: opt_str(&row.university).to_string(),
        payment_mode: opt_str(&row.payment_mode).to_string(),
        trans_id: opt_str(&row.trans_id).to_string(),
        bank: opt_str(&row.bank).to_string(),
        rm: opt_str(&row.rm).to_string(),
        is_reconciled: flag_bool(row.is_reconciled),
        online_exported: flag_bool(row.online_exported),
    }
}

fn form_page(
    id: i64,
    form: FeeForm,
    error: String,
    q: &ModalFormQuery,
    refresh: String,
) -> FeeFormPage {
    FeeFormPage {
        id,
        form,
        error,
        form_name: q.form_name(),
        refresh_table: refresh,
        target_input: q.target_input(),
    }
}

pub async fn list(
    Cap(state): Cap<StudentFeesState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    uri: Uri,
    Query(q): Query<FeeListQuery>,
) -> Response {
    if let Some(resp) = forbid_non_admin(&ctx) {
        return resp;
    }
    let (records, connection_error) = match state.mysql().await {
        Ok(db) => (query_rows(&db, &q).await, String::new()),
        Err(e) => (empty_list(), e.to_string()),
    };
    let page = list_page(
        records,
        &q,
        &uri,
        is_admin(&ctx),
        connection_error,
        String::new(),
        String::new(),
    );
    if htmx.targets::<FeeTableKey>() {
        return page.render_table().into_response();
    }
    html_built_page_or_app_layout(&page, &htmx, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}

pub async fn sync(
    Cap(state): Cap<StudentFeesState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    uri: Uri,
    Query(q): Query<FeeListQuery>,
    multipart: Multipart,
) -> Response {
    if let Some(resp) = forbid_non_admin(&ctx) {
        return resp;
    }
    let mut sync_message = String::new();
    let mut sync_error = String::new();
    match FeeUploadForm::from_multipart(multipart).await {
        Ok(form) => match form.file.into_bytes().await {
            Ok(bytes) => {
                if bytes.len() > MAX_UPLOAD_BYTES {
                    sync_error = "xlsx file too large (max 50 MiB)".into();
                } else if bytes.is_empty() {
                    sync_error = "empty file".into();
                } else {
                    match parse_tblfee_xlsx(&bytes) {
                        Ok((rows, skipped)) => match state.mysql().await {
                            Ok(db) => match upsert_rows(&db, &rows).await {
                                Ok(report) => {
                                    sync_message = format!(
                                        "Imported {} rows ({} new, {} updated, {} skipped without Receipt ID).",
                                        rows.len(),
                                        report.inserted,
                                        report.updated,
                                        skipped + report.skipped
                                    );
                                }
                                Err(e) => sync_error = e,
                            },
                            Err(e) => sync_error = e.to_string(),
                        },
                        Err(e) => sync_error = e,
                    }
                }
            }
            Err(e) => sync_error = e.to_string(),
        },
        Err(e) => sync_error = e.to_string(),
    }
    let (records, connection_error) = match state.mysql().await {
        Ok(db) => (query_rows(&db, &q).await, String::new()),
        Err(e) => (empty_list(), e.to_string()),
    };
    let page = list_page(
        records,
        &q,
        &uri,
        is_admin(&ctx),
        connection_error,
        sync_message,
        sync_error,
    );
    html_built_page_or_app_layout(&page, &htmx, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}

pub async fn detail(
    Cap(state): Cap<StudentFeesState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Path(id): Path<i64>,
) -> Response {
    if let Some(resp) = forbid_non_admin(&ctx) {
        return resp;
    }
    let Ok(db) = state.mysql().await else {
        return Redirect::to("/student-fees/").into_response();
    };
    let Some(row) = find_row(&db, id).await else {
        return Redirect::to("/student-fees/").into_response();
    };
    let page = FeeDetailPage::from_model(row, is_admin(&ctx));
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
    let page = form_page(0, FeeForm::default(), String::new(), &q, q.refresh_table());
    html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}

pub async fn create_post(
    Cap(state): Cap<StudentFeesState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Query(q): Query<ModalFormQuery>,
    HtmlFormBody(form): HtmlFormBody<FeeForm>,
) -> Response {
    if let Some(resp) = forbid_non_admin(&ctx) {
        return resp;
    }
    let id = match receipt_id(form.id) {
        Ok(id) => id,
        Err(e) => {
            let page = form_page(0, form, e, &q, q.refresh_table());
            return html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx))
                .into_response();
        }
    };
    let db = match state.mysql().await {
        Ok(db) => db,
        Err(e) => {
            let page = form_page(0, form, e.to_string(), &q, q.refresh_table());
            return html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx))
                .into_response();
        }
    };
    if find_row(&db, i64::from(id)).await.is_some() {
        let page = form_page(
            0,
            form,
            "Receipt ID already exists".into(),
            &q,
            q.refresh_table(),
        );
        return html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx)).into_response();
    }
    let model = form_to_active(id, &form);
    match model.insert(&db).await {
        Ok(saved) => {
            let label = if opt_str(&saved.student).is_empty() {
                format!("Receipt {}", saved.id)
            } else {
                opt_str(&saved.student).to_string()
            };
            respond_create_modal_done_fk::<FeeCreateModalKey>(
                &htmx,
                &q.refresh_table(),
                &StudentFeesDetailRouteTag::new(i64::from(saved.id)).url(),
                i64::from(saved.id),
                &label,
                &q.target_input(),
            )
        }
        Err(e) => {
            let page = form_page(0, form, e.to_string(), &q, q.refresh_table());
            html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
        }
    }
}

pub async fn edit_get(
    Cap(state): Cap<StudentFeesState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    Path(id): Path<i64>,
    Query(q): Query<ModalFormQuery>,
) -> Response {
    if let Some(resp) = forbid_non_admin(&ctx) {
        return resp;
    }
    let Ok(db) = state.mysql().await else {
        return Redirect::to("/student-fees/").into_response();
    };
    let Some(row) = find_row(&db, id).await else {
        return Redirect::to("/student-fees/").into_response();
    };
    let page = form_page(id, model_to_form(&row), String::new(), &q, String::new());
    html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}

pub async fn edit_post(
    Cap(state): Cap<StudentFeesState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Path(id): Path<i64>,
    Query(q): Query<ModalFormQuery>,
    HtmlFormBody(form): HtmlFormBody<FeeForm>,
) -> Response {
    if let Some(resp) = forbid_non_admin(&ctx) {
        return resp;
    }
    let db = match state.mysql().await {
        Ok(db) => db,
        Err(e) => {
            let page = form_page(id, form, e.to_string(), &q, String::new());
            return html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx))
                .into_response();
        }
    };
    if find_row(&db, id).await.is_none() {
        return Redirect::to("/student-fees/").into_response();
    }
    let Some(pk) = i32::try_from(id).ok().filter(|n| *n > 0) else {
        return Redirect::to("/student-fees/").into_response();
    };
    let model = form_to_active(pk, &form);
    match model.update(&db).await {
        Ok(_) => respond_edit_modal_done::<FeeEditModalKey>(
            &htmx,
            &StudentFeesDetailRouteTag::new(id).url(),
        ),
        Err(e) => {
            let page = form_page(id, form, e.to_string(), &q, String::new());
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
        modal_uid: FeeDeleteModalKey::ID.to_string(),
        message: format!("Delete fee receipt #{id}? This cannot be undone."),
        form_name: "studentfees.FeeDeleteForm".into(),
        id,
        error: String::new(),
    };
    html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}

pub async fn delete_post(
    Cap(state): Cap<StudentFeesState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    Path(id): Path<i64>,
) -> Response {
    if let Some(resp) = forbid_non_admin(&ctx) {
        return resp;
    }
    let db = match state.mysql().await {
        Ok(db) => db,
        Err(e) => {
            let page = ConfirmDeletePage {
                modal_uid: FeeDeleteModalKey::ID.to_string(),
                message: format!("Delete fee receipt #{id}? This cannot be undone."),
                form_name: "studentfees.FeeDeleteForm".into(),
                id,
                error: e.to_string(),
            };
            return html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx))
                .into_response();
        }
    };
    let Some(pk) = i32::try_from(id).ok().filter(|n| *n > 0) else {
        return Redirect::to("/student-fees/").into_response();
    };
    match FeeEntity::delete_by_id(pk).exec(&db).await {
        Ok(_) => Redirect::to("/student-fees/").into_response(),
        Err(e) => {
            let page = ConfirmDeletePage {
                modal_uid: FeeDeleteModalKey::ID.to_string(),
                message: format!("Delete fee receipt #{id}? This cannot be undone."),
                form_name: "studentfees.FeeDeleteForm".into(),
                id,
                error: e.to_string(),
            };
            html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
        }
    }
}
