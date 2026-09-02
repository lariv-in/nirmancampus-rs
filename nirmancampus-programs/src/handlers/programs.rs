use axum::{
    extract::{Path, Query},
    http::Uri,
    response::{IntoResponse, Redirect, Response},
};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait,
    PaginatorTrait, QueryFilter, QueryOrder, sea_query::Expr,
};
use serde::Deserialize;

use crate::{
    entities::{
        program::{self, Entity as ProgramEntity},
        program_media::{self, Entity as ProgramMediaEntity},
        program_program_media::{self, Entity as ProgramMediaLinkEntity},
        program_structure_unit::{self, Entity as StructureUnitEntity},
        program_structure_unit_compulsory_course::{
            self, Entity as CompulsoryLinkEntity,
        },
        program_structure_unit_optional_course::{self, Entity as OptionalLinkEntity},
    },
    forms::ProgramForm,
    handlers::{course_codes_label, course_items_from_ids},
    keys::{
        ProgramCreateModalKey, ProgramDeleteModalKey, ProgramEditModalKey,
        ProgramMediaMultiSelectModalKey, ProgramMediaMultiSelectTableKey, ProgramSelectModalKey,
        ProgramSelectTableKey, ProgramTableKey,
    },
    routes::ProgramsDetailRouteTag,
    state::ProgramsState,
    templates::{
        ConfirmDeletePage, ProgramDetailPage, ProgramFormPage, ProgramListPage,
        ProgramMediaMultiSelectPage, ProgramMediaRow, ProgramRow, ProgramSelectPage,
        ProgramStructureUnitView, program_display_label,
    },
};
use lariv_rs::{
    components::{ManyToManyItem, ObjectList, SharedChromeFolder, SlotCtx, SwapKey},
    html_form::HtmlFormBody,
    http::Cap,
    picker::respond_picker_select,
    plugins::users::middleware::RequireAuth,
    template::RenderAppPane,
    web::{
        html_built_page_or_app_layout, html_built_page_with_slots, respond_create_modal_done_fk,
        respond_edit_modal_done, Htmx, ModalFormQuery,
    },
};
use nirmancampus_common::{is_admin, is_student, is_unassigned, path_and_query};

const PAGE_SIZE: u32 = 20;

