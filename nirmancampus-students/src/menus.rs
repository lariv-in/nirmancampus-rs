//! Shared Students sidebar menus.

use maud::{Markup, html};

use lariv_rs::components::{
    SidebarMenu, SidebarMenuItem, sidebar_menu, sidebar_menu_item,
};

use crate::routes::{StudentsDetailRouteTag, StudentsListRouteTag};
use nirmancampus_common::ui::{item_crumbs, list_crumbs};

pub fn students_menu() -> Markup {
    nirmancampus_common::ui::students_hub_menu()
}

pub fn students_crumbs(leaf: &str) -> Markup {
    let list_url = StudentsListRouteTag.url();
    list_crumbs("Students", &list_url, leaf)
}

pub fn student_detail_crumbs(id: i64, name: &str, action: Option<&str>) -> Markup {
    let list_url = StudentsListRouteTag.url();
    let detail_url = StudentsDetailRouteTag::new(id).url();
    item_crumbs("Students", &list_url, name, Some(&detail_url), action)
}

pub fn student_detail_menu(id: i64, name: &str, is_admin: bool) -> Markup {
    let menu_title = format!("Student: {name}");
    let detail_url = StudentsDetailRouteTag::new(id).url();
    let list_url = StudentsListRouteTag.url();
    sidebar_menu(SidebarMenu {
        title: &menu_title,
        children: html! {
            (sidebar_menu_item(SidebarMenuItem {
                title: "Back to students",
                url: &list_url,
                ..Default::default()
            }))
            (sidebar_menu_item(SidebarMenuItem {
                title: "Student Detail",
                url: &detail_url,
                ..Default::default()
            }))
            @if is_admin {
                (sidebar_menu_item(SidebarMenuItem {
                    title: "Edit Student",
                    url: &format!("/students/{id}/edit/"),
                    ..Default::default()
                }))
            }
        },
    })
}
