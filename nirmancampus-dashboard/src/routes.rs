//! Dashboard launchpad and `/apps/` alias. Website owns `/`.

lariv_rs::define_plugin_routes! {
    plugin: NirmancampusDashboardTag;
    routes: [
        get DashboardAppsRouteTag, "/dashboard", crate::handlers::apps;
        get AppsAliasRouteTag, "/apps", crate::handlers::apps;
    ]
}
