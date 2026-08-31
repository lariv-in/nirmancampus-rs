use maud::{Markup, html};

use lariv_rs::components::{SidebarMenu, SidebarMenuItem, sidebar_menu, sidebar_menu_item};

use crate::routes::{ExamRegistrationsDetailRouteTag, ExamRegistrationsListRouteTag};
use nirmancampus_common::ui::{item_crumbs, list_crumbs};

pub fn exams_menu() -> Markup {
    sidebar_menu(SidebarMenu {
        title: "Exam Registrations",
        children: html! {
            (sidebar_menu_item(SidebarMenuItem {
                title: "All Registrations",
                url: "/exam-registrations/",
                ..Default::default()
            }))
        },
    })
}

pub fn exams_crumbs(leaf: &str) -> Markup {
    let list_url = ExamRegistrationsListRouteTag.url();
    list_crumbs("Exam Registrations", &list_url, leaf)
}

pub fn exam_detail_crumbs(id: i64, title: &str, action: Option<&str>) -> Markup {
    let list_url = ExamRegistrationsListRouteTag.url();
    let detail_url = ExamRegistrationsDetailRouteTag::new(id).url();
    item_crumbs(
        "Exam Registrations",
        &list_url,
        title,
        Some(&detail_url),
        action,
    )
}

pub fn exam_detail_menu(id: i64, title: &str, is_admin: bool) -> Markup {
    let menu_title = format!("Exam: {title}");
    let detail_url = ExamRegistrationsDetailRouteTag::new(id).url();
    let list_url = ExamRegistrationsListRouteTag.url();
    sidebar_menu(SidebarMenu {
        title: &menu_title,
        children: html! {
            (sidebar_menu_item(SidebarMenuItem {
                title: "Back to registrations",
                url: &list_url,
                ..Default::default()
            }))
            (sidebar_menu_item(SidebarMenuItem {
                title: "Registration Detail",
                url: &detail_url,
                ..Default::default()
            }))
            @if is_admin {
                (sidebar_menu_item(SidebarMenuItem {
                    title: "Edit registration",
                    url: &format!("/exam-registrations/{id}/edit/"),
                    ..Default::default()
                }))
            }
        },
    })
}
