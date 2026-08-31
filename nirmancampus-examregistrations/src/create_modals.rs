use super::keys::ExamCreateModalKey;
use super::routes::{ExamRegistrationsCreateGetRouteTag, ExamRegistrationsCreatePostRouteTag};

lariv_rs::impl_create_modal!(
    ExamCreateModalKey,
    ExamRegistrationsCreateGetRouteTag,
    ExamRegistrationsCreatePostRouteTag,
    "examregistrations.CreateForm"
);
