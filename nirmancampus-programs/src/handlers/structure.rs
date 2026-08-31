use axum::{
    extract::{Path, Query},
    response::{IntoResponse, Redirect, Response},
};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};
use serde::Deserialize;

use crate::{
    entities::{
        program::{self, Entity as ProgramEntity},
        program_structure_unit::{self, Entity as StructureUnitEntity},
        program_structure_unit_compulsory_course::{
            self, Entity as CompulsoryLinkEntity,
        },
        program_structure_unit_optional_course::{self, Entity as OptionalLinkEntity},
    },
    forms::StructureUnitForm,
    handlers::{course_items_from_ids, programs::load_structure_units, programs::scope_programs},
    keys::StructureUnitDeleteModalKey,
    routes::ProgramsStructureEditRouteTag,
    state::ProgramsState,
    templates::{ConfirmDeletePage, ProgramStructureEditPage, StructureUnitFormPage},
};
use lariv_rs::{
    components::{SharedChromeFolder, SlotCtx, SwapKey},
    html_form::HtmlFormBody,
    http::Cap,
    plugins::users::middleware::RequireAuth,
    web::{html_built_page_or_app_layout, html_built_page_with_slots, Htmx, ModalFormQuery},
};
use nirmancampus_common::is_admin;

#[derive(Debug, Deserialize)]
pub struct UnitPath {
    pub id: i64,
    pub unit_id: i64,
}

async fn find_program_admin(
    db: &DatabaseConnection,
    id: i64,
    auth: &lariv_rs::plugins::users::state::AuthContext,
) -> Option<program::Model> {
    if !is_admin(auth) {
        return None;
    }
    let query = ProgramEntity::find_by_id(id).filter(program::Column::DeletedAt.is_null());
    let query = scope_programs(query, auth);
    lariv_rs::web::opt_or_log(query.one(db).await, "db find one")
}

async fn find_unit(
    db: &DatabaseConnection,
    program_id: i64,
    unit_id: i64,
) -> Option<program_structure_unit::Model> {
    lariv_rs::web::opt_or_log(
        StructureUnitEntity::find_by_id(unit_id)
            .filter(program_structure_unit::Column::ProgramId.eq(program_id))
            .filter(program_structure_unit::Column::DeletedAt.is_null())
            .one(db)
            .await,
        "db find one",
    )
}

async fn sync_unit_courses(
    db: &DatabaseConnection,
    unit_id: i64,
    compulsory_ids: &[i64],
    optional_ids: &[i64],
) -> Result<(), sea_orm::DbErr> {
    CompulsoryLinkEntity::delete_many()
        .filter(program_structure_unit_compulsory_course::Column::ProgramStructureUnitId.eq(unit_id))
        .exec(db)
        .await?;
    for &course_id in compulsory_ids {
        program_structure_unit_compulsory_course::ActiveModel {
            program_structure_unit_id: Set(unit_id),
            course_id: Set(course_id),
        }
        .insert(db)
        .await?;
    }
    OptionalLinkEntity::delete_many()
        .filter(program_structure_unit_optional_course::Column::ProgramStructureUnitId.eq(unit_id))
        .exec(db)
        .await?;
    for &course_id in optional_ids {
        program_structure_unit_optional_course::ActiveModel {
            program_structure_unit_id: Set(unit_id),
            course_id: Set(course_id),
        }
        .insert(db)
        .await?;
    }
    Ok(())
}

