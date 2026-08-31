#![feature(impl_trait_in_assoc_type)]

//! Nirmancampus students plugin — student records CRUD.

pub mod apps;
pub mod entities;
pub mod forms;
pub mod handlers;
pub mod keys;
pub mod menus;
pub mod migrations;
pub mod routes;
pub mod state;
pub mod student_detail_related;
pub mod templates;

mod create_modals;

pub use menus::{student_detail_crumbs, student_detail_menu, students_crumbs, students_menu};

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

use state::StudentsState;

pub struct NirmancampusStudentsTag;

lariv_rs::define_passthrough_cap!(
    NirmancampusStudentsStateCap,
    NirmancampusStudentsTag,
    StudentsState
);

lariv_rs::define_plugin_install! {
    plugin: NirmancampusStudentsTag;
    steps: [
        cap_attach(student_detail_related::StudentDetailRelatedTag, student_detail_related::StudentDetailRelatedCap, student_detail_related::StudentDetailRelatedCap::<frunk::HNil>::new()),
        cap_hook(student_detail_related::StudentDetailRelatedTag, student_detail_related::StudentDetailRelatedCap, student_detail_related::BaseHook),
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
    L: HList + CapTagAbsent<NirmancampusStudentsTag, TagProof>,
{
    type Output = HCons<NirmancampusStudentsStateCap, L>;

    fn attach_state(app: App<L>) -> App<Self::Output> {
        let conn = app.get_capability::<DbTag, DbIdx>().items.conn.clone();
        app.add_capability(CapStore::with_items(StudentsState::new(conn)))
    }
}
