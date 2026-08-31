//! Nirmancampus users addon — seeds campus-specific roles.

pub mod apps;
pub mod seed;

use frunk::{HCons, hlist::HList};

use lariv_rs::{
    app::{App, MountedApp},
    capability::CapStore,
    db::{DbCap, DbTag},
    hooks::{AttachState, RunSeed},
    traits::{
        add::{AddCapability, CapTagAbsent},
        get::{GetByCapTag, GetByTag},
    },
};

use seed::NirmancampusUsersState;

pub struct NirmancampusUsersTag;

lariv_rs::define_passthrough_cap!(
    NirmancampusUsersStateCap,
    NirmancampusUsersTag,
    NirmancampusUsersState
);

lariv_rs::define_plugin_install! {
    plugin: NirmancampusUsersTag;
    steps: [
        apps(apps::Hook),
        state(StateHook),
        seeds(SeedsHook),
    ]
}

#[derive(Clone, Copy, Default)]
pub struct StateHook;

impl<L, DbIdx, TagProof> AttachState<L, (DbIdx, TagProof)> for StateHook
where
    L: GetByCapTag<DbTag, DbIdx, Value = DbCap>,
    L: HList + CapTagAbsent<NirmancampusUsersTag, TagProof>,
{
    type Output = HCons<NirmancampusUsersStateCap, L>;

    fn attach_state(app: App<L>) -> App<Self::Output> {
        let conn = app.get_capability::<DbTag, DbIdx>().items.conn.clone();
        app.add_capability(CapStore::with_items(NirmancampusUsersState::new(conn)))
    }
}

#[derive(Clone, Copy, Default)]
pub struct SeedsHook;

#[async_trait::async_trait]
impl<M, Idx> RunSeed<M, Idx> for SeedsHook
where
    M: GetByTag<NirmancampusUsersTag, Idx, Value = NirmancampusUsersState> + Sync,
{
    async fn run_seed(app: &MountedApp<M>) -> anyhow::Result<()> {
        seed::seed(app.get_capability_output::<NirmancampusUsersTag, Idx>()).await?;
        Ok(())
    }
}