#[derive(Debug, Deserialize, Default)]
pub struct ProgramListQuery {
    #[serde(default, rename = "Name", alias = "name")]
    pub name: Option<String>,
    #[serde(default, rename = "Code", alias = "code")]
    pub code: Option<String>,
    #[serde(default, rename = "University", alias = "university")]
    pub university: Option<String>,
    #[serde(default, rename = "ProgramType", alias = "program_type")]
    pub program_type: Option<String>,
    #[serde(default)]
    pub page: Option<u32>,
    #[serde(default)]
    pub sort: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct ProgramSelectQuery {
    #[serde(flatten)]
    pub filter: ProgramListQuery,
    #[serde(default)]
    pub target_input: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct ProgramMediaSelectQuery {
    #[serde(default)]
    pub page: Option<u32>,
    #[serde(default)]
    pub target_input: Option<String>,
}

pub(crate) fn scope_programs(
    query: sea_orm::Select<ProgramEntity>,
    auth: &lariv_rs::plugins::users::state::AuthContext,
) -> sea_orm::Select<ProgramEntity> {
    if is_admin(auth) || is_unassigned(auth) {
        return query;
    }
    if is_student(auth) {
        let email = auth.user.email.clone();
        return query.filter(Expr::cust_with_values(
            "programs.id IN ( \
                SELECT academic_records.program_id \
                FROM academic_records \
                JOIN students ON students.id = academic_records.student_id AND students.email = ? \
                WHERE academic_records.deleted_at IS NULL \
            )",
            [email],
        ));
    }
    query.filter(Expr::cust("1 = 0"))
}

async fn find_program_scoped(
    db: &DatabaseConnection,
    id: i64,
    auth: &lariv_rs::plugins::users::state::AuthContext,
) -> Option<program::Model> {
    let query = ProgramEntity::find_by_id(id).filter(program::Column::DeletedAt.is_null());
    let query = scope_programs(query, auth);
    lariv_rs::web::opt_or_log(query.one(db).await, "db find one")
}

async fn query_programs(
    db: &DatabaseConnection,
    q: &ProgramListQuery,
    auth: &lariv_rs::plugins::users::state::AuthContext,
) -> ObjectList<ProgramRow> {
    let mut query = ProgramEntity::find().filter(program::Column::DeletedAt.is_null());
    query = scope_programs(query, auth);

    if let Some(name) = q.name.as_ref().filter(|n| !n.is_empty()) {
        query = query.filter(program::Column::Name.contains(name));
    }
    if let Some(code) = q.code.as_ref().filter(|n| !n.is_empty()) {
        query = query.filter(program::Column::Code.contains(code));
    }
    if let Some(university) = q.university.as_ref().filter(|n| !n.is_empty()) {
        query = query.filter(program::Column::University.eq(university));
    }
    if let Some(program_type) = q.program_type.as_ref().filter(|n| !n.is_empty()) {
        query = query.filter(program::Column::ProgramType.eq(program_type));
    }

    let sort = q.sort.as_deref().unwrap_or("").trim();
    query = match sort {
        s if s.eq_ignore_ascii_case("Name DESC") => query.order_by_desc(program::Column::Name),
        s if s.eq_ignore_ascii_case("Name ASC") || s.eq_ignore_ascii_case("Name") => {
            query.order_by_asc(program::Column::Name)
        }
        s if s.eq_ignore_ascii_case("Code DESC") => query.order_by_desc(program::Column::Code),
        s if s.eq_ignore_ascii_case("Code ASC") || s.eq_ignore_ascii_case("Code") => {
            query.order_by_asc(program::Column::Code)
        }
        s if s.eq_ignore_ascii_case("Fee DESC") => query.order_by_desc(program::Column::Fee),
        s if s.eq_ignore_ascii_case("Fee ASC") || s.eq_ignore_ascii_case("Fee") => {
            query.order_by_asc(program::Column::Fee)
        }
        _ => query.order_by_desc(program::Column::Id),
    };

    let page = q.page.unwrap_or(1).max(1);
    let paginator = query.paginate(db, PAGE_SIZE as u64);
    let total = paginator.num_items().await.unwrap_or(0);
    let models = paginator
        .fetch_page((page as u64).saturating_sub(1))
        .await
        .unwrap_or_default();
    let rows = models
        .into_iter()
        .map(|p| ProgramRow {
            id: p.id,
            name: p.name().to_string(),
            code: p.code().to_string(),
            university: p.university.clone(),
            program_type: p.program_type.clone(),
            fee: p.fee,
            description: p.description().to_string(),
            display_label: program_display_label(p.name(), &p.university),
        })
        .collect();
    ObjectList::from_page(rows, page, PAGE_SIZE, total)
}

async fn media_items_for_program(db: &DatabaseConnection, program_id: i64) -> Vec<ManyToManyItem> {
    let links = ProgramMediaLinkEntity::find()
        .filter(program_program_media::Column::ProgramId.eq(program_id))
        .all(db)
        .await
        .unwrap_or_default();
    let ids: Vec<i64> = links.into_iter().map(|l| l.program_media_id).collect();
    media_items_from_ids(db, &ids).await
}

async fn media_items_from_ids(db: &DatabaseConnection, ids: &[i64]) -> Vec<ManyToManyItem> {
    if ids.is_empty() {
        return Vec::new();
    }
    let rows = ProgramMediaEntity::find()
        .filter(program_media::Column::DeletedAt.is_null())
        .filter(program_media::Column::Id.is_in(ids.to_vec()))
        .all(db)
        .await
        .unwrap_or_default();
    ids.iter()
        .filter_map(|id| {
            rows.iter()
                .find(|m| m.id == *id)
                .map(|m| ManyToManyItem::new(m.id.to_string(), m.language.clone()))
        })
        .collect()
}

async fn sync_program_media(
    db: &DatabaseConnection,
    program_id: i64,
    media_ids: &[i64],
) -> Result<(), sea_orm::DbErr> {
    ProgramMediaLinkEntity::delete_many()
        .filter(program_program_media::Column::ProgramId.eq(program_id))
        .exec(db)
        .await?;
    for &program_media_id in media_ids {
        program_program_media::ActiveModel {
            program_id: Set(program_id),
            program_media_id: Set(program_media_id),
        }
        .insert(db)
        .await?;
    }
    Ok(())
}

pub(crate) async fn load_structure_units(
    db: &DatabaseConnection,
    program_id: i64,
) -> Vec<ProgramStructureUnitView> {
    let units = StructureUnitEntity::find()
        .filter(program_structure_unit::Column::ProgramId.eq(program_id))
        .filter(program_structure_unit::Column::DeletedAt.is_null())
        .order_by_asc(program_structure_unit::Column::TermNumber)
        .all(db)
        .await
        .unwrap_or_default();
    let mut out = Vec::with_capacity(units.len());
    for unit in units {
        let compulsory_ids = unit_course_ids(db, unit.id, true).await;
        let optional_ids = unit_course_ids(db, unit.id, false).await;
        out.push(ProgramStructureUnitView {
            id: unit.id,
            term_number: unit.term_number,
            optional_course_count: unit.optional_course_count(),
            compulsory_label: course_codes_label(db, &compulsory_ids).await,
            optional_label: course_codes_label(db, &optional_ids).await,
            compulsory_items: course_items_from_ids(db, &compulsory_ids).await,
            optional_items: course_items_from_ids(db, &optional_ids).await,
        });
    }
    out
}

async fn unit_course_ids(db: &DatabaseConnection, unit_id: i64, compulsory: bool) -> Vec<i64> {
    if compulsory {
        CompulsoryLinkEntity::find()
            .filter(program_structure_unit_compulsory_course::Column::ProgramStructureUnitId.eq(unit_id))
            .all(db)
            .await
            .unwrap_or_default()
            .into_iter()
            .map(|l| l.course_id)
            .collect()
    } else {
        OptionalLinkEntity::find()
            .filter(program_structure_unit_optional_course::Column::ProgramStructureUnitId.eq(unit_id))
            .all(db)
            .await
            .unwrap_or_default()
            .into_iter()
            .map(|l| l.course_id)
            .collect()
    }
}

fn form_page(
    id: i64,
    form: &ProgramForm,
    media_items: Vec<ManyToManyItem>,
    error: String,
    q: &ModalFormQuery,
) -> ProgramFormPage {
    ProgramFormPage {
        id,
        name: form.name.clone(),
        code: form.code.clone(),
        description: form.description.clone(),
        university: form.university.clone(),
        program_type: form.program_type.clone(),
        admission_sessions: form.admission_sessions.clone(),
        term_type: form.term_type.clone(),
        fee: form.fee,
        media_items,
        error,
        form_name: q.form_name(),
        refresh_table: q.refresh_table(),
        target_input: q.target_input(),
    }
}

pub async fn list(
    Cap(state): Cap<ProgramsState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    uri: Uri,
    Query(q): Query<ProgramListQuery>,
) -> maud::Markup {
    let programs = query_programs(&state.db, &q, &ctx).await;
    let page = ProgramListPage {
        programs,
        filter_name: q.name.clone().unwrap_or_default(),
        filter_code: q.code.clone().unwrap_or_default(),
        filter_university: q.university.clone().unwrap_or_default(),
        filter_program_type: q.program_type.clone().unwrap_or_default(),
        path_and_query: path_and_query(&uri),
        sort: q.sort.clone().unwrap_or_default(),
        is_admin: is_admin(&ctx),
    };
    if htmx.targets::<ProgramTableKey>() {
        return page.render_table();
    }
    if htmx.wants_main_content() {
        return page.render_main().into();
    }
    if htmx.wants_app_layout() {
        return page.render_pane().into();
    }
    html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx))
}

