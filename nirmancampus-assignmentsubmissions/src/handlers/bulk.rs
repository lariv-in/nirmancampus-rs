use axum::{
    extract::{Form, Query},
    response::{IntoResponse, Redirect, Response},
};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter, QueryOrder,
};
use serde::Deserialize;

use crate::{
    entities::assignment_submission::{self, Entity as AssignmentEntity},
    handlers::assignments::{
        STATUS_MARKED, STATUS_NOT_MARKED, academic_record_detail_url, forbid_non_admin,
        load_academic_record, load_course, record_courses, record_label,
    },
    state::AssignmentSubmissionsState,
    templates::{BulkCourseRow, BulkCreatePage, BulkMarksPage, BulkMarksRow},
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
pub struct BulkCreateBody {
    #[serde(default, rename = "AcademicRecordID", alias = "academic_record_id")]
    pub academic_record_id: Option<i64>,
    #[serde(default, rename = "CourseIDs", deserialize_with = "form_vec_i64")]
    pub course_ids: Vec<i64>,
}

#[derive(Debug, Deserialize, Default)]
pub struct BulkMarksBody {
    #[serde(default, rename = "AcademicRecordID", alias = "academic_record_id")]
    pub academic_record_id: Option<i64>,
    #[serde(default, rename = "SubmissionIDs", deserialize_with = "form_vec_i64")]
    pub submission_ids: Vec<i64>,
    #[serde(default, rename = "Marks", deserialize_with = "form_vec_i64")]
    pub marks: Vec<i64>,
}

fn academic_record_id(query: Option<i64>, body: Option<i64>) -> i64 {
    body.filter(|id| *id > 0)
        .or_else(|| query.filter(|id| *id > 0))
        .unwrap_or(0)
}

async fn existing_course_ids(
    db: &sea_orm::DatabaseConnection,
    academic_record_id: i64,
) -> Vec<i64> {
    AssignmentEntity::find()
        .filter(assignment_submission::Column::DeletedAt.is_null())
        .filter(assignment_submission::Column::AcademicRecordId.eq(academic_record_id))
        .all(db)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|e| e.course_id)
        .collect()
}

async fn bulk_create_page(
    db: &sea_orm::DatabaseConnection,
    academic_record_id: i64,
    error: String,
) -> Option<BulkCreatePage> {
    load_academic_record(db, academic_record_id).await?;
    let existing = existing_course_ids(db, academic_record_id).await;
    let courses = record_courses(db, academic_record_id).await;
    let rows = courses
        .into_iter()
        .map(|c| BulkCourseRow {
            id: c.id,
            name: c.name().to_string(),
            already: existing.contains(&c.id),
        })
        .collect();
    Some(BulkCreatePage {
        academic_record_id,
        student_line: record_label(academic_record_id),
        courses: rows,
        error,
    })
}

async fn listed_submissions(
    db: &sea_orm::DatabaseConnection,
    academic_record_id: i64,
) -> Vec<assignment_submission::Model> {
    AssignmentEntity::find()
        .filter(assignment_submission::Column::DeletedAt.is_null())
        .filter(assignment_submission::Column::AcademicRecordId.eq(academic_record_id))
        .order_by_asc(assignment_submission::Column::Id)
        .all(db)
        .await
        .unwrap_or_default()
}

async fn bulk_marks_page(
    db: &sea_orm::DatabaseConnection,
    academic_record_id: i64,
    posted_marks: &[(i64, i64)],
    error: String,
) -> Option<BulkMarksPage> {
    load_academic_record(db, academic_record_id).await?;
    let rows_src = listed_submissions(db, academic_record_id).await;
    let mut rows = Vec::with_capacity(rows_src.len());
    for s in rows_src {
        let course_name = load_course(db, s.course_id)
            .await
            .map(|c| c.name().to_string())
            .unwrap_or_default();
        let marks = posted_marks
            .iter()
            .find(|(id, _)| *id == s.id)
            .map(|(_, m)| *m)
            .unwrap_or(s.marks);
        rows.push(BulkMarksRow {
            id: s.id,
            assignment_title: s.assignment_title().to_string(),
            course_name,
            max_marks: s.max_marks,
            marks,
        });
    }
    Some(BulkMarksPage {
        academic_record_id,
        student_line: record_label(academic_record_id),
        submissions: rows,
        error,
    })
}

