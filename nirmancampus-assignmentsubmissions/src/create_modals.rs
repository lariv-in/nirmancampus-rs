use super::keys::AssignmentCreateModalKey;
use super::routes::{
    AssignmentSubmissionsCreateGetRouteTag, AssignmentSubmissionsCreatePostRouteTag,
};

lariv_rs::impl_create_modal!(
    AssignmentCreateModalKey,
    AssignmentSubmissionsCreateGetRouteTag,
    AssignmentSubmissionsCreatePostRouteTag,
    "assignmentsubmissions.CreateForm"
);
