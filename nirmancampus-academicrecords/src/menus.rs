use maud::{Markup, html};

use lariv_rs::components::{SidebarMenu, SidebarMenuItem, sidebar_menu, sidebar_menu_item};

use crate::routes::{AcademicRecordsDetailRouteTag, AcademicRecordsListRouteTag};
use nirmancampus_common::ui::{item_crumbs, list_crumbs};

pub fn academic_records_menu() -> Markup {
    sidebar_menu(SidebarMenu {
        title: "Academic Records",
        children: html! {
            (sidebar_menu_item(SidebarMenuItem {
                title: "All Academic Records",
                url: "/academic-records/",
                ..Default::default()
            }))
        },
    })
}

pub fn academic_records_crumbs(leaf: &str) -> Markup {
    let list_url = AcademicRecordsListRouteTag.url();
    list_crumbs("Academic Records", &list_url, leaf)
}

pub fn academic_record_detail_crumbs(id: i64, title: &str, action: Option<&str>) -> Markup {
    let list_url = AcademicRecordsListRouteTag.url();
    let detail_url = AcademicRecordsDetailRouteTag::new(id).url();
    item_crumbs(
        "Academic Records",
        &list_url,
        title,
        Some(&detail_url),
        action,
    )
}

pub fn academic_record_detail_menu(id: i64, title: &str, is_admin: bool) -> Markup {
    let menu_title = format!("Academic record: {title}");
    let detail_url = AcademicRecordsDetailRouteTag::new(id).url();
    let list_url = AcademicRecordsListRouteTag.url();
    sidebar_menu(SidebarMenu {
        title: &menu_title,
        children: html! {
            (sidebar_menu_item(SidebarMenuItem {
                title: "Back to academic records",
                url: &list_url,
                ..Default::default()
            }))
            (sidebar_menu_item(SidebarMenuItem {
                title: "Record Detail",
                url: &detail_url,
                ..Default::default()
            }))
            @if is_admin {
                (sidebar_menu_item(SidebarMenuItem {
                    title: "Edit Academic Record",
                    url: &format!("/academic-records/{id}/edit/"),
                    ..Default::default()
                }))
            }
        },
    })
}
