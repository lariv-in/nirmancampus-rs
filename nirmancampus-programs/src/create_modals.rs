//! Typed [`CreateModal`] wiring for program create dialogs.

use super::keys::ProgramCreateModalKey;
use super::routes::{ProgramsCreateGetRouteTag, ProgramsCreatePostRouteTag};

lariv_rs::impl_create_modal!(
    ProgramCreateModalKey,
    ProgramsCreateGetRouteTag,
    ProgramsCreatePostRouteTag,
    "programs.ProgramCreateForm"
);
