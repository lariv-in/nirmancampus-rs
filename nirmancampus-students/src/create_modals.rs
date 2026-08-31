use super::keys::StudentCreateModalKey;
use super::routes::{StudentsCreateGetRouteTag, StudentsCreatePostRouteTag};

lariv_rs::impl_create_modal!(
    StudentCreateModalKey,
    StudentsCreateGetRouteTag,
    StudentsCreatePostRouteTag,
    "students.StudentCreateForm"
);
