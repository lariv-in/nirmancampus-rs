use axum::{
    extract::{Multipart, Path, Query},
    http::Uri,
    response::{IntoResponse, Redirect, Response},
};
use sea_orm::{ColumnTrait, Condition, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder};
use serde::Deserialize;

use crate::{
    entities::tblfee::{self, Entity as TblfeeEntity},
    forms::TblfeeUploadForm,
    handlers::forbid_non_admin,
    keys::TblfeeTableKey,
    state::WebsiteState,
    tblfee_xlsx::{parse_tblfee_xlsx, upsert_rows},
    templates::{TblfeeAdminRow, TblfeeDetailPage, TblfeeListPage},
};
use lariv_rs::{
    components::{ObjectList, SharedChromeFolder, SlotCtx},
    html_form::HtmlForm,
    http::Cap,
    plugins::users::middleware::RequireAuth,
    web::{Htmx, html_built_page_or_app_layout},
};
use nirmancampus_common::{is_admin, path_and_query};

const PAGE_SIZE: u32 = 20;
const MAX_UPLOAD_BYTES: usize = lariv_rs::http::REQUEST_BODY_LIMIT_BYTES;

#[derive(Debug, Deserialize, Default)]
pub struct TblfeeListQuery {
    #[serde(default, rename = "Search", alias = "search")]
    pub search: Option<String>,
    #[serde(default)]
    pub page: Option<u32>,
}

fn search_condition(q: &str) -> Condition {
    let mut cond = Condition::any()
        .add(tblfee::Column::AdmSession.contains(q))
        .add(tblfee::Column::AdmYear.contains(q))
        .add(tblfee::Column::Submit.contains(q))
        .add(tblfee::Column::Prog.contains(q))
        .add(tblfee::Column::Enroll.contains(q))
        .add(tblfee::Column::Student.contains(q))
        .add(tblfee::Column::YearSem.contains(q))
        .add(tblfee::Column::Category.contains(q))
        .add(tblfee::Column::Dob.contains(q))
        .add(tblfee::Column::Contact.contains(q))
        .add(tblfee::Column::Deposit.contains(q))
        .add(tblfee::Column::Nsd.contains(q))
        .add(tblfee::Column::Fee.contains(q))
        .add(tblfee::Column::Courses.contains(q))
        .add(tblfee::Column::Remarks.contains(q))
        .add(tblfee::Column::DepositBy.contains(q))
        .add(tblfee::Column::Ts.contains(q))
        .add(tblfee::Column::Medium.contains(q))
        .add(tblfee::Column::MotherName.contains(q))
        .add(tblfee::Column::FatherName.contains(q))
        .add(tblfee::Column::Username.contains(q))
        .add(tblfee::Column::ControlId.contains(q))
        .add(tblfee::Column::Descrepency.contains(q))
        .add(tblfee::Column::University.contains(q))
        .add(tblfee::Column::PaymentMode.contains(q))
        .add(tblfee::Column::TransId.contains(q))
        .add(tblfee::Column::Bank.contains(q))
        .add(tblfee::Column::Rm.contains(q))
        .add(tblfee::Column::IsReconciled.contains(q))
        .add(tblfee::Column::OnlineExported.contains(q));
    if let Ok(id) = q.parse::<i64>() {
        cond = cond.add(tblfee::Column::Id.eq(id));
    }
    cond
}

async fn query_rows(
    db: &sea_orm::DatabaseConnection,
    q: &TblfeeListQuery,
) -> ObjectList<TblfeeAdminRow> {
    let mut query = TblfeeEntity::find();
    if let Some(search) = q
        .search
        .as_ref()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
    {
        query = query.filter(search_condition(&search));
    }
    query = query.order_by_desc(tblfee::Column::Id);
    let page = q.page.unwrap_or(1).max(1);
    let paginator = query.paginate(db, PAGE_SIZE as u64);
    let total = paginator.num_items().await.unwrap_or(0);
    let models = paginator
        .fetch_page((page as u64).saturating_sub(1))
        .await
        .unwrap_or_default();
    let rows = models.into_iter().map(TblfeeAdminRow::from_model).collect();
    ObjectList::from_page(rows, page, PAGE_SIZE, total)
}

fn list_page(
    records: ObjectList<TblfeeAdminRow>,
    q: &TblfeeListQuery,
    uri: &Uri,
    is_admin: bool,
    sync_message: String,
    sync_error: String,
) -> TblfeeListPage {
    TblfeeListPage {
        records,
        filter_search: q.search.clone().unwrap_or_default(),
        path_and_query: path_and_query(uri),
        is_admin,
        sync_message,
        sync_error,
    }
}

pub async fn list(
    Cap(state): Cap<WebsiteState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    uri: Uri,
    Query(q): Query<TblfeeListQuery>,
) -> Response {
    if let Some(resp) = forbid_non_admin(&ctx) {
        return resp;
    }
    let records = query_rows(&state.db, &q).await;
    let page = list_page(
        records,
        &q,
        &uri,
        is_admin(&ctx),
        String::new(),
        String::new(),
    );
    if htmx.targets::<TblfeeTableKey>() {
        return page.render_table().into_response();
    }
    html_built_page_or_app_layout(&page, &htmx, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}

async fn find_row(db: &sea_orm::DatabaseConnection, id: i64) -> Option<tblfee::Model> {
    lariv_rs::web::opt_or_log(TblfeeEntity::find_by_id(id).one(db).await, "db find tblfee")
}

pub async fn detail(
    Cap(state): Cap<WebsiteState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Path(id): Path<i64>,
) -> Response {
    if let Some(resp) = forbid_non_admin(&ctx) {
        return resp;
    }
    let Some(row) = find_row(&state.db, id).await else {
        return Redirect::to("/website/tblfee/").into_response();
    };
    let page = TblfeeDetailPage::from_model(row);
    html_built_page_or_app_layout(&page, &htmx, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}

pub async fn sync(
    Cap(state): Cap<WebsiteState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    uri: Uri,
    Query(q): Query<TblfeeListQuery>,
    multipart: Multipart,
) -> Response {
    if let Some(resp) = forbid_non_admin(&ctx) {
        return resp;
    }
    let mut sync_message = String::new();
    let mut sync_error = String::new();
    match TblfeeUploadForm::from_multipart(multipart).await {
        Ok(form) => match form.file.into_bytes().await {
            Ok(bytes) => {
                if bytes.len() > MAX_UPLOAD_BYTES {
                    sync_error = "xlsx file too large (max 50 MiB)".into();
                } else if bytes.is_empty() {
                    sync_error = "empty file".into();
                } else {
                    match parse_tblfee_xlsx(&bytes) {
                        Ok((rows, skipped)) => match upsert_rows(&state.db, &rows).await {
                            Ok(report) => {
                                sync_message = format!(
                                    "Synced {} rows ({} new, {} updated, {} skipped without Receipt ID).",
                                    rows.len(),
                                    report.inserted,
                                    report.updated,
                                    skipped + report.skipped
                                );
                            }
                            Err(e) => sync_error = e,
                        },
                        Err(e) => sync_error = e,
                    }
                }
            }
            Err(e) => sync_error = e.to_string(),
        },
        Err(e) => sync_error = e.to_string(),
    }
    let records = query_rows(&state.db, &q).await;
    let page = list_page(records, &q, &uri, is_admin(&ctx), sync_message, sync_error);
    html_built_page_or_app_layout(&page, &htmx, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}
