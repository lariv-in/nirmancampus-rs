use maud::{Markup, html};

use lariv_rs::components::{SidebarMenu, SidebarMenuItem, sidebar_menu, sidebar_menu_item};

use crate::routes::{StudentPaymentsDetailRouteTag, StudentPaymentsListRouteTag};
use nirmancampus_common::ui::{item_crumbs, list_crumbs};

pub fn payments_menu() -> Markup {
    sidebar_menu(SidebarMenu {
        title: "Student Payments",
        children: html! {
            (sidebar_menu_item(SidebarMenuItem {
                title: "All Payments",
                url: "/student-payments/",
                ..Default::default()
            }))
        },
    })
}

pub fn payments_crumbs(leaf: &str) -> Markup {
    let list_url = StudentPaymentsListRouteTag.url();
    list_crumbs("Student Payments", &list_url, leaf)
}

pub fn payment_detail_crumbs(id: i64, title: &str, action: Option<&str>) -> Markup {
    let list_url = StudentPaymentsListRouteTag.url();
    let detail_url = StudentPaymentsDetailRouteTag::new(id).url();
    item_crumbs(
        "Student Payments",
        &list_url,
        title,
        Some(&detail_url),
        action,
    )
}

pub fn payment_detail_menu(id: i64, title: &str, is_admin: bool) -> Markup {
    let menu_title = format!("Payment: {title}");
    let detail_url = StudentPaymentsDetailRouteTag::new(id).url();
    let list_url = StudentPaymentsListRouteTag.url();
    sidebar_menu(SidebarMenu {
        title: &menu_title,
        children: html! {
            (sidebar_menu_item(SidebarMenuItem {
                title: "Back to payments",
                url: &list_url,
                ..Default::default()
            }))
            (sidebar_menu_item(SidebarMenuItem {
                title: "Payment Detail",
                url: &detail_url,
                ..Default::default()
            }))
            @if is_admin {
                (sidebar_menu_item(SidebarMenuItem {
                    title: "Edit payment",
                    url: &format!("/student-payments/{id}/edit/"),
                    ..Default::default()
                }))
            }
        },
    })
}
