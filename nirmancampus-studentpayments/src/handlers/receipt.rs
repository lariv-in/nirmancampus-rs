use axum::{
    body::Body,
    extract::Path,
    http::{header, HeaderValue, StatusCode},
    response::{IntoResponse, Redirect, Response},
};
use chrono::Utc;
use rust_decimal::Decimal;

use lariv_rs::{http::Cap, plugins::users::middleware::RequireAuth};
use nirmancampus_common::doc_export::{attachment_filename, export_pdf};
use nirmancampus_common::can_view_campus_records;

use crate::{
    handlers::payments::{find_payment_scoped, load_student, payment_method_label, student_label},
    state::StudentPaymentsState,
};

fn md_cell(s: &str) -> String {
    let t = s.replace('|', "\\|").trim().to_string();
    if t.is_empty() {
        "—".into()
    } else {
        t
    }
}

/// Build a markdown payment receipt table for PDF / attachment export.
pub fn payment_receipt_markdown(
    issued: &str,
    student_name: &str,
    student_no: &str,
    email: &str,
    phone: &str,
    payment_id: i64,
    amount: Decimal,
    method: &str,
    transaction_id: &str,
    paid_on: &str,
    remarks: &str,
) -> String {
    format!(
        r#"# Payment receipt

**Issued:** {issued}

## Student

| Field | Value |
|---|---|
| Name | {name} |
| Enrolment no. | {no} |
| Email | {email} |
| Phone | {phone} |

## Payment

| Field | Value |
|---|---|
| Receipt no. | {id} |
| Amount | ₹ {amount:.2} |
| Method | {method} |
| Transaction ID | {txn} |
| Paid on | {paid} |
| Remarks | {remarks} |

---

This certifies payment of **₹ {amount:.2}** recorded as receipt **#{id}**, received from the student named above.
"#,
        issued = md_cell(issued),
        name = md_cell(student_name),
        no = md_cell(student_no),
        email = md_cell(email),
        phone = md_cell(phone),
        id = payment_id,
        amount = amount,
        method = md_cell(method),
        txn = md_cell(transaction_id),
        paid = md_cell(paid_on),
        remarks = md_cell(remarks),
    )
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

pub async fn download(
    Cap(state): Cap<StudentPaymentsState>,
    RequireAuth(ctx): RequireAuth,
    Path(id): Path<i64>,
) -> Response {
    if !can_view_campus_records(&ctx) {
        return Redirect::to("/").into_response();
    }
    let Some(payment) = find_payment_scoped(&state.db, id, &ctx).await else {
        return Redirect::to("/student-payments/").into_response();
    };
    let student = load_student(&state.db, payment.student_id).await;
    let (name, no, email, phone) = match &student {
        Some(s) => (
            s.name().to_string(),
            s.student_no.clone(),
            s.email().to_string(),
            s.phone().to_string(),
        ),
        None => (
            String::new(),
            String::new(),
            String::new(),
            String::new(),
        ),
    };
    let issued = ctx.format_datetime(Utc::now()).into_string();
    let paid_on = payment
        .paid_at
        .map(lariv_rs::datetime::format_date)
        .unwrap_or_default();
    let md = payment_receipt_markdown(
        &issued,
        &name,
        &no,
        &email,
        &phone,
        payment.id,
        payment.amount,
        &payment_method_label(&payment.payment_method),
        payment.transaction_id(),
        &paid_on,
        payment.remarks(),
    );
    let base = student
        .as_ref()
        .map(|s| student_label(s))
        .unwrap_or_else(|| format!("payment-{id}"));
    match export_pdf(&md).await {
        Ok(bytes) => file_response(
            "application/pdf",
            &attachment_filename(&format!("{base}-receipt-{id}"), "pdf"),
            bytes,
        ),
        Err(e) => {
            tracing::warn!(error = %e, id, "PDF export failed; falling back to markdown");
            file_response(
                "text/markdown; charset=utf-8",
                &attachment_filename(&format!("{base}-receipt-{id}"), "md"),
                md.into_bytes(),
            )
        }
    }
}
