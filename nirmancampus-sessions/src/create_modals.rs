use super::keys::SessionCreateModalKey;
use super::routes::{SessionsCreateGetRouteTag, SessionsCreatePostRouteTag};

lariv_rs::impl_create_modal!(
    SessionCreateModalKey,
    SessionsCreateGetRouteTag,
    SessionsCreatePostRouteTag,
    "sessions.SessionCreateForm"
);
