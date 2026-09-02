use super::keys::FeeCreateModalKey;
use super::routes::{StudentFeesCreateGetRouteTag, StudentFeesCreatePostRouteTag};

lariv_rs::impl_create_modal!(
    FeeCreateModalKey,
    StudentFeesCreateGetRouteTag,
    StudentFeesCreatePostRouteTag,
    "studentfees.FeeCreateForm"
);
