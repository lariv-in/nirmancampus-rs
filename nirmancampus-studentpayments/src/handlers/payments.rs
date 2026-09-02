use axum::{
    extract::{Path, Query},
    http::Uri,
    response::{IntoResponse, Redirect, Response},
};
use chrono::{NaiveDate, Utc};
use rust_decimal::Decimal;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, sea_query::Expr,
};
use serde::Deserialize;

use crate::{
    entities::payment::{self, Entity as PaymentEntity},
    forms::PaymentForm,
    keys::{
        PaymentCreateModalKey, PaymentDeleteModalKey, PaymentEditModalKey, PaymentSelectModalKey,
        PaymentSelectTableKey, PaymentTableKey,
    },
    routes::StudentPaymentsDetailRouteTag,
    state::StudentPaymentsState,
    templates::{
        ConfirmDeletePage, PaymentDetailPage, PaymentFormPage, PaymentListPage, PaymentRow,
        PaymentSelectPage,
    },
};
use lariv_rs::{
    components::{ObjectList, SharedChromeFolder, SlotCtx, SwapKey},
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
    can_view_campus_records, is_admin, is_student, optional_string, path_and_query,
    payment_method_choice_pairs,
};
use nirmancampus_students::entities::student::{self, Entity as StudentEntity};

const PAGE_SIZE: u32 = 20;

