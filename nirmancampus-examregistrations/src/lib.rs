#![feature(impl_trait_in_assoc_type)]

//! Nirmancampus exam registrations plugin.

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

pub use menus::{exam_detail_crumbs, exam_detail_menu, exams_crumbs, exams_menu};

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

use state::ExamRegistrationsState;

pub struct NirmancampusExamRegistrationsTag;

lariv_rs::define_passthrough_cap!(
    NirmancampusExamRegistrationsStateCap,
    NirmancampusExamRegistrationsTag,
    ExamRegistrationsState
);

lariv_rs::define_plugin_install! {
    plugin: NirmancampusExamRegistrationsTag;
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
    L: HList + CapTagAbsent<NirmancampusExamRegistrationsTag, TagProof>,
{
    type Output = HCons<NirmancampusExamRegistrationsStateCap, L>;

    fn attach_state(app: App<L>) -> App<Self::Output> {
        let conn = app.get_capability::<DbTag, DbIdx>().items.conn.clone();
        app.add_capability(CapStore::with_items(ExamRegistrationsState::new(conn)))
    }
}
