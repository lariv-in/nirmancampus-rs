use maud::{Markup, html};

use lariv_rs::components::{SidebarMenu, SidebarMenuItem, sidebar_menu, sidebar_menu_item};

use crate::routes::{StudentApplicationsDetailRouteTag, StudentApplicationsListRouteTag};
use nirmancampus_common::ui::{item_crumbs, list_crumbs};

pub fn applications_menu() -> Markup {
    sidebar_menu(SidebarMenu {
        title: "Applications",
        children: html! {
            (sidebar_menu_item(SidebarMenuItem {
                title: "All Applications",
                url: "/student-applications/",
                ..Default::default()
            }))
        },
    })
}

pub fn applications_crumbs(leaf: &str) -> Markup {
    let list_url = StudentApplicationsListRouteTag.url();
    list_crumbs("Applications", &list_url, leaf)
}

pub fn application_detail_crumbs(id: i64, name: &str, action: Option<&str>) -> Markup {
    let list_url = StudentApplicationsListRouteTag.url();
    let detail_url = StudentApplicationsDetailRouteTag::new(id).url();
    item_crumbs("Applications", &list_url, name, Some(&detail_url), action)
}

pub fn application_detail_menu(id: i64, name: &str) -> Markup {
    let menu_title = format!("Application: {name}");
    let detail_url = StudentApplicationsDetailRouteTag::new(id).url();
    let list_url = StudentApplicationsListRouteTag.url();
    sidebar_menu(SidebarMenu {
        title: &menu_title,
        children: html! {
            (sidebar_menu_item(SidebarMenuItem {
                title: "Back to applications",
                url: &list_url,
                ..Default::default()
            }))
            (sidebar_menu_item(SidebarMenuItem {
                title: "Application Detail",
                url: &detail_url,
                ..Default::default()
            }))
        },
    })
}