pub async fn detail(
    Cap(state): Cap<ProgramsState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Path(id): Path<i64>,
) -> Response {
    let Some(program) = find_program_scoped(&state.db, id, &ctx).await else {
        return Redirect::to("/programs/").into_response();
    };
    let media_items = media_items_for_program(&state.db, program.id).await;
    let units = load_structure_units(&state.db, program.id).await;
    let page = ProgramDetailPage {
        id: program.id,
        name: program.name().to_string(),
        code: program.code().to_string(),
        description: program.description().to_string(),
        university: program.university,
        program_type: program.program_type,
        admission_sessions: program.admission_sessions,
        term_type: program.term_type,
        fee: program.fee,
        media_items,
        units,
        is_admin: is_admin(&ctx),
    };
    html_built_page_or_app_layout(&page, &htmx, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}

pub async fn create_get(
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    Query(q): Query<ModalFormQuery>,
) -> maud::Markup {
    if !is_admin(&ctx) {
        return maud::html! { p { "Forbidden" } };
    }
    let page = ProgramFormPage {
        id: 0,
        name: String::new(),
        code: String::new(),
        description: String::new(),
        university: String::new(),
        program_type: String::new(),
        admission_sessions: String::new(),
        term_type: String::new(),
        fee: 0,
        media_items: Vec::new(),
        error: String::new(),
        form_name: q.form_name(),
        refresh_table: q.refresh_table(),
        target_input: q.target_input(),
    };
    html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx))
}

