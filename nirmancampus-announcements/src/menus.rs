use maud::{Markup, html};

use lariv_rs::components::{SidebarMenu, SidebarMenuItem, sidebar_menu, sidebar_menu_item};

use crate::routes::{AnnouncementsDetailRouteTag, AnnouncementsListRouteTag};
use nirmancampus_common::ui::{item_crumbs, list_crumbs};

pub fn announcements_menu() -> Markup {
    sidebar_menu(SidebarMenu {
        title: "Announcements",
        children: html! {
            (sidebar_menu_item(SidebarMenuItem {
                title: "All Announcements",
                url: "/announcements/",
                ..Default::default()
            }))
        },
    })
}

pub fn announcements_crumbs(leaf: &str) -> Markup {
    let list_url = AnnouncementsListRouteTag.url();
    list_crumbs("Announcements", &list_url, leaf)
}

pub fn announcement_detail_crumbs(id: i64, title: &str, action: Option<&str>) -> Markup {
    let list_url = AnnouncementsListRouteTag.url();
    let detail_url = AnnouncementsDetailRouteTag::new(id).url();
    item_crumbs("Announcements", &list_url, title, Some(&detail_url), action)
}

pub fn announcement_detail_menu(id: i64, title: &str) -> Markup {
    let menu_title = format!("Announcement: {title}");
    let detail_url = AnnouncementsDetailRouteTag::new(id).url();
    let list_url = AnnouncementsListRouteTag.url();
    sidebar_menu(SidebarMenu {
        title: &menu_title,
        children: html! {
            (sidebar_menu_item(SidebarMenuItem {
                title: "Back to announcements",
                url: &list_url,
                ..Default::default()
            }))
            (sidebar_menu_item(SidebarMenuItem {
                title: "Announcement Detail",
                url: &detail_url,
                ..Default::default()
            }))
        },
    })
}
