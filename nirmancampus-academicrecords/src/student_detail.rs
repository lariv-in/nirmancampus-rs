//! Embeds academic records on student detail via `StudentDetailRelatedCap`.

use maud::html;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder};

use lariv_rs::{
    components::{
        ButtonModalForm, FieldText, TableColumnHeader, TableRow, button_modal_form,
        data_table_list_grid, field_text, row_attr_navigate_route, SwapKey,
    },
    http::RouteQueryBuilder,
    plugins::users::state::AuthContext,
    web::CreateModal,
};
use nirmancampus_common::{is_admin, program_display};
use nirmancampus_programs::entities::program::{self, Entity as ProgramEntity};
use nirmancampus_sessions::entities::admission_session::{self, Entity as SessionEntity};
use nirmancampus_students::student_detail_related::{
    self, StudentDetailRelatedRegistrar, StudentDetailRelatedRegistry,
};

use crate::entities::academic_record::{self, Entity as AcademicRecordEntity};
use crate::handlers::records::{load_program_structure_unit, term_label};
use crate::keys::{AcademicRecordCreateModalKey, StudentDetailAcademicRecordsKey};
use crate::routes::{AcademicRecordsCreateGetRouteTag, AcademicRecordsDetailRouteTag};

#[derive(Clone, Copy, Default)]
pub struct StudentDetailHook;

impl StudentDetailRelatedRegistrar for StudentDetailHook {
    fn register_student_detail_related(
        self,
        cap: StudentDetailRelatedRegistry,
    ) -> StudentDetailRelatedRegistry {
        cap.push(student_detail_related::section(20, |db, student_id, auth| async move {
            records_section(&db, student_id, &auth).await
        }))
    }
}

fn create_url_with_student(student_id: i64) -> String {
    RouteQueryBuilder::new(AcademicRecordsCreateGetRouteTag)
        .query("StudentID", student_id)
        .build()
}

struct RecordCard {
    id: i64,
    program: String,
    session: String,
    status: String,
    term: String,
}

async fn records_section(db: &DatabaseConnection, student_id: i64, auth: &AuthContext) -> String {
    let Ok(rows) = AcademicRecordEntity::find()
        .filter(academic_record::Column::DeletedAt.is_null())
        .filter(academic_record::Column::StudentId.eq(student_id))
        .order_by_desc(academic_record::Column::Id)
        .all(db)
        .await
    else {
        return String::new();
    };

    let mut items = Vec::with_capacity(rows.len());
    for r in rows {
        let program = match ProgramEntity::find_by_id(r.program_id)
            .filter(program::Column::DeletedAt.is_null())
            .one(db)
            .await
        {
            Ok(Some(p)) => program_display(p.name(), &p.university),
            _ => format!("Program #{}", r.program_id),
        };
        let session = match SessionEntity::find_by_id(r.session_id)
            .filter(admission_session::Column::DeletedAt.is_null())
            .one(db)
            .await
        {
            Ok(Some(s)) => s.name().to_string(),
            _ => format!("Session #{}", r.session_id),
        };
        let term = load_program_structure_unit(db, r.program_structure_unit_id)
            .await
            .map(|u| term_label(&u))
            .unwrap_or_default();
        items.push(RecordCard {
            id: r.id,
            program,
            session,
            status: r.status().to_string(),
            term,
        });
    }

    let headers = [
        TableColumnHeader {
            key: "Program",
            label: "Program",
            sort_url: None,
            push_url: false,
        },
        TableColumnHeader {
            key: "AdmissionSession",
            label: "Admission session",
            sort_url: None,
            push_url: false,
        },
        TableColumnHeader {
            key: "Status",
            label: "Status",
            sort_url: None,
            push_url: false,
        },
        TableColumnHeader {
            key: "Term",
            label: "Term",
            sort_url: None,
            push_url: false,
        },
    ];
    let table_rows: Vec<TableRow> = items
        .iter()
        .map(|r| TableRow {
            attrs: row_attr_navigate_route(AcademicRecordsDetailRouteTag::new(r.id)),
            cells: vec![
                field_text(FieldText {
                    value: &r.program,
                    classes: "",
                }),
                field_text(FieldText {
                    value: &r.session,
                    classes: "",
                }),
                field_text(FieldText {
                    value: &r.status,
                    classes: "",
                }),
                field_text(FieldText {
                    value: &r.term,
                    classes: "",
                }),
            ],
        })
        .collect();

    let create_url = create_url_with_student(student_id);
    let admin = is_admin(auth);
    html! {
        div class="mt-4" {
            (data_table_list_grid::<StudentDetailAcademicRecordsKey>(
                "Academic records",
                html! {
                    @if admin {
                        (button_modal_form(ButtonModalForm {
                            href: &create_url,
                            name: AcademicRecordCreateModalKey::FORM_NAME,
                            modal_uid: AcademicRecordCreateModalKey::ID,
                            label: "",
                            icon_name: Some("plus"),
                            classes: "btn-square btn-outline btn-sm",
                            ..Default::default()
                        }))
                    }
                },
                &headers,
                &table_rows,
                html! {},
            ))
        }
    }
    .into_string()
}
