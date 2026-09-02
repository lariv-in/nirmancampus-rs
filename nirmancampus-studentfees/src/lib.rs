#![feature(impl_trait_in_assoc_type)]

//! Nirmancampus student fees plugin — live MySQL `tblfee` CRUD.

pub mod apps;
pub mod db;
pub mod entities;
pub mod forms;
pub mod handlers;
pub mod keys;
pub mod lookup;
pub mod menus;
pub mod migrations;
pub mod parse;
pub mod preferences;
pub mod routes;
pub mod state;
pub mod tblfee_xlsx;
pub mod templates;

mod create_modals;

pub use lookup::{StudentFeeView, contact_matches, find_by_enroll, find_by_id};
pub use menus::{fee_detail_crumbs, fee_detail_menu, fees_crumbs, fees_menu};
pub use state::StudentFeesState;

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

pub struct NirmancampusStudentFeesTag;

lariv_rs::define_passthrough_cap!(
    NirmancampusStudentFeesStateCap,
    NirmancampusStudentFeesTag,
    StudentFeesState
);

lariv_rs::define_plugin_install! {
    plugin: NirmancampusStudentFeesTag;
    steps: [
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
    L: HList + CapTagAbsent<NirmancampusStudentFeesTag, TagProof>,
{
    type Output = HCons<NirmancampusStudentFeesStateCap, L>;

    fn attach_state(app: App<L>) -> App<Self::Output> {
        let conn = app.get_capability::<DbTag, DbIdx>().items.conn.clone();
        app.add_capability(CapStore::with_items(StudentFeesState::new(conn)))
    }
}
