//! Shared Courses sidebar menus.

use maud::{Markup, html};

use lariv_rs::components::{SidebarMenu, SidebarMenuItem, sidebar_menu, sidebar_menu_item};

use crate::routes::{CoursesDetailRouteTag, CoursesListRouteTag};
use nirmancampus_common::ui::{item_crumbs, list_crumbs};

pub fn courses_menu() -> Markup {
    sidebar_menu(SidebarMenu {
        title: "Courses",
        children: html! {
            (sidebar_menu_item(SidebarMenuItem {
                title: "All Courses",
                url: "/courses/",
                ..Default::default()
            }))
        },
    })
}

pub fn courses_crumbs(leaf: &str) -> Markup {
    let list_url = CoursesListRouteTag.url();
    list_crumbs("Courses", &list_url, leaf)
}

pub fn course_detail_crumbs(id: i64, name: &str, action: Option<&str>) -> Markup {
    let list_url = CoursesListRouteTag.url();
    let detail_url = CoursesDetailRouteTag::new(id).url();
    item_crumbs("Courses", &list_url, name, Some(&detail_url), action)
}

pub fn course_detail_menu(id: i64, name: &str, is_admin: bool) -> Markup {
    let menu_title = format!("Course: {name}");
    let detail_url = CoursesDetailRouteTag::new(id).url();
    let list_url = CoursesListRouteTag.url();
    sidebar_menu(SidebarMenu {
        title: &menu_title,
        children: html! {
            (sidebar_menu_item(SidebarMenuItem {
                title: "Back to courses",
                url: &list_url,
                ..Default::default()
            }))
            (sidebar_menu_item(SidebarMenuItem {
                title: "Course Detail",
                url: &detail_url,
                ..Default::default()
            }))
            @if is_admin {
                (sidebar_menu_item(SidebarMenuItem {
                    title: "Edit Course",
                    url: &format!("/courses/{id}/edit/"),
                    ..Default::default()
                }))
            }
        },
    })
}
