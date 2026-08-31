#![feature(impl_trait_in_assoc_type)]

//! Nirmancampus courses plugin — course catalog CRUD.

pub mod apps;
pub mod course_detail_related;
pub mod create_modals;
pub mod entities;
pub mod forms;
pub mod handlers;
pub mod keys;
pub mod menus;
pub mod migrations;
pub mod routes;
pub mod state;
pub mod templates;

pub use menus::{course_detail_crumbs, course_detail_menu, courses_crumbs, courses_menu};

use frunk::{HCons, hlist::HList};

use lariv_rs::{
    app::App,
    capability::CapStore,
    db::{DbCap, DbTag},
    hooks::AttachState,
    traits::{
        add::{AddCapability, CapTagAbsent},
        get::GetByCapTag,
    },
};

use state::CoursesState;

pub struct NirmancampusCoursesTag;

lariv_rs::define_passthrough_cap!(NirmancampusCoursesStateCap, NirmancampusCoursesTag, CoursesState);

lariv_rs::define_plugin_install! {
    plugin: NirmancampusCoursesTag;
    steps: [
        cap_attach(course_detail_related::CourseDetailRelatedTag, course_detail_related::CourseDetailRelatedCap, course_detail_related::CourseDetailRelatedCap::<frunk::HNil>::new()),
        cap_hook(course_detail_related::CourseDetailRelatedTag, course_detail_related::CourseDetailRelatedCap, course_detail_related::BaseHook),
        apps(apps::Hook),
        migrations(migrations::Hook),
        templates(templates::Hook),
        slots(templates::SlotsHook),
        http(routes::Hook),
        state(StateHook),
    ]
}

#[derive(Clone, Copy, Default)]
pub struct StateHook;

impl<L, DbIdx, TagProof> AttachState<L, (DbIdx, TagProof)> for StateHook
where
    L: GetByCapTag<DbTag, DbIdx, Value = DbCap>,
    L: HList + CapTagAbsent<NirmancampusCoursesTag, TagProof>,
{
    type Output = HCons<NirmancampusCoursesStateCap, L>;

    fn attach_state(app: App<L>) -> App<Self::Output> {
        let conn = app.get_capability::<DbTag, DbIdx>().items.conn.clone();
        app.add_capability(CapStore::with_items(CoursesState::new(conn)))
    }
}
