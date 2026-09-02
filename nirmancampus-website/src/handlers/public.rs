use axum::{
    extract::{Form, Path},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Redirect, Response},
};
use chrono::{Datelike, Utc};
use sea_orm::{ColumnTrait, Condition, EntityTrait, QueryFilter, QueryOrder};
use serde::Deserialize;

use crate::{
    entities::{
        contact_page_settings::{self, Entity as ContactSettingsEntity},
        important_link::{self, Entity as ImportantLinkEntity},
        student_zone_item::{self, Entity as StudentZoneItemEntity},
        student_zone_section::{self, Entity as StudentZoneSectionEntity},
    },
    fee_session::{FeeScope, clear_scope_cookie, scope_from_headers, set_scope_cookie},
    handlers::{media_url, static_files::website_static_path, stream_vnode},
    state::{CONTACT_PAGE_SETTINGS_ID, WebsiteState},
    templates::{
        ContactPage as PublicContactPage, HomeAnnouncement, HomePage, ImportantLinkItem,
        PrivacyPage, ProgramsPage, PublicProgram, PublicShell, StudentZonePage,
        StudentZonePublicItem, StudentZonePublicSection,
    },
};
use lariv_rs::{
    components::{SharedChromeFolder, SlotCtx},
    http::Cap,
    plugins::{
        filesystem::{node, state::FilesystemState},
        users::{middleware::OptionalAuth, state::UsersState},
    },
    web::html_built_page_with_slots,
};
use nirmancampus_announcements::entities::announcement::{self, Entity as AnnouncementEntity};
use nirmancampus_programs::entities::program::{self, Entity as ProgramEntity};
use nirmancampus_studentfees::{
    StudentFeeView, StudentFeesState, contact_matches, find_by_enroll, find_by_id,
};

fn slot_ctx(auth: &OptionalAuth) -> SlotCtx {
    match &auth.0 {
        Some(ctx) => SlotCtx::from_auth(ctx),
        None => SlotCtx::default(),
    }
}

fn shell(auth: &OptionalAuth) -> PublicShell {
    PublicShell {
        is_authenticated: auth.0.is_some(),
        year: Utc::now().year(),
    }
}

pub(crate) fn important_link_public_url(row: &important_link::Model) -> String {
    if row.is_link() {
        row.link().trim().to_string()
    } else {
        format!("/important-links/item/{}/", row.id)
    }
}

pub(crate) async fn load_important_link_items(
    db: &sea_orm::DatabaseConnection,
) -> Vec<ImportantLinkItem> {
    ImportantLinkEntity::find()
        .filter(important_link::Column::DeletedAt.is_null())
        .order_by_asc(important_link::Column::Order)
        .order_by_asc(important_link::Column::Id)
        .all(db)
        .await
        .unwrap_or_default()
        .into_iter()
        .filter_map(|l| {
            let title = l.title.trim().to_string();
            if title.is_empty() {
                return None;
            }
            let url = important_link_public_url(&l);
            if url.trim().is_empty() {
                return None;
            }
            Some(ImportantLinkItem { title, url })
        })
        .collect()
}

async fn load_home_announcements(db: &sea_orm::DatabaseConnection) -> Vec<HomeAnnouncement> {
    let now = Utc::now();
    AnnouncementEntity::find()
        .filter(announcement::Column::DeletedAt.is_null())
        .filter(announcement::Column::ReleaseAt.lte(now))
        .filter(
            Condition::any()
                .add(announcement::Column::ExpiryAt.is_null())
                .add(announcement::Column::ExpiryAt.gt(now)),
        )
        .order_by_desc(announcement::Column::ReleaseAt)
        .all(db)
        .await
        .unwrap_or_default()
        .into_iter()
        .filter_map(|a| {
            let title = a.title.trim().to_string();
            if title.is_empty() {
                return None;
            }
            let date = a
                .release_at
                .map(|d| d.format("%b %-d, %Y").to_string())
                .unwrap_or_default();
            Some(HomeAnnouncement {
                title,
                description_html: lariv_rs::components::render_markdown(a.description()),
                date,
                url: a.url().trim().to_string(),
            })
        })
        .collect()
}

pub async fn home(
    Cap(state): Cap<WebsiteState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    auth: OptionalAuth,
) -> Response {
    let page = HomePage {
        shell: shell(&auth),
        announcements: load_home_announcements(&state.db).await,
        important_links: load_important_link_items(&state.db).await,
        hero_url: website_static_path("images/hero.jpg"),
        director_img_url: website_static_path("images/kansalfoundationwpic.jpeg"),
    };
    html_built_page_with_slots(&page, &chrome, &slot_ctx(&auth)).into_response()
}

