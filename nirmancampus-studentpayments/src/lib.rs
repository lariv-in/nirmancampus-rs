#![feature(impl_trait_in_assoc_type)]

//! Nirmancampus student payments plugin.

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

pub use menus::{payment_detail_crumbs, payment_detail_menu, payments_crumbs, payments_menu};

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

use state::StudentPaymentsState;

pub struct NirmancampusStudentPaymentsTag;

lariv_rs::define_passthrough_cap!(
    NirmancampusStudentPaymentsStateCap,
    NirmancampusStudentPaymentsTag,
    StudentPaymentsState
);

lariv_rs::define_plugin_install! {
    plugin: NirmancampusStudentPaymentsTag;
    steps: [
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
    L: HList + CapTagAbsent<NirmancampusStudentPaymentsTag, TagProof>,
{
    type Output = HCons<NirmancampusStudentPaymentsStateCap, L>;

    fn attach_state(app: App<L>) -> App<Self::Output> {
        let conn = app.get_capability::<DbTag, DbIdx>().items.conn.clone();
        app.add_capability(CapStore::with_items(StudentPaymentsState::new(conn)))
    }
}