pub async fn edit(
    Cap(state): Cap<ProgramsState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Path(id): Path<i64>,
) -> Response {
    let Some(program) = find_program_admin(&state.db, id, &ctx).await else {
        return Redirect::to("/programs/").into_response();
    };
    let page = ProgramStructureEditPage {
        id: program.id,
        name: program.name().to_string(),
        units: load_structure_units(&state.db, program.id).await,
        is_admin: true,
    };
    html_built_page_or_app_layout(&page, &htmx, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}

pub async fn unit_create_get(
    Cap(state): Cap<ProgramsState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    Path(id): Path<i64>,
    Query(q): Query<ModalFormQuery>,
) -> Response {
    if find_program_admin(&state.db, id, &ctx).await.is_none() {
        return Redirect::to("/programs/").into_response();
    }
    let page = StructureUnitFormPage {
        program_id: id,
        unit_id: 0,
        term_number: 0,
        optional_course_count: 0,
        compulsory_items: Vec::new(),
        optional_items: Vec::new(),
        error: String::new(),
        form_name: q.form_name(),
    };
    html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}

pub async fn unit_create_post(
    Cap(state): Cap<ProgramsState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Path(id): Path<i64>,
    Query(q): Query<ModalFormQuery>,
    HtmlFormBody(form): HtmlFormBody<StructureUnitForm>,
) -> Response {
    if find_program_admin(&state.db, id, &ctx).await.is_none() {
        return Redirect::to("/programs/").into_response();
    }
    let now = Utc::now();
    let model = program_structure_unit::ActiveModel {
        id: Default::default(),
        created_at: Set(Some(now)),
        updated_at: Set(Some(now)),
        deleted_at: Set(None),
        program_id: Set(id),
        term_number: Set(form.term_number),
        optional_course_count: Set(Some(form.optional_course_count)),
    };
    match model.insert(&state.db).await {
        Ok(saved) => {
            if let Err(e) = sync_unit_courses(
                &state.db,
                saved.id,
                &form.compulsory_courses,
                &form.optional_course_selection_pool,
            )
            .await
            {
                let page = StructureUnitFormPage {
                    program_id: id,
                    unit_id: 0,
                    term_number: form.term_number,
                    optional_course_count: form.optional_course_count,
                    compulsory_items: course_items_from_ids(&state.db, &form.compulsory_courses)
                        .await,
                    optional_items: course_items_from_ids(
                        &state.db,
                        &form.optional_course_selection_pool,
                    )
                    .await,
                    error: e.to_string(),
                    form_name: q.form_name(),
                };
                return html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx))
                    .into_response();
            }
            htmx.redirect(&ProgramsStructureEditRouteTag::new(id).url())
        }
        Err(e) => {
            let page = StructureUnitFormPage {
                program_id: id,
                unit_id: 0,
                term_number: form.term_number,
                optional_course_count: form.optional_course_count,
                compulsory_items: course_items_from_ids(&state.db, &form.compulsory_courses).await,
                optional_items: course_items_from_ids(
                    &state.db,
                    &form.optional_course_selection_pool,
                )
                .await,
                error: e.to_string(),
                form_name: q.form_name(),
            };
            html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
        }
    }
}

pub async fn unit_edit_get(
    Cap(state): Cap<ProgramsState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    Path(path): Path<UnitPath>,
    Query(q): Query<ModalFormQuery>,
) -> Response {
    if find_program_admin(&state.db, path.id, &ctx).await.is_none() {
        return Redirect::to("/programs/").into_response();
    }
    let Some(unit) = find_unit(&state.db, path.id, path.unit_id).await else {
        return Redirect::to("/programs/").into_response();
    };
    let units = load_structure_units(&state.db, path.id).await;
    let view = units.into_iter().find(|u| u.id == unit.id);
    let page = StructureUnitFormPage {
        program_id: path.id,
        unit_id: unit.id,
        term_number: unit.term_number,
        optional_course_count: unit.optional_course_count(),
        compulsory_items: view
            .as_ref()
            .map(|u| u.compulsory_items.clone())
            .unwrap_or_default(),
        optional_items: view
            .map(|u| u.optional_items)
            .unwrap_or_default(),
        error: String::new(),
        form_name: q.form_name(),
    };
    html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}

