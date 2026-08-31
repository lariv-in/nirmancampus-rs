//! Programs dashboard app tile.

lariv_rs::define_register_apps! {
    plugin: NirmancampusProgramsTag;
    key: "p_nirmancampus_programs";
    name: "Programs";
    href: "/programs/";
    icon: "academic-cap";
    roles: ["admin", "student"];
}
