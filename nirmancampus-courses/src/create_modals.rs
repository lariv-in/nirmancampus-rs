//! Typed [`CreateModal`] wiring for course create dialogs.

use super::keys::CourseCreateModalKey;
use super::routes::{CoursesCreateGetRouteTag, CoursesCreatePostRouteTag};

lariv_rs::impl_create_modal!(
    CourseCreateModalKey,
    CoursesCreateGetRouteTag,
    CoursesCreatePostRouteTag,
    "courses.CourseCreateForm"
);