pub async fn programs(
    Cap(state): Cap<WebsiteState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    auth: OptionalAuth,
) -> Response {
    let programs = ProgramEntity::find()
        .filter(program::Column::DeletedAt.is_null())
        .order_by_asc(program::Column::Name)
        .order_by_asc(program::Column::Code)
        .all(&state.db)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|p| PublicProgram {
            name: p.name().to_string(),
            code: p.code().to_string(),
            description: p.description().to_string(),
            university: p.university.clone(),
        })
        .collect();
    let page = ProgramsPage {
        shell: shell(&auth),
        programs,
    };
    html_built_page_with_slots(&page, &chrome, &slot_ctx(&auth)).into_response()
}

pub async fn contact(
    Cap(state): Cap<WebsiteState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    auth: OptionalAuth,
) -> Response {
    let settings = ContactSettingsEntity::find_by_id(CONTACT_PAGE_SETTINGS_ID)
        .filter(contact_page_settings::Column::DeletedAt.is_null())
        .one(&state.db)
        .await
        .ok()
        .flatten();
    let has_committees = settings
        .as_ref()
        .and_then(|s| s.essential_committees_list_file_id)
        .filter(|id| *id > 0)
        .is_some();
    let page = PublicContactPage {
        shell: shell(&auth),
        essential_committees_url: if has_committees {
            "/contact-us/essential-committees-list/".into()
        } else {
            String::new()
        },
    };
    html_built_page_with_slots(&page, &chrome, &slot_ctx(&auth)).into_response()
}

pub async fn privacy(Cap(chrome): Cap<SharedChromeFolder>, auth: OptionalAuth) -> Response {
    let page = PrivacyPage {
        shell: shell(&auth),
    };
    html_built_page_with_slots(&page, &chrome, &slot_ctx(&auth)).into_response()
}

async fn load_student_zone_sections(
    db: &sea_orm::DatabaseConnection,
) -> Vec<StudentZonePublicSection> {
    let sections = StudentZoneSectionEntity::find()
        .filter(student_zone_section::Column::DeletedAt.is_null())
        .order_by_asc(student_zone_section::Column::Order)
        .order_by_asc(student_zone_section::Column::Id)
        .all(db)
        .await
        .unwrap_or_default();
    let mut public_sections = Vec::with_capacity(sections.len());
    for s in sections {
        let items = StudentZoneItemEntity::find()
            .filter(student_zone_item::Column::DeletedAt.is_null())
            .filter(student_zone_item::Column::StudentZoneSectionId.eq(s.id))
            .order_by_asc(student_zone_item::Column::Id)
            .all(db)
            .await
            .unwrap_or_default()
            .into_iter()
            .map(|item| StudentZonePublicItem {
                title: item.title.clone(),
                url: format!("/students-zone/item/{}/", item.id),
            })
            .collect();
        public_sections.push(StudentZonePublicSection {
            title: s.title.clone(),
            items,
        });
    }
    public_sections
}

async fn load_fee_records_for_scope(
    fees: &StudentFeesState,
    scope: &FeeScope,
) -> Vec<StudentFeeView> {
    let models = match scope {
        FeeScope::Receipt(id) => find_by_id(fees, *id).await.into_iter().collect(),
        FeeScope::Enroll(enroll) => find_by_enroll(fees, enroll).await,
    };
    models.iter().map(StudentFeeView::from_model).collect()
}

async fn render_student_zone(
    state: &WebsiteState,
    fees: &StudentFeesState,
    chrome: &SharedChromeFolder,
    auth: &OptionalAuth,
    headers: &HeaderMap,
    users: &UsersState,
    login_error: String,
    userid: String,
) -> Response {
    let sections = load_student_zone_sections(&state.db).await;
    let scope = scope_from_headers(headers, users.signing_key.as_slice());
    let records = if let Some(scope) = &scope {
        load_fee_records_for_scope(fees, scope).await
    } else {
        Vec::new()
    };
    let page = StudentZonePage {
        shell: shell(auth),
        sections,
        login_error,
        userid,
        records,
        logged_in: scope.is_some(),
    };
    html_built_page_with_slots(&page, chrome, &slot_ctx(auth)).into_response()
}

pub async fn student_zone(
    Cap(state): Cap<WebsiteState>,
    Cap(fees): Cap<StudentFeesState>,
    Cap(users): Cap<UsersState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    auth: OptionalAuth,
    headers: HeaderMap,
) -> Response {
    render_student_zone(
        &state,
        &fees,
        &chrome,
        &auth,
        &headers,
        &users,
        String::new(),
        String::new(),
    )
    .await
}

