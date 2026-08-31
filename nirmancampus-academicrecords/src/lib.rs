#![feature(impl_trait_in_assoc_type)]

//! Nirmancampus academic records plugin.

pub mod academic_record_detail_related;
pub mod apps;
pub mod entities;
pub mod forms;
pub mod handlers;
pub mod keys;
pub mod menus;
pub mod migrations;
pub mod routes;
pub mod state;
pub mod student_detail;
pub mod templates;

mod create_modals;

pub use menus::{academic_record_detail_crumbs, academic_record_detail_menu, academic_records_crumbs, academic_records_menu};

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
use nirmancampus_students::student_detail_related::{
    StudentDetailRelatedCap, StudentDetailRelatedTag,
};

use state::AcademicRecordsState;

pub struct NirmancampusAcademicRecordsTag;

lariv_rs::define_passthrough_cap!(
    NirmancampusAcademicRecordsStateCap,
    NirmancampusAcademicRecordsTag,
    AcademicRecordsState
);

lariv_rs::define_plugin_install! {
    plugin: NirmancampusAcademicRecordsTag;
    steps: [
        cap_attach(academic_record_detail_related::AcademicRecordDetailRelatedTag, academic_record_detail_related::AcademicRecordDetailRelatedCap, academic_record_detail_related::AcademicRecordDetailRelatedCap::<frunk::HNil>::new()),
        cap_hook(academic_record_detail_related::AcademicRecordDetailRelatedTag, academic_record_detail_related::AcademicRecordDetailRelatedCap, academic_record_detail_related::BaseHook),
        cap_hook(StudentDetailRelatedTag, StudentDetailRelatedCap, student_detail::StudentDetailHook),
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
    L: HList + CapTagAbsent<NirmancampusAcademicRecordsTag, TagProof>,
{
    type Output = HCons<NirmancampusAcademicRecordsStateCap, L>;

    fn attach_state(app: App<L>) -> App<Self::Output> {
        let conn = app.get_capability::<DbTag, DbIdx>().items.conn.clone();
        app.add_capability(CapStore::with_items(AcademicRecordsState::new(conn)))
    }
}
