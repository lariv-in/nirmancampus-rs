use maud::{Markup, html};

use lariv_rs::components::{SidebarMenu, SidebarMenuItem, sidebar_menu, sidebar_menu_item};

use crate::routes::{SessionsDetailRouteTag, SessionsListRouteTag};
use nirmancampus_common::ui::{item_crumbs, list_crumbs};

pub fn sessions_menu() -> Markup {
    sidebar_menu(SidebarMenu {
        title: "Sessions",
        children: html! {
            (sidebar_menu_item(SidebarMenuItem {
                title: "All Sessions",
                url: "/sessions/",
                ..Default::default()
            }))
        },
    })
}

pub fn sessions_crumbs(leaf: &str) -> Markup {
    let list_url = SessionsListRouteTag.url();
    list_crumbs("Sessions", &list_url, leaf)
}

pub fn session_detail_crumbs(id: i64, name: &str, action: Option<&str>) -> Markup {
    let list_url = SessionsListRouteTag.url();
    let detail_url = SessionsDetailRouteTag::new(id).url();
    item_crumbs("Sessions", &list_url, name, Some(&detail_url), action)
}

pub fn session_detail_menu(id: i64, name: &str) -> Markup {
    let menu_title = format!("Session: {name}");
    let detail_url = SessionsDetailRouteTag::new(id).url();
    let list_url = SessionsListRouteTag.url();
    sidebar_menu(SidebarMenu {
        title: &menu_title,
        children: html! {
            (sidebar_menu_item(SidebarMenuItem {
                title: "Back to sessions",
                url: &list_url,
                ..Default::default()
            }))
            (sidebar_menu_item(SidebarMenuItem {
                title: "Session Detail",
                url: &detail_url,
                ..Default::default()
            }))
        },
    })
}
