//! Students dashboard app tile.

lariv_rs::define_register_apps! {
    plugin: NirmancampusStudentsTag;
    key: "p_nirmancampus_students";
    name: "Students";
    href: "/students/";
    icon: "user";
    roles: ["superuser", "admin", "student"];
}