pub async fn unit_update_post(
    Cap(state): Cap<ProgramsState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Path(path): Path<UnitPath>,
    Query(q): Query<ModalFormQuery>,
    HtmlFormBody(form): HtmlFormBody<StructureUnitForm>,
) -> Response {
    if find_program_admin(&state.db, path.id, &ctx).await.is_none() {
        return Redirect::to("/programs/").into_response();
    }
    let Some(existing) = find_unit(&state.db, path.id, path.unit_id).await else {
        return Redirect::to("/programs/").into_response();
    };
    let now = Utc::now();
    let model = program_structure_unit::ActiveModel {
        id: Set(existing.id),
        created_at: Set(existing.created_at),
        updated_at: Set(Some(now)),
        deleted_at: Set(existing.deleted_at),
        program_id: Set(path.id),
        term_number: Set(form.term_number),
        optional_course_count: Set(Some(form.optional_course_count)),
    };
    match model.update(&state.db).await {
        Ok(_) => {
            if let Err(e) = sync_unit_courses(
                &state.db,
                existing.id,
                &form.compulsory_courses,
                &form.optional_course_selection_pool,
            )
            .await
            {
                let page = StructureUnitFormPage {
                    program_id: path.id,
                    unit_id: existing.id,
                    term_number: form.term_number,
                    optional_course_count: form.optional_course_count,
                    compulsory_items: course_items_from_ids(&state.db, &form.compulsory_courses)
                        .await,
                    optional_items: course_items_from_ids(
                        &state.db,
                        &form.optional_course_selection_pool,
                    )
                    .await,
                    error: e.to_string(),
                    form_name: q.form_name(),
                };
                return html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx))
                    .into_response();
            }
            htmx.redirect(&ProgramsStructureEditRouteTag::new(path.id).url())
        }
        Err(e) => {
            let page = StructureUnitFormPage {
                program_id: path.id,
                unit_id: existing.id,
                term_number: form.term_number,
                optional_course_count: form.optional_course_count,
                compulsory_items: course_items_from_ids(&state.db, &form.compulsory_courses).await,
                optional_items: course_items_from_ids(
                    &state.db,
                    &form.optional_course_selection_pool,
                )
                .await,
                error: e.to_string(),
                form_name: q.form_name(),
            };
            html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
        }
    }
}

pub async fn unit_delete_get(
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    Path(path): Path<UnitPath>,
) -> maud::Markup {
    let _ = ctx;
    let page = ConfirmDeletePage {
        modal_uid: StructureUnitDeleteModalKey::ID.to_string(),
        message: "This removes the term from the program structure. Course links for this unit will be cleared.".into(),
        id: path.id,
        unit_id: path.unit_id,
        error: String::new(),
        is_unit: true,
    };
    html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx))
}

pub async fn unit_delete_post(
    Cap(state): Cap<ProgramsState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Path(path): Path<UnitPath>,
) -> Response {
    if find_program_admin(&state.db, path.id, &ctx).await.is_none() {
        return Redirect::to("/programs/").into_response();
    }
    let Some(existing) = find_unit(&state.db, path.id, path.unit_id).await else {
        return Redirect::to("/programs/").into_response();
    };
    let now = Utc::now();
    if let Err(e) = sync_unit_courses(&state.db, existing.id, &[], &[]).await {
        let page = ConfirmDeletePage {
            modal_uid: StructureUnitDeleteModalKey::ID.to_string(),
            message: "This removes the term from the program structure. Course links for this unit will be cleared.".into(),
            id: path.id,
            unit_id: path.unit_id,
            error: e.to_string(),
            is_unit: true,
        };
        return html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx)).into_response();
    }
    let model = program_structure_unit::ActiveModel {
        id: Set(existing.id),
        deleted_at: Set(Some(now)),
        ..Default::default()
    };
    match model.update(&state.db).await {
        Ok(_) => htmx.redirect(&ProgramsStructureEditRouteTag::new(path.id).url()),
        Err(e) => {
            let page = ConfirmDeletePage {
                modal_uid: StructureUnitDeleteModalKey::ID.to_string(),
                message: "This removes the term from the program structure. Course links for this unit will be cleared.".into(),
                id: path.id,
                unit_id: path.unit_id,
                error: e.to_string(),
                is_unit: true,
            };
            html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
        }
    }
}