async fn respond_bulk_create_error(
    db: &sea_orm::DatabaseConnection,
    chrome: &SharedChromeFolder,
    ctx: &lariv_rs::plugins::users::state::AuthContext,
    academic_record_id: i64,
    error: String,
) -> Response {
    let page = bulk_create_page(db, academic_record_id, error.clone())
        .await
        .unwrap_or_else(|| BulkCreatePage {
            academic_record_id,
            student_line: record_label(academic_record_id),
            courses: Vec::new(),
            error,
        });
    html_built_page_with_slots(&page, chrome, &SlotCtx::from_auth(ctx)).into_response()
}

async fn respond_bulk_marks_error(
    db: &sea_orm::DatabaseConnection,
    chrome: &SharedChromeFolder,
    ctx: &lariv_rs::plugins::users::state::AuthContext,
    academic_record_id: i64,
    posted: &[(i64, i64)],
    error: String,
) -> Response {
    let page = bulk_marks_page(db, academic_record_id, posted, error.clone())
        .await
        .unwrap_or_else(|| BulkMarksPage {
            academic_record_id,
            student_line: record_label(academic_record_id),
            submissions: Vec::new(),
            error,
        });
    html_built_page_with_slots(&page, chrome, &SlotCtx::from_auth(ctx)).into_response()
}

pub async fn bulk_create_get(
    Cap(state): Cap<AssignmentSubmissionsState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    Query(q): Query<BulkAcademicRecordQuery>,
) -> Response {
    if let Some(resp) = forbid_non_admin(&ctx) {
        return resp;
    }
    let academic_record_id = q.academic_record_id.unwrap_or(0);
    let Some(page) = bulk_create_page(&state.db, academic_record_id, String::new()).await else {
        return Redirect::to("/assignment-submissions/").into_response();
    };
    html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}

pub async fn bulk_create_post(
    Cap(state): Cap<AssignmentSubmissionsState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    Query(q): Query<BulkAcademicRecordQuery>,
    Form(body): Form<BulkCreateBody>,
) -> Response {
    if let Some(resp) = forbid_non_admin(&ctx) {
        return resp;
    }
    let academic_record_id = academic_record_id(q.academic_record_id, body.academic_record_id);
    let Some(rec) = load_academic_record(&state.db, academic_record_id).await else {
        return Redirect::to("/assignment-submissions/").into_response();
    };
    let allowed = record_courses(&state.db, rec.id).await;
    let existing = existing_course_ids(&state.db, rec.id).await;
    let selected: Vec<i64> = body
        .course_ids
        .into_iter()
        .filter(|id| *id > 0 && !existing.contains(id))
        .collect();
    if selected.is_empty() {
        return respond_bulk_create_error(
            &state.db,
            &chrome,
            &ctx,
            academic_record_id,
            "select at least one course".into(),
        )
        .await;
    }
    for cid in &selected {
        if !allowed.iter().any(|c| c.id == *cid) {
            return respond_bulk_create_error(
                &state.db,
                &chrome,
                &ctx,
                academic_record_id,
                "one or more selected courses are not on this academic record".into(),
            )
            .await;
        }
    }
    let now = Utc::now();
    for cid in selected {
        let Some(course) = allowed.iter().find(|c| c.id == cid) else {
            continue;
        };
        let model = assignment_submission::ActiveModel {
            id: Default::default(),
            created_at: Set(Some(now)),
            updated_at: Set(Some(now)),
            deleted_at: Set(None),
            assignment_title: Set(course.name().to_string()),
            max_marks: Set(0),
            submission_status: Set(STATUS_NOT_MARKED.to_string()),
            marks: Set(0),
            course_id: Set(cid),
            academic_record_id: Set(rec.id),
        };
        if let Err(e) = model.insert(&state.db).await {
            return respond_bulk_create_error(
                &state.db,
                &chrome,
                &ctx,
                academic_record_id,
                e.to_string(),
            )
            .await;
        }
    }
    Redirect::to(&academic_record_detail_url(rec.id)).into_response()
}