pub async fn create_post(
    Cap(state): Cap<ProgramsState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Query(q): Query<ModalFormQuery>,
    HtmlFormBody(form): HtmlFormBody<ProgramForm>,
) -> Response {
    if !is_admin(&ctx) {
        return Redirect::to("/programs/").into_response();
    }
    let now = Utc::now();
    let model = program::ActiveModel {
        id: Default::default(),
        created_at: Set(Some(now)),
        updated_at: Set(Some(now)),
        deleted_at: Set(None),
        name: Set(Some(form.name.clone())),
        code: Set(Some(form.code.clone())),
        description: Set(nirmancampus_common::optional_string(&form.description)),
        university: Set(form.university.clone()),
        program_type: Set(form.program_type.clone()),
        admission_sessions: Set(form.admission_sessions.clone()),
        term_type: Set(form.term_type.clone()),
        fee: Set(form.fee),
    };
    match model.insert(&state.db).await {
        Ok(saved) => {
            if let Err(e) = sync_program_media(&state.db, saved.id, &form.program_media).await {
                let media_items = media_items_from_ids(&state.db, &form.program_media).await;
                let page = form_page(0, &form, media_items, e.to_string(), &q);
                return html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx))
                    .into_response();
            }
            respond_create_modal_done_fk::<ProgramCreateModalKey>(
                &htmx,
                &q.refresh_table(),
                &ProgramsDetailRouteTag::new(saved.id).url(),
                saved.id,
                saved.name(),
                &q.target_input(),
            )
        }
        Err(e) => {
            let media_items = media_items_from_ids(&state.db, &form.program_media).await;
            let page = form_page(0, &form, media_items, e.to_string(), &q);
            html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
        }
    }
}

pub async fn edit_get(
    Cap(state): Cap<ProgramsState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    Path(id): Path<i64>,
    Query(q): Query<ModalFormQuery>,
) -> Response {
    if !is_admin(&ctx) {
        return Redirect::to("/programs/").into_response();
    }
    let Some(program) = find_program_scoped(&state.db, id, &ctx).await else {
        return Redirect::to("/programs/").into_response();
    };
    let page = ProgramFormPage {
        id: program.id,
        name: program.name().to_string(),
        code: program.code().to_string(),
        description: program.description().to_string(),
        university: program.university,
        program_type: program.program_type,
        admission_sessions: program.admission_sessions,
        term_type: program.term_type,
        fee: program.fee,
        media_items: media_items_for_program(&state.db, program.id).await,
        error: String::new(),
        form_name: q.form_name(),
        refresh_table: String::new(),
        target_input: String::new(),
    };
    html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}

pub async fn edit_post(
    Cap(state): Cap<ProgramsState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Path(id): Path<i64>,
    Query(q): Query<ModalFormQuery>,
    HtmlFormBody(form): HtmlFormBody<ProgramForm>,
) -> Response {
    if !is_admin(&ctx) {
        return Redirect::to("/programs/").into_response();
    }
    let Some(existing) = find_program_scoped(&state.db, id, &ctx).await else {
        return Redirect::to("/programs/").into_response();
    };
    let now = Utc::now();
    let model = program::ActiveModel {
        id: Set(existing.id),
        created_at: Set(existing.created_at),
        updated_at: Set(Some(now)),
        deleted_at: Set(existing.deleted_at),
        name: Set(Some(form.name.clone())),
        code: Set(Some(form.code.clone())),
        description: Set(nirmancampus_common::optional_string(&form.description)),
        university: Set(form.university.clone()),
        program_type: Set(form.program_type.clone()),
        admission_sessions: Set(form.admission_sessions.clone()),
        term_type: Set(form.term_type.clone()),
        fee: Set(form.fee),
    };
    match model.update(&state.db).await {
        Ok(_) => {
            if let Err(e) = sync_program_media(&state.db, id, &form.program_media).await {
                let media_items = media_items_from_ids(&state.db, &form.program_media).await;
                let mut page = form_page(id, &form, media_items, e.to_string(), &q);
                page.refresh_table = String::new();
                page.target_input = String::new();
                return html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx))
                    .into_response();
            }
            respond_edit_modal_done::<ProgramEditModalKey>(
                &htmx,
                &ProgramsDetailRouteTag::new(id).url(),
            )
        }
        Err(e) => {
            let media_items = media_items_from_ids(&state.db, &form.program_media).await;
            let mut page = form_page(id, &form, media_items, e.to_string(), &q);
            page.refresh_table = String::new();
            page.target_input = String::new();
            html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
        }
    }
}

