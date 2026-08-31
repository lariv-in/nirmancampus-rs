use super::keys::ApplicationCreateModalKey;
use super::routes::{StudentApplicationsCreateGetRouteTag, StudentApplicationsCreatePostRouteTag};

lariv_rs::impl_create_modal!(
    ApplicationCreateModalKey,
    StudentApplicationsCreateGetRouteTag,
    StudentApplicationsCreatePostRouteTag,
    "studentapplications.ApplicationCreateForm"
);
