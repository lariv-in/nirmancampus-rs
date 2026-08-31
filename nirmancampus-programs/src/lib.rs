#![feature(impl_trait_in_assoc_type)]

//! Nirmancampus programs plugin — program catalog, structure editor, and course placements.

pub mod apps;
pub mod course_detail;
pub mod create_modals;
pub mod entities;
pub mod forms;
pub mod handlers;
pub mod keys;
pub mod menus;
pub mod migrations;
pub mod routes;
pub mod seed;
pub mod state;
pub mod templates;

pub use menus::{program_detail_crumbs, program_detail_menu, programs_crumbs, programs_menu};

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
use nirmancampus_courses::course_detail_related::{CourseDetailRelatedCap, CourseDetailRelatedTag};

use state::ProgramsState;

pub struct NirmancampusProgramsTag;

lariv_rs::define_passthrough_cap!(
    NirmancampusProgramsStateCap,
    NirmancampusProgramsTag,
    ProgramsState
);

lariv_rs::define_plugin_install! {
    plugin: NirmancampusProgramsTag;
    steps: [
        cap_hook(CourseDetailRelatedTag, CourseDetailRelatedCap, course_detail::CourseDetailHook),
        apps(apps::Hook),
        migrations(migrations::Hook),
        templates(templates::Hook),
        slots(templates::SlotsHook),
        http(routes::Hook),
        state(StateHook),
        seeds(SeedsHook),
    ]
}

#[derive(Clone, Copy, Default)]
pub struct StateHook;

impl<L, DbIdx, TagProof> AttachState<L, (DbIdx, TagProof)> for StateHook
where
    L: GetByCapTag<DbTag, DbIdx, Value = DbCap>,
    L: HList + CapTagAbsent<NirmancampusProgramsTag, TagProof>,
{
    type Output = HCons<NirmancampusProgramsStateCap, L>;

    fn attach_state(app: App<L>) -> App<Self::Output> {
        let conn = app.get_capability::<DbTag, DbIdx>().items.conn.clone();
        app.add_capability(CapStore::with_items(ProgramsState::new(conn)))
    }
}

#[derive(Clone, Copy, Default)]
pub struct SeedsHook;

#[async_trait::async_trait]
impl<M, Idx> RunSeed<M, Idx> for SeedsHook
where
    M: GetByTag<NirmancampusProgramsTag, Idx, Value = ProgramsState> + Sync,
{
    async fn run_seed(app: &MountedApp<M>) -> anyhow::Result<()> {
        seed::seed(&app.get_capability_output::<NirmancampusProgramsTag, Idx>().db).await?;
        Ok(())
    }
}
