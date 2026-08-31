#![feature(impl_trait_in_assoc_type)]

//! Nirmancampus announcements plugin.

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

pub use menus::{
    announcement_detail_crumbs, announcement_detail_menu, announcements_crumbs, announcements_menu,
};

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

use state::AnnouncementsState;

pub struct NirmancampusAnnouncementsTag;

lariv_rs::define_passthrough_cap!(
    NirmancampusAnnouncementsStateCap,
    NirmancampusAnnouncementsTag,
    AnnouncementsState
);

lariv_rs::define_plugin_install! {
    plugin: NirmancampusAnnouncementsTag;
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
    L: HList + CapTagAbsent<NirmancampusAnnouncementsTag, TagProof>,
{
    type Output = HCons<NirmancampusAnnouncementsStateCap, L>;

    fn attach_state(app: App<L>) -> App<Self::Output> {
        let conn = app.get_capability::<DbTag, DbIdx>().items.conn.clone();
        app.add_capability(CapStore::with_items(AnnouncementsState::new(conn)))
    }
}
