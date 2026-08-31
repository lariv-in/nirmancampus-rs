//! Exam Registrations addon — nested under Students, not a dashboard tile.

lariv_rs::define_register_apps! {
    plugin: NirmancampusExamRegistrationsTag;
    key: "p_nirmancampus_examregistrations";
    name: "Exam Registrations";
    href: "/exam-registrations/";
    icon: "clipboard-document-list";
    plugin_type: lariv_rs::apps::PluginType::Addon;
    roles: ["superuser", "admin", "student"];
}
