//! Academic Records addon — nested under Students, not a dashboard tile.

lariv_rs::define_register_apps! {
    plugin: NirmancampusAcademicRecordsTag;
    key: "p_nirmancampus_academicrecords";
    name: "Academic Records";
    href: "/academic-records/";
    icon: "book-open";
    plugin_type: lariv_rs::apps::PluginType::Addon;
    roles: ["superuser", "admin", "student"];
}
