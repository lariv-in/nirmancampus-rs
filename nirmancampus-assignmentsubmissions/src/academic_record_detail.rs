//! Embeds assignment submissions on academic record detail via `AcademicRecordDetailRelatedCap`.

use maud::html;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder};

use lariv_rs::{
    components::{
        ButtonModalForm, FieldText, SwapKey, TableColumnHeader, TableRow, button_download_route,
        button_modal_form, data_table_list_grid, field_text, row_attr_navigate_route,
    },
    http::RouteQueryBuilder,
    plugins::users::state::AuthContext,
};
use nirmancampus_academicrecords::academic_record_detail_related::{
    self, AcademicRecordDetailRelatedRegistrar, AcademicRecordDetailRelatedRegistry,
};
use nirmancampus_common::is_admin;

use crate::entities::assignment_submission::{self, Entity as AssignmentEntity};
use crate::handlers::assignments::{load_course, status_label};
use crate::keys::{
    AcademicRecordAssignmentTableKey, AssignmentBulkCreateModalKey, AssignmentBulkMarksModalKey,
};
use crate::routes::{
    AssignmentSubmissionsBulkCreateGetRouteTag, AssignmentSubmissionsBulkCreatePostRouteTag,
    AssignmentSubmissionsBulkMarksGetRouteTag, AssignmentSubmissionsBulkMarksPostRouteTag,
    AssignmentSubmissionsDetailRouteTag, AssignmentSubmissionsReceiptRouteTag,
};

#[derive(Clone, Copy, Default)]
pub struct AcademicRecordDetailHook;

impl AcademicRecordDetailRelatedRegistrar for AcademicRecordDetailHook {
    fn register_academic_record_detail_related(
        self,
        cap: AcademicRecordDetailRelatedRegistry,
    ) -> AcademicRecordDetailRelatedRegistry {
        cap.push(academic_record_detail_related::section(
            30,
            |db, academic_record_id, auth| async move {
                assignments_section(&db, academic_record_id, &auth).await
            },
        ))
    }
}

struct AssignmentCard {
    id: i64,
    assignment_title: String,
    course_name: String,
    status: String,
}

async fn assignments_section(
    db: &DatabaseConnection,
    academic_record_id: i64,
    auth: &AuthContext,
) -> String {
    let Ok(rows) = AssignmentEntity::find()
        .filter(assignment_submission::Column::DeletedAt.is_null())
        .filter(assignment_submission::Column::AcademicRecordId.eq(academic_record_id))
        .order_by_desc(assignment_submission::Column::CreatedAt)
        .order_by_desc(assignment_submission::Column::Id)
        .all(db)
        .await
    else {
        return String::new();
    };

    let mut items = Vec::with_capacity(rows.len());
    for e in rows {
        let course_name = load_course(db, e.course_id)
            .await
            .map(|c| c.name().to_string())
            .unwrap_or_default();
        items.push(AssignmentCard {
            id: e.id,
            assignment_title: e.assignment_title().to_string(),
            course_name,
            status: status_label(&e.submission_status),
        });
    }

    let headers = [
        TableColumnHeader {
            key: "Assignment",
            label: "Assignment",
            sort_url: None,
            push_url: false,
        },
        TableColumnHeader {
            key: "Course",
            label: "Course",
            sort_url: None,
            push_url: false,
        },
        TableColumnHeader {
            key: "Status",
            label: "Status",
            sort_url: None,
            push_url: false,
        },
    ];
    let table_rows: Vec<TableRow> = items
        .iter()
        .map(|e| TableRow {
            attrs: row_attr_navigate_route(AssignmentSubmissionsDetailRouteTag::new(e.id)),
            cells: vec![
                field_text(FieldText {
                    value: &e.assignment_title,
                    classes: "",
                }),
                field_text(FieldText {
                    value: &e.course_name,
                    classes: "",
                }),
                field_text(FieldText {
                    value: &e.status,
                    classes: "",
                }),
            ],
        })
        .collect();

    let bulk_create_get = RouteQueryBuilder::new(AssignmentSubmissionsBulkCreateGetRouteTag)
        .query("AcademicRecordID", academic_record_id)
        .build();
    let bulk_create_post = RouteQueryBuilder::new(AssignmentSubmissionsBulkCreatePostRouteTag)
        .query("AcademicRecordID", academic_record_id)
        .build_with_query();
    let bulk_marks_get = RouteQueryBuilder::new(AssignmentSubmissionsBulkMarksGetRouteTag)
        .query("AcademicRecordID", academic_record_id)
        .build();
    let bulk_marks_post = RouteQueryBuilder::new(AssignmentSubmissionsBulkMarksPostRouteTag)
        .query("AcademicRecordID", academic_record_id)
        .build_with_query();
    let admin = is_admin(auth);
    html! {
        div class="mt-4" {
            (data_table_list_grid::<AcademicRecordAssignmentTableKey>(
                "Assignment Submissions",
                html! {
                    @if admin {
                        (button_modal_form(ButtonModalForm {
                            href: &bulk_create_get,
                            form_post_url: &bulk_create_post,
                            label: "Create Submissions for Student",
                            modal_uid: AssignmentBulkCreateModalKey::ID,
                            classes: "btn-outline btn-sm",
                            ..Default::default()
                        }))
                        (button_modal_form(ButtonModalForm {
                            href: &bulk_marks_get,
                            form_post_url: &bulk_marks_post,
                            label: "Add Marks for Student",
                            modal_uid: AssignmentBulkMarksModalKey::ID,
                            classes: "btn-outline btn-sm",
                            ..Default::default()
                        }))
                    }
                    (button_download_route(
                        AssignmentSubmissionsReceiptRouteTag::new(academic_record_id),
                        "Download Receipt",
                        "btn-outline btn-secondary btn-sm",
                    ))
                },
                &headers,
                &table_rows,
                html! {},
            ))
        }
    }
    .into_string()
}
