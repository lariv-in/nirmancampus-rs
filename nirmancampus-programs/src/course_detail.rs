//! Registers a Programs section on the courses detail hub.

use maud::html;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder};

use lariv_rs::components::{
    FieldText, TableColumnHeader, TableRow, data_table_list_grid, field_text,
    row_attr_navigate_route,
};
use nirmancampus_courses::course_detail_related::{
    self, CourseDetailRelatedRegistrar, CourseDetailRelatedRegistry,
};

use crate::entities::program::{self, Entity as ProgramEntity};
use crate::entities::program_structure_unit::{self, Entity as StructureUnitEntity};
use crate::entities::program_structure_unit_compulsory_course::{
    self, Entity as CompulsoryLinkEntity,
};
use crate::entities::program_structure_unit_optional_course::{
    self, Entity as OptionalLinkEntity,
};
use crate::handlers::programs::scope_programs;
use crate::keys::CourseDetailProgramPlacementsKey;
use crate::routes::ProgramsDetailRouteTag;
use crate::templates::program_display_label;

#[derive(Clone, Copy, Default)]
pub struct CourseDetailHook;

impl CourseDetailRelatedRegistrar for CourseDetailHook {
    fn register_course_detail_related(
        self,
        cap: CourseDetailRelatedRegistry,
    ) -> CourseDetailRelatedRegistry {
        cap.push(course_detail_related::section(10, |db, course_id, auth| async move {
            programs_section(&db, course_id, &auth).await
        }))
    }
}

struct PlacementRow {
    program_id: i64,
    program_label: String,
    term_number: i64,
    kind: &'static str,
}

async fn programs_section(
    db: &DatabaseConnection,
    course_id: i64,
    auth: &lariv_rs::plugins::users::state::AuthContext,
) -> String {
    let mut rows = Vec::new();
    collect_kind(
        db,
        auth,
        &mut rows,
        CompulsoryLinkEntity::find()
            .filter(program_structure_unit_compulsory_course::Column::CourseId.eq(course_id))
            .all(db)
            .await
            .unwrap_or_default()
            .into_iter()
            .map(|l| l.program_structure_unit_id)
            .collect(),
        "Compulsory",
    )
    .await;
    collect_kind(
        db,
        auth,
        &mut rows,
        OptionalLinkEntity::find()
            .filter(program_structure_unit_optional_course::Column::CourseId.eq(course_id))
            .all(db)
            .await
            .unwrap_or_default()
            .into_iter()
            .map(|l| l.program_structure_unit_id)
            .collect(),
        "Optional pool",
    )
    .await;

    rows.sort_by(|a, b| {
        a.program_label
            .cmp(&b.program_label)
            .then(a.term_number.cmp(&b.term_number))
            .then(a.kind.cmp(b.kind))
    });

    let headers = [
        TableColumnHeader {
            key: "Program",
            label: "Program",
            sort_url: None,
            push_url: false,
        },
        TableColumnHeader {
            key: "Term",
            label: "Term",
            sort_url: None,
            push_url: false,
        },
        TableColumnHeader {
            key: "Kind",
            label: "Role",
            sort_url: None,
            push_url: false,
        },
    ];
    let table_rows: Vec<TableRow> = rows
        .iter()
        .map(|r| {
            let term = r.term_number.to_string();
            TableRow {
                attrs: row_attr_navigate_route(ProgramsDetailRouteTag::new(r.program_id)),
                cells: vec![
                    field_text(FieldText {
                        value: &r.program_label,
                        classes: "",
                    }),
                    field_text(FieldText {
                        value: &term,
                        classes: "",
                    }),
                    field_text(FieldText {
                        value: r.kind,
                        classes: "",
                    }),
                ],
            }
        })
        .collect();

    html! {
        div class="w-full mt-4" {
            (data_table_list_grid::<CourseDetailProgramPlacementsKey>(
                "Programs",
                html! {},
                &headers,
                &table_rows,
                html! {},
            ))
        }
    }
    .into_string()
}

async fn collect_kind(
    db: &DatabaseConnection,
    auth: &lariv_rs::plugins::users::state::AuthContext,
    rows: &mut Vec<PlacementRow>,
    unit_ids: Vec<i64>,
    kind: &'static str,
) {
    if unit_ids.is_empty() {
        return;
    }
    let units = StructureUnitEntity::find()
        .filter(program_structure_unit::Column::Id.is_in(unit_ids))
        .filter(program_structure_unit::Column::DeletedAt.is_null())
        .order_by_asc(program_structure_unit::Column::TermNumber)
        .all(db)
        .await
        .unwrap_or_default();
    if units.is_empty() {
        return;
    }
    let program_ids: Vec<i64> = units.iter().map(|u| u.program_id).collect();
    let mut query = ProgramEntity::find()
        .filter(program::Column::Id.is_in(program_ids))
        .filter(program::Column::DeletedAt.is_null());
    query = scope_programs(query, auth);
    let programs = query.all(db).await.unwrap_or_default();
    for unit in units {
        let Some(program) = programs.iter().find(|p| p.id == unit.program_id) else {
            continue;
        };
        rows.push(PlacementRow {
            program_id: program.id,
            program_label: program_display_label(program.name(), &program.university),
            term_number: unit.term_number,
            kind,
        });
    }
}