pub async fn bulk_marks_get(
    Cap(state): Cap<AssignmentSubmissionsState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    Query(q): Query<BulkAcademicRecordQuery>,
) -> Response {
    if let Some(resp) = forbid_non_admin(&ctx) {
        return resp;
    }
    let academic_record_id = q.academic_record_id.unwrap_or(0);
    let Some(page) = bulk_marks_page(&state.db, academic_record_id, &[], String::new()).await
    else {
        return Redirect::to("/assignment-submissions/").into_response();
    };
    html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}

pub async fn bulk_marks_post(
    Cap(state): Cap<AssignmentSubmissionsState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    Query(q): Query<BulkAcademicRecordQuery>,
    Form(body): Form<BulkMarksBody>,
) -> Response {
    if let Some(resp) = forbid_non_admin(&ctx) {
        return resp;
    }
    let academic_record_id = academic_record_id(q.academic_record_id, body.academic_record_id);
    let Some(rec) = load_academic_record(&state.db, academic_record_id).await else {
        return Redirect::to("/assignment-submissions/").into_response();
    };
    let existing = listed_submissions(&state.db, rec.id).await;
    if existing.is_empty() {
        return Redirect::to(&academic_record_detail_url(rec.id)).into_response();
    }
    let posted: Vec<(i64, i64)> = body
        .submission_ids
        .iter()
        .copied()
        .zip(body.marks.iter().copied())
        .filter(|(id, _)| *id > 0)
        .collect();
    if posted.len() != existing.len() {
        return respond_bulk_marks_error(
            &state.db,
            &chrome,
            &ctx,
            academic_record_id,
            &posted,
            "enter marks for every listed submission".into(),
        )
        .await;
    }
    let mut seen = std::collections::HashSet::new();
    for (id, marks) in &posted {
        if !seen.insert(*id) {
            return respond_bulk_marks_error(
                &state.db,
                &chrome,
                &ctx,
                academic_record_id,
                &posted,
                "duplicate submission id in payload".into(),
            )
            .await;
        }
        let Some(sub) = existing.iter().find(|s| s.id == *id) else {
            return respond_bulk_marks_error(
                &state.db,
                &chrome,
                &ctx,
                academic_record_id,
                &posted,
                "one or more submissions are not on this academic record".into(),
            )
            .await;
        };
        if *marks < 0 {
            return respond_bulk_marks_error(
                &state.db,
                &chrome,
                &ctx,
                academic_record_id,
                &posted,
                "marks cannot be negative".into(),
            )
            .await;
        }
        if *marks > sub.max_marks {
            return respond_bulk_marks_error(
                &state.db,
                &chrome,
                &ctx,
                academic_record_id,
                &posted,
                format!(
                    "marks cannot exceed max marks for \"{}\"",
                    sub.assignment_title()
                ),
            )
            .await;
        }
    }
    let now = Utc::now();
    for (id, marks) in &posted {
        let Some(existing_row) = existing.iter().find(|s| s.id == *id) else {
            continue;
        };
        let model = assignment_submission::ActiveModel {
            id: Set(existing_row.id),
            created_at: Set(existing_row.created_at),
            updated_at: Set(Some(now)),
            deleted_at: Set(existing_row.deleted_at),
            assignment_title: Set(existing_row.assignment_title.clone()),
            max_marks: Set(existing_row.max_marks),
            submission_status: Set(STATUS_MARKED.to_string()),
            marks: Set(*marks),
            course_id: Set(existing_row.course_id),
            academic_record_id: Set(existing_row.academic_record_id),
        };
        if let Err(e) = model.update(&state.db).await {
            return respond_bulk_marks_error(
                &state.db,
                &chrome,
                &ctx,
                academic_record_id,
                &posted,
                e.to_string(),
            )
            .await;
        }
    }
    Redirect::to(&academic_record_detail_url(rec.id)).into_response()
}
