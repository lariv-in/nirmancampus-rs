//! Student Payments addon — nested under Students, not a dashboard tile.

lariv_rs::define_register_apps! {
    plugin: NirmancampusStudentPaymentsTag;
    key: "p_nirmancampus_studentpayments";
    name: "Student Payments";
    href: "/student-payments/";
    icon: "banknotes";
    plugin_type: lariv_rs::apps::PluginType::Addon;
    roles: ["superuser", "admin", "student"];
}
