#![feature(impl_trait_in_assoc_type)]

//! Nirmancampus public website and admin CMS.

pub mod apps;
pub mod config;
pub mod entities;
pub mod fee_session;
pub mod forms;
pub mod handlers;
pub mod keys;
pub mod menus;
pub mod migrations;
pub mod routes;
pub mod seed;
pub mod state;
pub mod templates;

mod create_modals;

use frunk::{HCons, hlist::HList};

use lariv_rs::{
    app::{App, MountedApp},
    capability::CapStore,
    config::{ConfigCap, ConfigTag},
    db::{DbCap, DbTag},
    hooks::{AttachState, RunSeed, RunServeStartup},
    plugins::filesystem::{FilesystemTag, state::FilesystemState},
    traits::{
        add::{AddCapability, CapTagAbsent},
        get::{GetByCapTag, GetByTag},
    },
};

use config::{WebsiteConfig, WebsiteConfigTag};
use state::WebsiteState;

pub struct NirmancampusWebsiteTag;

lariv_rs::define_passthrough_cap!(
    NirmancampusWebsiteStateCap,
    NirmancampusWebsiteTag,
    WebsiteState
);

lariv_rs::define_plugin_install! {
    plugin: NirmancampusWebsiteTag;
    steps: [
        apps(apps::Hook),
        migrations(migrations::Hook),
        templates(templates::Hook),
        slots(templates::SlotsHook),
        config(WebsiteConfigTag, WebsiteConfig),
        http(routes::Hook),
        state(StateHook),
        seeds(SeedsHook),
        serve_startup(SeedsHook),
    ]
}

#[derive(Clone, Copy, Default)]
pub struct StateHook;

impl<L, DbIdx, CfgIdx, Configs, WsCfgIdx, TagProof>
    AttachState<L, (DbIdx, CfgIdx, Configs, WsCfgIdx, TagProof)> for StateHook
where
    L: GetByCapTag<DbTag, DbIdx, Value = DbCap>,
    L: GetByCapTag<ConfigTag, CfgIdx, Value = ConfigCap<frunk::HNil, Configs>>,
    Configs: GetByTag<WebsiteConfigTag, WsCfgIdx, Value = WebsiteConfig>,
    L: HList + CapTagAbsent<NirmancampusWebsiteTag, TagProof>,
{
    type Output = HCons<NirmancampusWebsiteStateCap, L>;

    fn attach_state(app: App<L>) -> App<Self::Output> {
        let conn = app.get_capability::<DbTag, DbIdx>().items.conn.clone();
        let config = <Configs as GetByTag<WebsiteConfigTag, WsCfgIdx>>::get_by_tag(
            &app.get_capability::<ConfigTag, CfgIdx>().items,
        )
        .clone();
        app.add_capability(CapStore::with_items(WebsiteState::new(
            conn,
            config.static_dir,
        )))
    }
}

#[derive(Clone, Copy, Default)]
pub struct SeedsHook;

#[async_trait::async_trait]
impl<M, WsIdx, FsIdx> RunSeed<M, (WsIdx, FsIdx)> for SeedsHook
where
    M: GetByTag<NirmancampusWebsiteTag, WsIdx, Value = WebsiteState> + Sync,
    M: GetByTag<FilesystemTag, FsIdx, Value = FilesystemState>,
{
    async fn run_seed(app: &MountedApp<M>) -> anyhow::Result<()> {
        let website = app.get_capability_output::<NirmancampusWebsiteTag, WsIdx>();
        let fs = app.get_capability_output::<FilesystemTag, FsIdx>();
        seed::ensure_website_static(website, fs).await
    }
}

#[async_trait::async_trait]
impl<M, WsIdx, FsIdx> RunServeStartup<M, (WsIdx, FsIdx)> for SeedsHook
where
    M: GetByTag<NirmancampusWebsiteTag, WsIdx, Value = WebsiteState> + Sync,
    M: GetByTag<FilesystemTag, FsIdx, Value = FilesystemState>,
{
    async fn run_serve_startup(app: &MountedApp<M>) -> anyhow::Result<()> {
        let website = app.get_capability_output::<NirmancampusWebsiteTag, WsIdx>();
        let fs = app.get_capability_output::<FilesystemTag, FsIdx>();
        seed::ensure_website_static(website, fs).await
    }
}
