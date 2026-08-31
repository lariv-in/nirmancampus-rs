#![feature(impl_trait_in_assoc_type)]

//! Nirmancampus student applications plugin.

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

pub use menus::{application_detail_crumbs, application_detail_menu, applications_crumbs, applications_menu};

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

use state::StudentApplicationsState;

pub struct NirmancampusStudentApplicationsTag;

lariv_rs::define_passthrough_cap!(
    NirmancampusStudentApplicationsStateCap,
    NirmancampusStudentApplicationsTag,
    StudentApplicationsState
);

lariv_rs::define_plugin_install! {
    plugin: NirmancampusStudentApplicationsTag;
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
    L: HList + CapTagAbsent<NirmancampusStudentApplicationsTag, TagProof>,
{
    type Output = HCons<NirmancampusStudentApplicationsStateCap, L>;

    fn attach_state(app: App<L>) -> App<Self::Output> {
        let conn = app.get_capability::<DbTag, DbIdx>().items.conn.clone();
        app.add_capability(CapStore::with_items(StudentApplicationsState::new(conn)))
    }
}