#[derive(Debug, Deserialize, Default)]
pub struct PaymentListQuery {
    #[serde(default, rename = "PaymentMethod", alias = "payment_method")]
    pub payment_method: Option<String>,
    #[serde(default, rename = "TransactionID", alias = "transaction_id")]
    pub transaction_id: Option<String>,
    #[serde(default, rename = "StudentID", alias = "student_id")]
    pub student_id: Option<i64>,
    #[serde(default)]
    pub page: Option<u32>,
    #[serde(default)]
    pub sort: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct PaymentSelectQuery {
    #[serde(flatten)]
    pub filter: PaymentListQuery,
    #[serde(default)]
    pub target_input: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct PaymentCreateQuery {
    #[serde(flatten)]
    pub modal: ModalFormQuery,
    #[serde(default, rename = "StudentID", alias = "student_id")]
    pub student_id: Option<i64>,
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

fn scope_payments(
    query: sea_orm::Select<PaymentEntity>,
    auth: &lariv_rs::plugins::users::state::AuthContext,
) -> sea_orm::Select<PaymentEntity> {
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

pub fn payment_method_label(key: &str) -> String {
    payment_method_choice_pairs()
        .into_iter()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v)
        .unwrap_or_else(|| key.to_string())
}

pub fn student_label(s: &student::Model) -> String {
    let name = s.name();
    if name.is_empty() {
        s.student_no.clone()
    } else {
        format!("{name} ({})", s.student_no)
    }
}

pub async fn load_student(
    db: &sea_orm::DatabaseConnection,
    id: i64,
) -> Option<student::Model> {
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

async fn find_payment(db: &sea_orm::DatabaseConnection, id: i64) -> Option<payment::Model> {
    lariv_rs::web::opt_or_log(
        PaymentEntity::find_by_id(id)
            .filter(payment::Column::DeletedAt.is_null())
            .one(db)
            .await,
        "db find one",
    )
}

pub async fn find_payment_scoped(
    db: &sea_orm::DatabaseConnection,
    id: i64,
    auth: &lariv_rs::plugins::users::state::AuthContext,
) -> Option<payment::Model> {
    let query = PaymentEntity::find_by_id(id).filter(payment::Column::DeletedAt.is_null());
    let query = scope_payments(query, auth);
    lariv_rs::web::opt_or_log(query.one(db).await, "db find one")
}

pub fn format_amount(amount: Decimal) -> String {
    format!("₹ {amount:.2}")
}

fn parse_amount(raw: &str) -> Result<Decimal, String> {
    let t = raw.trim();
    if t.is_empty() {
        return Err("Amount is required".into());
    }
    t.parse::<Decimal>()
        .map_err(|_| "Amount must be a number".into())
}

async fn query_payments(
    db: &sea_orm::DatabaseConnection,
    q: &PaymentListQuery,
    auth: &lariv_rs::plugins::users::state::AuthContext,
) -> ObjectList<PaymentRow> {
    let mut query = PaymentEntity::find().filter(payment::Column::DeletedAt.is_null());
    query = scope_payments(query, auth);
    if let Some(method) = q.payment_method.as_ref().filter(|n| !n.is_empty()) {
        query = query.filter(payment::Column::PaymentMethod.eq(method.clone()));
    }
    if let Some(tid) = q.transaction_id.as_ref().filter(|n| !n.is_empty()) {
        query = query.filter(payment::Column::TransactionId.contains(tid));
    }
    if let Some(sid) = q.student_id.filter(|id| *id > 0) {
        query = query.filter(payment::Column::StudentId.eq(sid));
    }
    let sort = q.sort.as_deref().unwrap_or("").trim();
    query = match sort {
        s if s.eq_ignore_ascii_case("Amount DESC") => {
            query.order_by_desc(payment::Column::Amount)
        }
        s if s.eq_ignore_ascii_case("Amount ASC") || s.eq_ignore_ascii_case("Amount") => {
            query.order_by_asc(payment::Column::Amount)
        }
        _ => query.order_by_desc(payment::Column::Id),
    };
    let page = q.page.unwrap_or(1).max(1);
    let paginator = query.paginate(db, PAGE_SIZE as u64);
    let total = paginator.num_items().await.unwrap_or(0);
    let models = paginator
        .fetch_page((page as u64).saturating_sub(1))
        .await
        .unwrap_or_default();
    let mut rows = Vec::with_capacity(models.len());
    for p in models {
        let student = load_student(db, p.student_id).await;
        let student_display = student
            .as_ref()
            .map(student_label)
            .unwrap_or_else(|| format!("Student #{}", p.student_id));
        rows.push(PaymentRow {
            id: p.id,
            student_id: p.student_id,
            student_display,
            amount: format_amount(p.amount),
            payment_method: payment_method_label(&p.payment_method),
            transaction_id: p.transaction_id().to_string(),
            paid_at: p.paid_at.map(format_date).unwrap_or_default(),
        });
    }
    ObjectList::from_page(rows, page, PAGE_SIZE, total)
}

fn empty_form(q: &PaymentCreateQuery) -> PaymentFormPage {
    PaymentFormPage {
        id: 0,
        student_id: q.student_id.unwrap_or(0),
        student_display: String::new(),
        amount: String::new(),
        payment_method: "cash".into(),
        transaction_id: String::new(),
        paid_at: String::new(),
        remarks: String::new(),
        error: String::new(),
        form_name: q.modal.form_name(),
        refresh_table: q.modal.refresh_table(),
        target_input: q.modal.target_input(),
    }
}

pub async fn list(
    Cap(state): Cap<StudentPaymentsState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    uri: Uri,
    Query(q): Query<PaymentListQuery>,
) -> Response {
    if let Some(resp) = forbid_if_no_access(&ctx) {
        return resp;
    }
    let payments = query_payments(&state.db, &q, &ctx).await;
    let student_display = if let Some(sid) = q.student_id.filter(|id| *id > 0) {
        load_student(&state.db, sid)
            .await
            .map(|s| student_label(&s))
            .unwrap_or_default()
    } else {
        String::new()
    };
    let page = PaymentListPage {
        payments,
        filter_payment_method: q.payment_method.clone().unwrap_or_default(),
        filter_transaction_id: q.transaction_id.clone().unwrap_or_default(),
        filter_student_id: q.student_id.unwrap_or(0),
        filter_student_display: student_display,
        path_and_query: path_and_query(&uri),
        sort: q.sort.clone().unwrap_or_default(),
        is_admin: is_admin(&ctx),
    };
    if htmx.targets::<PaymentTableKey>() {
        return page.render_table().into_response();
    }
    html_built_page_or_app_layout(&page, &htmx, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}

pub async fn detail(
    Cap(state): Cap<StudentPaymentsState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Path(id): Path<i64>,
) -> Response {
    if let Some(resp) = forbid_if_no_access(&ctx) {
        return resp;
    }
    let Some(row) = find_payment_scoped(&state.db, id, &ctx).await else {
        return Redirect::to("/student-payments/").into_response();
    };
    let student = load_student(&state.db, row.student_id).await;
    let student_display = student
        .as_ref()
        .map(student_label)
        .unwrap_or_else(|| format!("Student #{}", row.student_id));
    let page = PaymentDetailPage {
        id: row.id,
        student_id: row.student_id,
        student_display,
        amount: format_amount(row.amount),
        payment_method: payment_method_label(&row.payment_method),
        transaction_id: row.transaction_id().to_string(),
        paid_at: row.paid_at.map(format_date).unwrap_or_default(),
        remarks: row.remarks().to_string(),
        is_admin: is_admin(&ctx),
    };
    html_built_page_or_app_layout(&page, &htmx, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}

pub async fn create_get(
    Cap(state): Cap<StudentPaymentsState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    Query(q): Query<PaymentCreateQuery>,
) -> Response {
    if let Some(resp) = forbid_non_admin(&ctx) {
        return resp;
    }
    let mut page = empty_form(&q);
    if page.student_id > 0 {
        if let Some(s) = load_student(&state.db, page.student_id).await {
            page.student_display = student_label(&s);
        }
    }
    html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}

fn form_to_page(
    id: i64,
    form: &PaymentForm,
    student_display: String,
    error: String,
    q: &ModalFormQuery,
    refresh: String,
) -> PaymentFormPage {
    PaymentFormPage {
        id,
        student_id: form.student_id,
        student_display,
        amount: form.amount.clone(),
        payment_method: form.payment_method.clone(),
        transaction_id: form.transaction_id.clone(),
        paid_at: form.paid_at.clone(),
        remarks: form.remarks.clone(),
        error,
        form_name: q.form_name(),
        refresh_table: refresh,
        target_input: q.target_input(),
    }
}

pub async fn create_post(
    Cap(state): Cap<StudentPaymentsState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Query(q): Query<ModalFormQuery>,
    HtmlFormBody(form): HtmlFormBody<PaymentForm>,
) -> Response {
    if let Some(resp) = forbid_non_admin(&ctx) {
        return resp;
    }
    let student_display = load_student(&state.db, form.student_id)
        .await
        .map(|s| student_label(&s))
        .unwrap_or_default();
    let amount = match parse_amount(&form.amount) {
        Ok(v) => v,
        Err(e) => {
            let page = form_to_page(0, &form, student_display, e, &q, q.refresh_table());
            return html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx))
                .into_response();
        }
    };
    let now = Utc::now();
    let paid_at: Option<NaiveDate> = parse_date(&form.paid_at);
    let method = if form.payment_method.trim().is_empty() {
        "cash".to_string()
    } else {
        form.payment_method.clone()
    };
    let model = payment::ActiveModel {
        id: Default::default(),
        created_at: Set(Some(now)),
        updated_at: Set(Some(now)),
        deleted_at: Set(None),
        student_id: Set(form.student_id),
        amount: Set(amount),
        payment_method: Set(method),
        remarks: Set(optional_string(&form.remarks)),
        transaction_id: Set(optional_string(&form.transaction_id).or(Some(String::new()))),
        paid_at: Set(paid_at),
    };
    match model.insert(&state.db).await {
        Ok(saved) => respond_create_modal_done_fk::<PaymentCreateModalKey>(
            &htmx,
            &q.refresh_table(),
            &StudentPaymentsDetailRouteTag::new(saved.id).url(),
            saved.id,
            &format_amount(saved.amount),
            &q.target_input(),
        ),
        Err(e) => {
            let page = form_to_page(0, &form, student_display, e.to_string(), &q, q.refresh_table());
            html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
        }
    }
}

pub async fn edit_get(
    Cap(state): Cap<StudentPaymentsState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    Path(id): Path<i64>,
    Query(q): Query<ModalFormQuery>,
) -> Response {
    if let Some(resp) = forbid_non_admin(&ctx) {
        return resp;
    }
    let Some(row) = find_payment(&state.db, id).await else {
        return Redirect::to("/student-payments/").into_response();
    };
    let student_display = load_student(&state.db, row.student_id)
        .await
        .map(|s| student_label(&s))
        .unwrap_or_default();
    let page = PaymentFormPage {
        id: row.id,
        student_id: row.student_id,
        student_display,
        amount: format!("{:.2}", row.amount),
        payment_method: row.payment_method().to_string(),
        transaction_id: row.transaction_id().to_string(),
        paid_at: row.paid_at.map(format_date).unwrap_or_default(),
        remarks: row.remarks().to_string(),
        error: String::new(),
        form_name: q.form_name(),
        refresh_table: String::new(),
        target_input: String::new(),
    };
    html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}

pub async fn edit_post(
    Cap(state): Cap<StudentPaymentsState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Path(id): Path<i64>,
    Query(q): Query<ModalFormQuery>,
    HtmlFormBody(form): HtmlFormBody<PaymentForm>,
) -> Response {
    if let Some(resp) = forbid_non_admin(&ctx) {
        return resp;
    }
    let Some(existing) = find_payment(&state.db, id).await else {
        return Redirect::to("/student-payments/").into_response();
    };
    let student_display = load_student(&state.db, form.student_id)
        .await
        .map(|s| student_label(&s))
        .unwrap_or_default();
    let amount = match parse_amount(&form.amount) {
        Ok(v) => v,
        Err(e) => {
            let page = form_to_page(id, &form, student_display, e, &q, String::new());
            return html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx))
                .into_response();
        }
    };
    let now = Utc::now();
    let method = if form.payment_method.trim().is_empty() {
        existing.payment_method.clone()
    } else {
        form.payment_method.clone()
    };
    let model = payment::ActiveModel {
        id: Set(existing.id),
        created_at: Set(existing.created_at),
        updated_at: Set(Some(now)),
        deleted_at: Set(existing.deleted_at),
        student_id: Set(existing.student_id),
        amount: Set(amount),
        payment_method: Set(method),
        remarks: Set(optional_string(&form.remarks)),
        transaction_id: Set(optional_string(&form.transaction_id).or(Some(String::new()))),
        paid_at: Set(parse_date(&form.paid_at)),
    };
    match model.update(&state.db).await {
        Ok(_) => respond_edit_modal_done::<PaymentEditModalKey>(
            &htmx,
            &StudentPaymentsDetailRouteTag::new(id).url(),
        ),
        Err(e) => {
            let page = form_to_page(id, &form, student_display, e.to_string(), &q, String::new());
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
        modal_uid: PaymentDeleteModalKey::ID.to_string(),
        message: format!("Delete payment #{id}?"),
        form_name: "studentpayments.PaymentDeleteForm".into(),
        id,
        error: String::new(),
    };
    html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}

pub async fn delete_post(
    Cap(state): Cap<StudentPaymentsState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    Path(id): Path<i64>,
) -> Response {
    if let Some(resp) = forbid_non_admin(&ctx) {
        return resp;
    }
    let Some(existing) = find_payment(&state.db, id).await else {
        return Redirect::to("/student-payments/").into_response();
    };
    let now = Utc::now();
    let model = payment::ActiveModel {
        id: Set(existing.id),
        deleted_at: Set(Some(now)),
        ..Default::default()
    };
    match model.update(&state.db).await {
        Ok(_) => Redirect::to("/student-payments/").into_response(),
        Err(e) => {
            let page = ConfirmDeletePage {
                modal_uid: PaymentDeleteModalKey::ID.to_string(),
                message: format!("Delete payment #{id}?"),
                form_name: "studentpayments.PaymentDeleteForm".into(),
                id,
                error: e.to_string(),
            };
            html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
        }
    }
}

pub async fn select(
    Cap(state): Cap<StudentPaymentsState>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    uri: Uri,
    Query(q): Query<PaymentSelectQuery>,
) -> Response {
    if let Some(resp) = forbid_if_no_access(&ctx) {
        return resp;
    }
    let payments = query_payments(&state.db, &q.filter, &ctx).await;
    let page = PaymentSelectPage {
        payments,
        filter_transaction_id: q.filter.transaction_id.clone().unwrap_or_default(),
        path_and_query: path_and_query(&uri),
        target_input: q.target_input.unwrap_or_default(),
    };
    respond_picker_select::<PaymentSelectTableKey, PaymentSelectModalKey, _>(&htmx, &page)
        .into_response()
}
