//! Nirmancampus dashboard addon — unassigned applicants see application
//! shortcuts; everyone else gets the tot-school-style apps grid.

pub mod handlers;
pub mod routes;
pub mod templates;

use lariv_rs::define_plugin_install;

pub struct NirmancampusDashboardTag;

define_plugin_install! {
    plugin: NirmancampusDashboardTag;
    steps: [
        templates(templates::Hook, AppsIdx),
        http(routes::Hook),
    ]
}
