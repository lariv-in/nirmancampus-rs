//! Courses addon — nested under Programs, not a dashboard tile.

lariv_rs::define_register_apps! {
    plugin: NirmancampusCoursesTag;
    key: "p_nirmancampus_courses";
    name: "Courses";
    href: "/courses/";
    icon: "book-open";
    plugin_type: lariv_rs::apps::PluginType::Addon;
    roles: ["admin", "student"];
}
