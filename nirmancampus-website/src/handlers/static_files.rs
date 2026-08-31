use axum::{
    extract::Path as AxumPath,
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::{handlers::stream_vnode, seed::find_static_vnode};
use lariv_rs::{http::Cap, plugins::filesystem::state::FilesystemState};

const STATIC_PREFIX: &str = "/nirman/static/";

pub fn website_static_path(path: &str) -> String {
    format!("{STATIC_PREFIX}{}", path.trim_start_matches('/'))
}

pub async fn serve(Cap(fs): Cap<FilesystemState>, AxumPath(path): AxumPath<String>) -> Response {
    let Some(node) = lariv_rs::web::opt_or_log(
        find_static_vnode(&fs.db, &path).await,
        "find website static vnode",
    ) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    stream_vnode(&fs, &node).await
}
