use maud::{Markup, html};

use lariv_rs::components::{SidebarMenu, SidebarMenuItem, sidebar_menu, sidebar_menu_item};

use crate::routes::{AssignmentSubmissionsDetailRouteTag, AssignmentSubmissionsListRouteTag};
use nirmancampus_common::ui::{item_crumbs, list_crumbs};

pub fn assignments_menu() -> Markup {
    sidebar_menu(SidebarMenu {
        title: "Assignment Submissions",
        children: html! {
            (sidebar_menu_item(SidebarMenuItem {
                title: "All Submissions",
                url: "/assignment-submissions/",
                ..Default::default()
            }))
        },
    })
}

pub fn assignments_crumbs(leaf: &str) -> Markup {
    let list_url = AssignmentSubmissionsListRouteTag.url();
    list_crumbs("Assignment Submissions", &list_url, leaf)
}

pub fn assignment_detail_crumbs(id: i64, title: &str, action: Option<&str>) -> Markup {
    let list_url = AssignmentSubmissionsListRouteTag.url();
    let detail_url = AssignmentSubmissionsDetailRouteTag::new(id).url();
    item_crumbs(
        "Assignment Submissions",
        &list_url,
        title,
        Some(&detail_url),
        action,
    )
}

pub fn assignment_detail_menu(id: i64, title: &str, is_admin: bool) -> Markup {
    let menu_title = format!("Submission: {title}");
    let detail_url = AssignmentSubmissionsDetailRouteTag::new(id).url();
    let list_url = AssignmentSubmissionsListRouteTag.url();
    sidebar_menu(SidebarMenu {
        title: &menu_title,
        children: html! {
            (sidebar_menu_item(SidebarMenuItem {
                title: "Back to assignment submissions",
                url: &list_url,
                ..Default::default()
            }))
            (sidebar_menu_item(SidebarMenuItem {
                title: "Submission Detail",
                url: &detail_url,
                ..Default::default()
            }))
            @if is_admin {
                (sidebar_menu_item(SidebarMenuItem {
                    title: "Edit submission",
                    url: &format!("/assignment-submissions/{id}/edit/"),
                    ..Default::default()
                }))
            }
        },
    })
}
