//! Student Applications addon — unassigned users get dashboard shortcuts instead.

lariv_rs::define_register_apps! {
    plugin: NirmancampusStudentApplicationsTag;
    key: "p_nirmancampus_studentapplications";
    name: "Student Applications";
    href: "/student-applications/";
    icon: "document-text";
    plugin_type: lariv_rs::apps::PluginType::Addon;
    roles: ["admin", "unassigned"];
}
