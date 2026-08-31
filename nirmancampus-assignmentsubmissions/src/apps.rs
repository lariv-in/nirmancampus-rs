//! Assignment Submissions addon — nested under Students, not a dashboard tile.

lariv_rs::define_register_apps! {
    plugin: NirmancampusAssignmentSubmissionsTag;
    key: "p_nirmancampus_assignmentsubmissions";
    name: "Assignment Submissions";
    href: "/assignment-submissions/";
    icon: "document-check";
    plugin_type: lariv_rs::apps::PluginType::Addon;
    roles: ["superuser", "admin", "student"];
}
