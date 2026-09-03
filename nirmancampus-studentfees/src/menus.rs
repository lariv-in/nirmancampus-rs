use maud::{Markup, html};

use lariv_rs::components::{SidebarMenu, SidebarMenuItem, sidebar_menu, sidebar_menu_item};

use crate::routes::{StudentFeesDetailRouteTag, StudentFeesListRouteTag};
use nirmancampus_common::ui::{item_crumbs, list_crumbs};

pub fn fees_menu() -> Markup {
    sidebar_menu(SidebarMenu {
        title: "IGNOU Students",
        children: html! {
            (sidebar_menu_item(SidebarMenuItem {
                title: "Fee records",
                url: "/student-fees/",
                ..Default::default()
            }))
            (sidebar_menu_item(SidebarMenuItem {
                title: "Preferences",
                url: "/student-fees/preferences/",
                ..Default::default()
            }))
        },
    })
}

pub fn fees_crumbs(leaf: &str) -> Markup {
    let list_url = StudentFeesListRouteTag.url();
    list_crumbs("IGNOU Students", &list_url, leaf)
}

pub fn fee_detail_crumbs(id: i64, title: &str, action: Option<&str>) -> Markup {
    let list_url = StudentFeesListRouteTag.url();
    let detail_url = StudentFeesDetailRouteTag::new(id).url();
    item_crumbs("IGNOU Students", &list_url, title, Some(&detail_url), action)
}

pub fn fee_detail_menu(id: i64, title: &str, is_admin: bool) -> Markup {
    let menu_title = format!("Fee: {title}");
    let detail_url = StudentFeesDetailRouteTag::new(id).url();
    let list_url = StudentFeesListRouteTag.url();
    sidebar_menu(SidebarMenu {
        title: &menu_title,
        children: html! {
            (sidebar_menu_item(SidebarMenuItem {
                title: "Back to fees",
                url: &list_url,
                ..Default::default()
            }))
            (sidebar_menu_item(SidebarMenuItem {
                title: "Fee Detail",
                url: &detail_url,
                ..Default::default()
            }))
            @if is_admin {
                (sidebar_menu_item(SidebarMenuItem {
                    title: "Edit fee",
                    url: &format!("/student-fees/{id}/edit/"),
                    ..Default::default()
                }))
            }
        },
    })
}

pub fn prefs_crumbs() -> Markup {
    fees_crumbs("Preferences")
}
