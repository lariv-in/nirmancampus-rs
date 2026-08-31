#![feature(impl_trait_in_assoc_type)]

//! Nirmancampus admission sessions plugin.

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

pub use menus::{session_detail_crumbs, session_detail_menu, sessions_crumbs, sessions_menu};

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

use state::SessionsState;

pub struct NirmancampusSessionsTag;

lariv_rs::define_passthrough_cap!(
    NirmancampusSessionsStateCap,
    NirmancampusSessionsTag,
    SessionsState
);

lariv_rs::define_plugin_install! {
    plugin: NirmancampusSessionsTag;
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
    L: HList + CapTagAbsent<NirmancampusSessionsTag, TagProof>,
{
    type Output = HCons<NirmancampusSessionsStateCap, L>;

    fn attach_state(app: App<L>) -> App<Self::Output> {
        let conn = app.get_capability::<DbTag, DbIdx>().items.conn.clone();
        app.add_capability(CapStore::with_items(SessionsState::new(conn)))
    }
}
