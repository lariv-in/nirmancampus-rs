//! Embeds exam registrations on academic record detail via `AcademicRecordDetailRelatedCap`.

use maud::html;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder};

use lariv_rs::{
    components::{
        ButtonModalForm, FieldText, TableColumnHeader, TableRow, button_download_route,
        button_modal_form, data_table_list_grid, field_text, row_attr_navigate_route,
    },
    http::RouteQueryBuilder,
    plugins::users::state::AuthContext,
};
use nirmancampus_academicrecords::academic_record_detail_related::{
    self, AcademicRecordDetailRelatedRegistrar, AcademicRecordDetailRelatedRegistry,
};
use nirmancampus_common::{format_inr, is_admin};

use crate::entities::exam_registration::{self, Entity as ExamEntity};
use crate::handlers::exams::{load_course, status_label};
use crate::keys::AcademicRecordExamTableKey;
use crate::routes::{
    ExamRegistrationsBulkGetRouteTag, ExamRegistrationsBulkPostRouteTag,
    ExamRegistrationsDetailRouteTag, ExamRegistrationsReceiptRouteTag,
};

#[derive(Clone, Copy, Default)]
pub struct AcademicRecordDetailHook;

impl AcademicRecordDetailRelatedRegistrar for AcademicRecordDetailHook {
    fn register_academic_record_detail_related(
        self,
        cap: AcademicRecordDetailRelatedRegistry,
    ) -> AcademicRecordDetailRelatedRegistry {
        cap.push(academic_record_detail_related::section(
            20,
            |db, academic_record_id, auth| async move {
                exams_section(&db, academic_record_id, &auth).await
            },
        ))
    }
}

struct ExamCard {
    id: i64,
    exam_title: String,
    course_name: String,
    fee: String,
    status: String,
}

async fn exams_section(
    db: &DatabaseConnection,
    academic_record_id: i64,
    auth: &AuthContext,
) -> String {
    let Ok(rows) = ExamEntity::find()
        .filter(exam_registration::Column::DeletedAt.is_null())
        .filter(exam_registration::Column::AcademicRecordId.eq(academic_record_id))
        .order_by_desc(exam_registration::Column::Id)
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
        items.push(ExamCard {
            id: e.id,
            exam_title: e.exam_title().to_string(),
            course_name,
            fee: format_inr(e.fee),
            status: status_label(&e.registration_status),
        });
    }

    let headers = [
        TableColumnHeader {
            key: "Exam",
            label: "Exam",
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
            key: "Fee",
            label: "Fee",
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
            attrs: row_attr_navigate_route(ExamRegistrationsDetailRouteTag::new(e.id)),
            cells: vec![
                field_text(FieldText {
                    value: &e.exam_title,
                    classes: "",
                }),
                field_text(FieldText {
                    value: &e.course_name,
                    classes: "",
                }),
                field_text(FieldText {
                    value: &e.fee,
                    classes: "",
                }),
                field_text(FieldText {
                    value: &e.status,
                    classes: "",
                }),
            ],
        })
        .collect();

    let bulk_get = RouteQueryBuilder::new(ExamRegistrationsBulkGetRouteTag)
        .query("AcademicRecordID", academic_record_id)
        .build();
    let bulk_post = RouteQueryBuilder::new(ExamRegistrationsBulkPostRouteTag)
        .query("AcademicRecordID", academic_record_id)
        .build_with_query();
    let admin = is_admin(auth);
    html! {
        div class="mt-4" {
            (data_table_list_grid::<AcademicRecordExamTableKey>(
                "Exam Registrations",
                html! {
                    @if admin {
                        (button_modal_form(ButtonModalForm {
                            href: &bulk_get,
                            form_post_url: &bulk_post,
                            label: "Create Exam Registrations for Student",
                            modal_uid: "examregistrations-bulk-create-academic-record-modal",
                            classes: "btn-outline btn-sm",
                            ..Default::default()
                        }))
                    }
                    (button_download_route(
                        ExamRegistrationsReceiptRouteTag::new(academic_record_id),
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