pub async fn delete_get(
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    Path(id): Path<i64>,
) -> maud::Markup {
    let page = ConfirmDeletePage {
        modal_uid: ProgramDeleteModalKey::ID.to_string(),
        message: "Are you sure you want to delete this program?".into(),
        id,
        error: String::new(),
        unit_id: 0,
        is_unit: false,
    };
    html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx))
}

pub async fn delete_post(
    Cap(state): Cap<ProgramsState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    Path(id): Path<i64>,
) -> Response {
    if !is_admin(&ctx) {
        return Redirect::to("/programs/").into_response();
    }
    let Some(existing) = find_program_scoped(&state.db, id, &ctx).await else {
        return Redirect::to("/programs/").into_response();
    };
    let now = Utc::now();
    let model = program::ActiveModel {
        id: Set(existing.id),
        deleted_at: Set(Some(now)),
        ..Default::default()
    };
    match model.update(&state.db).await {
        Ok(_) => Redirect::to("/programs/").into_response(),
        Err(e) => {
            tracing::error!(error = %e, id, "failed to delete program");
            let page = ConfirmDeletePage {
                modal_uid: ProgramDeleteModalKey::ID.to_string(),
                message: "Are you sure you want to delete this program?".into(),
                id,
                error: e.to_string(),
                unit_id: 0,
                is_unit: false,
            };
            html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
        }
    }
}

pub async fn select(
    Cap(state): Cap<ProgramsState>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    uri: Uri,
    Query(q): Query<ProgramSelectQuery>,
) -> maud::Markup {
    let programs = query_programs(&state.db, &q.filter, &ctx).await;
    let page = ProgramSelectPage {
        programs,
        filter_name: q.filter.name.clone().unwrap_or_default(),
        filter_code: q.filter.code.clone().unwrap_or_default(),
        filter_university: q.filter.university.clone().unwrap_or_default(),
        filter_program_type: q.filter.program_type.clone().unwrap_or_default(),
        path_and_query: path_and_query(&uri),
        target_input: q.target_input.unwrap_or_default(),
    };
    respond_picker_select::<ProgramSelectTableKey, ProgramSelectModalKey, _>(&htmx, &page)
}

pub async fn media_multi_select(
    Cap(state): Cap<ProgramsState>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    uri: Uri,
    Query(q): Query<ProgramMediaSelectQuery>,
) -> maud::Markup {
    if !is_admin(&ctx) {
        return maud::html! { p { "Forbidden" } };
    }
    let query = ProgramMediaEntity::find()
        .filter(program_media::Column::DeletedAt.is_null())
        .order_by_asc(program_media::Column::Language);
    let page_num = q.page.unwrap_or(1).max(1);
    let paginator = query.paginate(&state.db, PAGE_SIZE as u64);
    let total = paginator.num_items().await.unwrap_or(0);
    let models = paginator
        .fetch_page((page_num as u64).saturating_sub(1))
        .await
        .unwrap_or_default();
    let items = models
        .into_iter()
        .map(|m| ProgramMediaRow {
            id: m.id,
            language: m.language,
        })
        .collect();
    let page = ProgramMediaMultiSelectPage {
        items: ObjectList::from_page(items, page_num, PAGE_SIZE, total),
        path_and_query: path_and_query(&uri),
        target_input: q.target_input.unwrap_or_else(|| "ProgramMedia".into()),
    };
    respond_picker_select::<ProgramMediaMultiSelectTableKey, ProgramMediaMultiSelectModalKey, _>(
        &htmx, &page,
    )
}
