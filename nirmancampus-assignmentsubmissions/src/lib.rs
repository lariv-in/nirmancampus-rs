#![feature(impl_trait_in_assoc_type)]

//! Nirmancampus assignment submissions plugin.

pub mod academic_record_detail;
pub mod apps;
pub mod entities;
pub mod forms;
pub mod handlers;
pub mod keys;
pub mod menus;
pub mod migrations;
pub mod routes;
pub mod state;
pub mod templates;

mod create_modals;

pub use menus::{assignment_detail_crumbs, assignment_detail_menu, assignments_crumbs, assignments_menu};

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
use nirmancampus_academicrecords::academic_record_detail_related::{
    AcademicRecordDetailRelatedCap, AcademicRecordDetailRelatedTag,
};

use state::AssignmentSubmissionsState;

pub struct NirmancampusAssignmentSubmissionsTag;

lariv_rs::define_passthrough_cap!(
    NirmancampusAssignmentSubmissionsStateCap,
    NirmancampusAssignmentSubmissionsTag,
    AssignmentSubmissionsState
);

lariv_rs::define_plugin_install! {
    plugin: NirmancampusAssignmentSubmissionsTag;
    steps: [
        cap_hook(AcademicRecordDetailRelatedTag, AcademicRecordDetailRelatedCap, academic_record_detail::AcademicRecordDetailHook),
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
    L: HList + CapTagAbsent<NirmancampusAssignmentSubmissionsTag, TagProof>,
{
    type Output = HCons<NirmancampusAssignmentSubmissionsStateCap, L>;

    fn attach_state(app: App<L>) -> App<Self::Output> {
        let conn = app.get_capability::<DbTag, DbIdx>().items.conn.clone();
        app.add_capability(CapStore::with_items(AssignmentSubmissionsState::new(conn)))
    }
}
