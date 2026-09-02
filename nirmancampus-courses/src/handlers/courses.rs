use axum::{
    extract::{Path, Query},
    http::Uri,
    response::{IntoResponse, Redirect, Response},
};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, sea_query::Expr,
};
use serde::Deserialize;

use crate::{
    course_detail_related::related_sections_html,
    entities::course::{self, Entity as CourseEntity},
    forms::CourseForm,
    keys::{
        CourseCreateModalKey, CourseDeleteModalKey, CourseEditModalKey, CourseMultiSelectModalKey,
        CourseMultiSelectTableKey, CourseSelectModalKey, CourseSelectTableKey, CourseTableKey,
    },
    routes::CoursesDetailRouteTag,
    state::CoursesState,
    templates::{
        ConfirmDeletePage, CourseDetailPage, CourseFormPage, CourseListPage, CourseMultiSelectPage,
        CourseRow, CourseSelectPage,
    },
};
use lariv_rs::{
    components::{ObjectList, SharedChromeFolder, SlotCtx, SwapKey},
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
use nirmancampus_common::{is_admin, is_student, path_and_query};

const PAGE_SIZE: u32 = 20;

#[derive(Debug, Deserialize, Default)]
pub struct CourseListQuery {
    #[serde(default, rename = "Name", alias = "name")]
    pub name: Option<String>,
    #[serde(default, rename = "Code", alias = "code")]
    pub code: Option<String>,
    #[serde(default, rename = "CourseType", alias = "course_type")]
    pub course_type: Option<String>,
    #[serde(default)]
    pub page: Option<u32>,
    #[serde(default)]
    pub sort: Option<String>,
    #[serde(default)]
    pub pool_course_ids: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct CourseSelectQuery {
    #[serde(flatten)]
    pub filter: CourseListQuery,
    #[serde(default)]
    pub target_input: Option<String>,
}

fn scope_courses(
    query: sea_orm::Select<CourseEntity>,
    auth: &lariv_rs::plugins::users::state::AuthContext,
) -> sea_orm::Select<CourseEntity> {
    if is_admin(auth) {
        return query;
    }
    if is_student(auth) {
        let email = auth.user.email.clone();
        return query.filter(Expr::cust_with_values(
            "courses.id IN ( \
                SELECT academic_record_compulsory_courses.course_id \
                FROM academic_record_compulsory_courses \
                JOIN academic_records ON academic_records.id = academic_record_compulsory_courses.academic_record_id \
                  AND academic_records.deleted_at IS NULL \
                JOIN students ON students.id = academic_records.student_id AND students.email = ? \
                UNION \
                SELECT academic_record_optional_courses.course_id \
                FROM academic_record_optional_courses \
                JOIN academic_records ON academic_records.id = academic_record_optional_courses.academic_record_id \
                  AND academic_records.deleted_at IS NULL \
                JOIN students ON students.id = academic_records.student_id AND students.email = ? \
            )",
            [email.clone(), email],
        ));
    }
    query.filter(Expr::cust("1 = 0"))
}

async fn find_course_scoped(
    db: &sea_orm::DatabaseConnection,
    id: i64,
    auth: &lariv_rs::plugins::users::state::AuthContext,
) -> Option<course::Model> {
    let query = CourseEntity::find_by_id(id).filter(course::Column::DeletedAt.is_null());
    let query = scope_courses(query, auth);
    lariv_rs::web::opt_or_log(query.one(db).await, "db find one")
}

async fn query_courses(
    db: &sea_orm::DatabaseConnection,
    q: &CourseListQuery,
    auth: &lariv_rs::plugins::users::state::AuthContext,
) -> ObjectList<CourseRow> {
    let mut query = CourseEntity::find().filter(course::Column::DeletedAt.is_null());
    query = scope_courses(query, auth);

    if let Some(name) = q.name.as_ref().filter(|n| !n.is_empty()) {
        query = query.filter(course::Column::Name.contains(name));
    }
    if let Some(code) = q.code.as_ref().filter(|n| !n.is_empty()) {
        query = query.filter(course::Column::Code.contains(code));
    }
    if let Some(course_type) = q.course_type.as_ref().filter(|n| !n.is_empty()) {
        query = query.filter(course::Column::CourseType.contains(course_type));
    }
    if let Some(raw) = q.pool_course_ids.as_ref() {
        let mut ids = Vec::new();
        let mut invalid = false;
        for p in raw.split(',') {
            let p = p.trim();
            if p.is_empty() {
                continue;
            }
            match p.parse::<i64>() {
                Ok(n) if n > 0 => ids.push(n),
                _ => {
                    invalid = true;
                    break;
                }
            }
        }
        if invalid || ids.is_empty() {
            query = query.filter(Expr::cust("1 = 0"));
        } else {
            query = query.filter(course::Column::Id.is_in(ids));
        }
    }

    let sort = q.sort.as_deref().unwrap_or("").trim();
    query = match sort {
        s if s.eq_ignore_ascii_case("Name DESC") => query.order_by_desc(course::Column::Name),
        s if s.eq_ignore_ascii_case("Name ASC") || s.eq_ignore_ascii_case("Name") => {
            query.order_by_asc(course::Column::Name)
        }
        s if s.eq_ignore_ascii_case("Code DESC") => query.order_by_desc(course::Column::Code),
        s if s.eq_ignore_ascii_case("Code ASC") || s.eq_ignore_ascii_case("Code") => {
            query.order_by_asc(course::Column::Code)
        }
        s if s.eq_ignore_ascii_case("Fee DESC") => query.order_by_desc(course::Column::Fee),
        s if s.eq_ignore_ascii_case("Fee ASC") || s.eq_ignore_ascii_case("Fee") => {
            query.order_by_asc(course::Column::Fee)
        }
        _ => query.order_by_desc(course::Column::Id),
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
        .map(|c| CourseRow {
            id: c.id,
            name: c.name().to_string(),
            code: c.code().to_string(),
            course_type: c.course_type.clone(),
            fee: c.fee,
            is_active: c.is_active(),
        })
        .collect();
    ObjectList::from_page(rows, page, PAGE_SIZE, total)
}

pub async fn list(
    Cap(state): Cap<CoursesState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    uri: Uri,
    Query(q): Query<CourseListQuery>,
) -> maud::Markup {
    let courses = query_courses(&state.db, &q, &ctx).await;
    let page = CourseListPage {
        courses,
        filter_name: q.name.clone().unwrap_or_default(),
        filter_code: q.code.clone().unwrap_or_default(),
        filter_course_type: q.course_type.clone().unwrap_or_default(),
        path_and_query: path_and_query(&uri),
        sort: q.sort.clone().unwrap_or_default(),
        is_admin: is_admin(&ctx),
    };
    if htmx.targets::<CourseTableKey>() {
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
    Cap(state): Cap<CoursesState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Path(id): Path<i64>,
) -> Response {
    let Some(course) = find_course_scoped(&state.db, id, &ctx).await else {
        return Redirect::to("/courses/").into_response();
    };
    let related_sections = related_sections_html(&state.db, course.id, &ctx).await;
    let page = CourseDetailPage {
        id: course.id,
        name: course.name().to_string(),
        code: course.code().to_string(),
        description: course.description().to_string(),
        is_active: course.is_active(),
        course_type: course.course_type,
        fee: course.fee,
        is_admin: is_admin(&ctx),
        related_sections,
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
    let page = CourseFormPage {
        id: 0,
        name: String::new(),
        code: String::new(),
        course_type: String::new(),
        description: String::new(),
        fee: 0,
        is_active: true,
        error: String::new(),
        form_name: q.form_name(),
        refresh_table: q.refresh_table(),
        target_input: q.target_input(),
    };
    html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx))
}

pub async fn create_post(
    Cap(state): Cap<CoursesState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Query(q): Query<ModalFormQuery>,
    HtmlFormBody(form): HtmlFormBody<CourseForm>,
) -> Response {
    if !is_admin(&ctx) {
        return Redirect::to("/courses/").into_response();
    }
    let now = Utc::now();
    let model = course::ActiveModel {
        id: Default::default(),
        created_at: Set(Some(now)),
        updated_at: Set(Some(now)),
        deleted_at: Set(None),
        name: Set(Some(form.name.clone())),
        is_active: Set(Some(form.is_active)),
        code: Set(Some(form.code.clone())),
        course_type: Set(form.course_type.clone()),
        description: Set(nirmancampus_common::optional_string(&form.description)),
        fee: Set(form.fee),
    };
    match model.insert(&state.db).await {
        Ok(saved) => respond_create_modal_done_fk::<CourseCreateModalKey>(
            &htmx,
            &q.refresh_table(),
            &CoursesDetailRouteTag::new(saved.id).url(),
            saved.id,
            saved.name(),
            &q.target_input(),
        ),
        Err(e) => {
            let page = CourseFormPage {
                id: 0,
                name: form.name,
                code: form.code,
                course_type: form.course_type,
                description: form.description,
                fee: form.fee,
                is_active: form.is_active,
                error: e.to_string(),
                form_name: q.form_name(),
                refresh_table: q.refresh_table(),
                target_input: q.target_input(),
            };
            html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
        }
    }
}

pub async fn edit_get(
    Cap(state): Cap<CoursesState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    Path(id): Path<i64>,
    Query(q): Query<ModalFormQuery>,
) -> Response {
    if !is_admin(&ctx) {
        return Redirect::to("/courses/").into_response();
    }
    let Some(course) = find_course_scoped(&state.db, id, &ctx).await else {
        return Redirect::to("/courses/").into_response();
    };
    let page = CourseFormPage {
        id: course.id,
        name: course.name().to_string(),
        code: course.code().to_string(),
        description: course.description().to_string(),
        is_active: course.is_active(),
        course_type: course.course_type,
        fee: course.fee,
        error: String::new(),
        form_name: q.form_name(),
        refresh_table: String::new(),
        target_input: String::new(),
    };
    html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}

pub async fn edit_post(
    Cap(state): Cap<CoursesState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Path(id): Path<i64>,
    Query(q): Query<ModalFormQuery>,
    HtmlFormBody(form): HtmlFormBody<CourseForm>,
) -> Response {
    if !is_admin(&ctx) {
        return Redirect::to("/courses/").into_response();
    }
    let Some(existing) = find_course_scoped(&state.db, id, &ctx).await else {
        return Redirect::to("/courses/").into_response();
    };
    let now = Utc::now();
    let model = course::ActiveModel {
        id: Set(existing.id),
        created_at: Set(existing.created_at),
        updated_at: Set(Some(now)),
        deleted_at: Set(existing.deleted_at),
        name: Set(Some(form.name.clone())),
        is_active: Set(Some(form.is_active)),
        code: Set(Some(form.code.clone())),
        course_type: Set(form.course_type.clone()),
        description: Set(nirmancampus_common::optional_string(&form.description)),
        fee: Set(form.fee),
    };
    match model.update(&state.db).await {
        Ok(_) => respond_edit_modal_done::<CourseEditModalKey>(
            &htmx,
            &CoursesDetailRouteTag::new(id).url(),
        ),
        Err(e) => {
            let page = CourseFormPage {
                id,
                name: form.name,
                code: form.code,
                course_type: form.course_type,
                description: form.description,
                fee: form.fee,
                is_active: form.is_active,
                error: e.to_string(),
                form_name: q.form_name(),
                refresh_table: String::new(),
                target_input: String::new(),
            };
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
        modal_uid: CourseDeleteModalKey::ID.to_string(),
        message: format!("Delete course #{id}?"),
        form_name: "courses.CourseDeleteForm".into(),
        id,
        error: String::new(),
    };
    html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx))
}

pub async fn delete_post(
    Cap(state): Cap<CoursesState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    Path(id): Path<i64>,
) -> Response {
    if !is_admin(&ctx) {
        return Redirect::to("/courses/").into_response();
    }
    let Some(existing) = find_course_scoped(&state.db, id, &ctx).await else {
        return Redirect::to("/courses/").into_response();
    };
    let now = Utc::now();
    let model = course::ActiveModel {
        id: Set(existing.id),
        deleted_at: Set(Some(now)),
        ..Default::default()
    };
    match model.update(&state.db).await {
        Ok(_) => Redirect::to("/courses/").into_response(),
        Err(e) => {
            tracing::error!(error = %e, id, "failed to delete course");
            let page = ConfirmDeletePage {
                modal_uid: CourseDeleteModalKey::ID.to_string(),
                message: format!("Delete course #{id}?"),
                form_name: "courses.CourseDeleteForm".into(),
                id,
                error: e.to_string(),
            };
            html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
        }
    }
}

pub async fn select(
    Cap(state): Cap<CoursesState>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    uri: Uri,
    Query(q): Query<CourseSelectQuery>,
) -> maud::Markup {
    let courses = query_courses(&state.db, &q.filter, &ctx).await;
    let page = CourseSelectPage {
        courses,
        filter_name: q.filter.name.clone().unwrap_or_default(),
        filter_code: q.filter.code.clone().unwrap_or_default(),
        path_and_query: path_and_query(&uri),
        sort: q.filter.sort.clone().unwrap_or_default(),
        target_input: q.target_input.unwrap_or_default(),
        is_admin: is_admin(&ctx),
    };
    respond_picker_select::<CourseSelectTableKey, CourseSelectModalKey, _>(&htmx, &page)
}

pub async fn multi_select(
    Cap(state): Cap<CoursesState>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    uri: Uri,
    Query(q): Query<CourseSelectQuery>,
) -> maud::Markup {
    let courses = query_courses(&state.db, &q.filter, &ctx).await;
    let page = CourseMultiSelectPage {
        courses,
        filter_name: q.filter.name.clone().unwrap_or_default(),
        filter_code: q.filter.code.clone().unwrap_or_default(),
        path_and_query: path_and_query(&uri),
        sort: q.filter.sort.clone().unwrap_or_default(),
        target_input: q.target_input.unwrap_or_else(|| "Courses".into()),
        pool_course_ids: q.filter.pool_course_ids.clone().unwrap_or_default(),
        is_admin: is_admin(&ctx),
    };
    respond_picker_select::<CourseMultiSelectTableKey, CourseMultiSelectModalKey, _>(&htmx, &page)
}
