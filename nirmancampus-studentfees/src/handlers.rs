pub mod fees;
pub mod preferences;

use axum::{
    response::{IntoResponse, Redirect, Response},
};
use nirmancampus_common::is_admin;

pub fn forbid_non_admin(ctx: &lariv_rs::plugins::users::state::AuthContext) -> Option<Response> {
    if is_admin(ctx) {
        None
    } else {
        Some(Redirect::to("/").into_response())
    }
}
