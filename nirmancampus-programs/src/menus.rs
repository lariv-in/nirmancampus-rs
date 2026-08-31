use maud::{Markup, html};

use lariv_rs::components::{SidebarMenu, SidebarMenuItem, sidebar_menu, sidebar_menu_item};

use crate::routes::{
    ProgramsDetailRouteTag, ProgramsListRouteTag, ProgramsStructureEditRouteTag,
};
use nirmancampus_common::ui::{item_crumbs, list_crumbs};

pub fn programs_menu() -> Markup {
    sidebar_menu(SidebarMenu {
        title: "Programs",
        children: html! {
            (sidebar_menu_item(SidebarMenuItem {
                title: "All Programs",
                url: "/programs/",
                ..Default::default()
            }))
            (sidebar_menu_item(SidebarMenuItem {
                title: "All Courses",
                url: "/courses/",
                ..Default::default()
            }))
        },
    })
}

pub fn programs_crumbs(leaf: &str) -> Markup {
    let list_url = ProgramsListRouteTag.url();
    list_crumbs("Programs", &list_url, leaf)
}

pub fn program_detail_crumbs(id: i64, name: &str, action: Option<&str>) -> Markup {
    let list_url = ProgramsListRouteTag.url();
    let detail_url = ProgramsDetailRouteTag::new(id).url();
    item_crumbs("Programs", &list_url, name, Some(&detail_url), action)
}

pub fn program_detail_menu(id: i64, name: &str, is_admin: bool) -> Markup {
    let menu_title = format!("Program: {name}");
    let detail_url = ProgramsDetailRouteTag::new(id).url();
    let list_url = ProgramsListRouteTag.url();
    let structure_url = ProgramsStructureEditRouteTag::new(id).url();
    sidebar_menu(SidebarMenu {
        title: &menu_title,
        children: html! {
            (sidebar_menu_item(SidebarMenuItem {
                title: "Back to all Programs",
                url: &list_url,
                ..Default::default()
            }))
            (sidebar_menu_item(SidebarMenuItem {
                title: "Program Detail",
                url: &detail_url,
                ..Default::default()
            }))
            @if is_admin {
                (sidebar_menu_item(SidebarMenuItem {
                    title: "Edit Program",
                    url: &format!("/programs/{id}/edit/"),
                    ..Default::default()
                }))
                (sidebar_menu_item(SidebarMenuItem {
                    title: "Edit program structure",
                    url: &structure_url,
                    ..Default::default()
                }))
            }
        },
    })
}
