use axum::response::{IntoResponse, Response};

use crate::{handlers::forbid_non_admin, templates::WebsiteLandingPage};
use lariv_rs::{
    components::{SharedChromeFolder, SlotCtx},
    http::Cap,
    plugins::users::middleware::RequireAuth,
    web::{Htmx, html_built_page_or_app_layout},
};

pub async fn landing(
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
) -> Response {
    if let Some(resp) = forbid_non_admin(&ctx) {
        return resp;
    }
    let page = WebsiteLandingPage {};
    html_built_page_or_app_layout(&page, &htmx, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}