#[derive(Debug, Deserialize, Default)]
pub struct FeeLoginForm {
    #[serde(default)]
    pub userid: String,
    #[serde(default)]
    pub password: String,
}

const LOGIN_FAILED: &str = "Mobile number or password did not match any record.";

async fn resolve_login(
    fees: &StudentFeesState,
    userid: &str,
    password: &str,
) -> Option<FeeScope> {
    let userid = userid.trim();
    let password = password.trim();
    if userid.is_empty() || password.is_empty() {
        return None;
    }
    if let Ok(id) = password.parse::<i64>() {
        if id > 0
            && let Some(row) = find_by_id(fees, id).await
            && contact_matches(row.contact.as_deref().unwrap_or(""), userid)
        {
            return Some(FeeScope::Receipt(id));
        }
    }
    let enroll_rows = find_by_enroll(fees, password).await;
    if enroll_rows
        .iter()
        .any(|row| contact_matches(row.contact.as_deref().unwrap_or(""), userid))
    {
        return Some(FeeScope::Enroll(password.to_string()));
    }
    None
}

pub async fn student_zone_login(
    Cap(state): Cap<WebsiteState>,
    Cap(fees): Cap<StudentFeesState>,
    Cap(users): Cap<UsersState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    auth: OptionalAuth,
    headers: HeaderMap,
    Form(form): Form<FeeLoginForm>,
) -> Response {
    let Some(scope) = resolve_login(&fees, &form.userid, &form.password).await else {
        return render_student_zone(
            &state,
            &fees,
            &chrome,
            &auth,
            &headers,
            &users,
            LOGIN_FAILED.into(),
            form.userid,
        )
        .await;
    };
    let mut response = Redirect::to("/students-zone/").into_response();
    set_scope_cookie(
        response.headers_mut(),
        &scope,
        users.signing_key.as_slice(),
        &headers,
    );
    response
}

pub async fn student_zone_logout(headers: HeaderMap) -> Response {
    let mut response = Redirect::to("/students-zone/").into_response();
    clear_scope_cookie(response.headers_mut(), &headers);
    response
}

async fn redirect_or_media(is_link: bool, link: &str, file_id: Option<i64>) -> Response {
    if is_link {
        let dest = link.trim();
        if dest.is_empty() {
            return StatusCode::NOT_FOUND.into_response();
        }
        return Redirect::to(dest).into_response();
    }
    let Some(id) = file_id.filter(|id| *id > 0) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    Redirect::to(&media_url(id)).into_response()
}

pub async fn student_zone_item(Cap(state): Cap<WebsiteState>, Path(id): Path<i64>) -> Response {
    let Some(item) = lariv_rs::web::opt_or_log(
        StudentZoneItemEntity::find_by_id(id)
            .filter(student_zone_item::Column::DeletedAt.is_null())
            .one(&state.db)
            .await,
        "db find student zone item",
    ) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    redirect_or_media(item.is_link(), item.link(), item.file_id).await
}

pub async fn important_link_item(Cap(state): Cap<WebsiteState>, Path(id): Path<i64>) -> Response {
    let Some(item) = lariv_rs::web::opt_or_log(
        ImportantLinkEntity::find_by_id(id)
            .filter(important_link::Column::DeletedAt.is_null())
            .one(&state.db)
            .await,
        "db find important link",
    ) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    redirect_or_media(item.is_link(), item.link(), item.file_id).await
}

pub async fn essential_committees(Cap(state): Cap<WebsiteState>) -> Response {
    let Some(settings) = lariv_rs::web::opt_or_log(
        ContactSettingsEntity::find_by_id(CONTACT_PAGE_SETTINGS_ID)
            .filter(contact_page_settings::Column::DeletedAt.is_null())
            .one(&state.db)
            .await,
        "db find contact page settings",
    ) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let Some(file_id) = settings
        .essential_committees_list_file_id
        .filter(|id| *id > 0)
    else {
        return StatusCode::NOT_FOUND.into_response();
    };
    Redirect::to(&media_url(file_id)).into_response()
}

pub async fn media(Cap(fs): Cap<FilesystemState>, Path(id): Path<i64>) -> Response {
    let Some(n) = lariv_rs::web::opt_or_log(node::get_by_id(&fs.db, id).await, "get node by id")
    else {
        return StatusCode::NOT_FOUND.into_response();
    };
    stream_vnode(&fs, &n).await
}
