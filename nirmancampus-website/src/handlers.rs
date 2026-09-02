pub mod admin;
pub mod contact_page;
pub mod important_links;
pub mod public;
pub mod static_files;
pub mod student_zone;

use axum::{
    http::{HeaderValue, StatusCode, header},
    response::{IntoResponse, Redirect, Response},
};
use nirmancampus_common::is_admin;
use tokio::io::AsyncReadExt;

use lariv_rs::plugins::filesystem::{entities::VNode, state::FilesystemState};

pub async fn stream_vnode(fs: &FilesystemState, n: &VNode) -> Response {
    if n.is_directory {
        return StatusCode::NOT_FOUND.into_response();
    }
    let path = n.file_path.as_deref().unwrap_or("");
    match fs.store.open(path, &n.name).await {
        Ok(mut download) => {
            let mut buf = Vec::new();
            if download.reader.read_to_end(&mut buf).await.is_err() {
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
            let mut res = Response::new(axum::body::Body::from(buf));
            if let Ok(v) = HeaderValue::from_str(&download.content_type) {
                res.headers_mut().insert(header::CONTENT_TYPE, v);
            }
            if let Ok(v) = HeaderValue::from_str(&format!(
                "inline; filename=\"{}\"",
                download.filename.replace('"', "")
            )) {
                res.headers_mut().insert(header::CONTENT_DISPOSITION, v);
            }
            res
        }
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

pub fn forbid_non_admin(ctx: &lariv_rs::plugins::users::state::AuthContext) -> Option<Response> {
    if is_admin(ctx) {
        None
    } else {
        Some(Redirect::to("/").into_response())
    }
}

pub fn file_opt(id: i64) -> Option<i64> {
    if id > 0 { Some(id) } else { None }
}

pub fn media_url(file_id: i64) -> String {
    format!("/media/{file_id}/")
}
