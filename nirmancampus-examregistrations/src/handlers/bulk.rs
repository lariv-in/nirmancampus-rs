use axum::{
    extract::{Form, Query},
    response::{IntoResponse, Redirect, Response},
};
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter};
use serde::Deserialize;

use crate::{
    entities::exam_registration::{self, Entity as ExamEntity},
    handlers::exams::{
        forbid_non_admin, load_academic_record, record_courses, record_label, STATUS_NOT_REGISTERED,
    },
    state::ExamRegistrationsState,
    templates::{BulkCourseRow, BulkFromRecordPage},
};
use lariv_rs::{
    components::{SharedChromeFolder, SlotCtx},
    html_form::form_vec_i64,
    http::Cap,
    plugins::users::middleware::RequireAuth,
    web::html_built_page_with_slots,
};

#[derive(Debug, Deserialize, Default)]
pub struct BulkAcademicRecordQuery {
    #[serde(default, rename = "AcademicRecordID", alias = "academic_record_id")]
    pub academic_record_id: Option<i64>,
}

#[derive(Debug, Deserialize, Default)]
pub struct BulkFromRecordBody {
    #[serde(default, rename = "AcademicRecordID", alias = "academic_record_id")]
    pub academic_record_id: Option<i64>,
    #[serde(default, rename = "CourseIDs", deserialize_with = "form_vec_i64")]
    pub course_ids: Vec<i64>,
}

fn academic_record_id(query: Option<i64>, body: Option<i64>) -> i64 {
    body.filter(|id| *id > 0)
        .or(query.filter(|id| *id > 0))
        .unwrap_or(0)
}

async fn existing_course_ids(db: &sea_orm::DatabaseConnection, academic_record_id: i64) -> Vec<i64> {
    ExamEntity::find()
        .filter(exam_registration::Column::DeletedAt.is_null())
        .filter(exam_registration::Column::AcademicRecordId.eq(academic_record_id))
        .all(db)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|e| e.course_id)
        .collect()
}

async fn bulk_page(
    db: &sea_orm::DatabaseConnection,
    academic_record_id: i64,
    error: String,
) -> Option<BulkFromRecordPage> {
    load_academic_record(db, academic_record_id).await?;
    let existing = existing_course_ids(db, academic_record_id).await;
    let courses = record_courses(db, academic_record_id).await;
    let rows = courses
        .into_iter()
        .map(|c| BulkCourseRow {
            id: c.id,
            name: c.name().to_string(),
            fee: c.fee,
            already: existing.contains(&c.id),
        })
        .collect();
    Some(BulkFromRecordPage {
        academic_record_id,
        student_line: record_label(academic_record_id),
        courses: rows,
        error,
    })
}

pub async fn bulk_get(
    Cap(state): Cap<ExamRegistrationsState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    Query(q): Query<BulkAcademicRecordQuery>,
) -> Response {
    if let Some(resp) = forbid_non_admin(&ctx) {
        return resp;
    }
    let academic_record_id = q.academic_record_id.unwrap_or(0);
    let Some(page) = bulk_page(&state.db, academic_record_id, String::new()).await else {
        return Redirect::to("/exam-registrations/").into_response();
    };
    html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}

pub async fn bulk_post(
    Cap(state): Cap<ExamRegistrationsState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    Query(q): Query<BulkAcademicRecordQuery>,
    Form(body): Form<BulkFromRecordBody>,
) -> Response {
    if let Some(resp) = forbid_non_admin(&ctx) {
        return resp;
    }
    let academic_record_id = academic_record_id(q.academic_record_id, body.academic_record_id);
    let Some(rec) = load_academic_record(&state.db, academic_record_id).await else {
        return Redirect::to("/exam-registrations/").into_response();
    };
    let allowed = record_courses(&state.db, rec.id).await;
    let existing = existing_course_ids(&state.db, rec.id).await;
    let selected: Vec<i64> = body
        .course_ids
        .into_iter()
        .filter(|id| *id > 0 && !existing.contains(id))
        .collect();
    if selected.is_empty() {
        let page = bulk_page(
            &state.db,
            academic_record_id,
            "select at least one course".into(),
        )
        .await
        .unwrap_or_else(|| BulkFromRecordPage {
            academic_record_id,
            student_line: record_label(academic_record_id),
            courses: Vec::new(),
            error: "select at least one course".into(),
        });
        return html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx)).into_response();
    }
    for cid in &selected {
        if !allowed.iter().any(|c| c.id == *cid) {
            let page = bulk_page(
                &state.db,
                academic_record_id,
                "one or more selected courses are not on this academic record".into(),
            )
            .await
            .unwrap();
            return html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx))
                .into_response();
        }
    }
    let now = Utc::now();
    for cid in selected {
        let Some(course) = allowed.iter().find(|c| c.id == cid) else {
            continue;
        };
        let model = exam_registration::ActiveModel {
            id: Default::default(),
            created_at: Set(Some(now)),
            updated_at: Set(Some(now)),
            deleted_at: Set(None),
            exam_title: Set(course.name().to_string()),
            max_marks: Set(0),
            registration_status: Set(STATUS_NOT_REGISTERED.to_string()),
            marks: Set(0),
            fee: Set(course.fee),
            course_id: Set(cid),
            academic_record_id: Set(rec.id),
        };
        if let Err(e) = model.insert(&state.db).await {
            let page = bulk_page(&state.db, academic_record_id, e.to_string())
                .await
                .unwrap();
            return html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx))
                .into_response();
        }
    }
    Redirect::to(&format!("/academic-records/{academic_record_id}/")).into_response()
}
