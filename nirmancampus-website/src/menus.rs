use maud::{Markup, html};

use lariv_rs::components::{SidebarMenu, SidebarMenuItem, sidebar_menu, sidebar_menu_item};

use crate::routes::{
    WebsiteAppLandingRouteTag, WebsiteImportantLinksListRouteTag,
    WebsiteStudentZoneItemsListRouteTag, WebsiteStudentZoneSectionsListRouteTag,
    WebsiteTblfeeListRouteTag,
};
use nirmancampus_common::ui::{current_crumb, list_crumbs, nested_crumbs};

pub fn website_menu() -> Markup {
    sidebar_menu(SidebarMenu {
        title: "Website",
        children: html! {
            (sidebar_menu_item(SidebarMenuItem {
                title: "Home",
                url: "/website/",
                ..Default::default()
            }))
            (sidebar_menu_item(SidebarMenuItem {
                title: "Student Zone Sections",
                url: "/website/student-zone/",
                ..Default::default()
            }))
            (sidebar_menu_item(SidebarMenuItem {
                title: "Student Zone Items",
                url: "/website/student-zone/items/",
                ..Default::default()
            }))
            (sidebar_menu_item(SidebarMenuItem {
                title: "Fee records",
                url: "/website/tblfee/",
                ..Default::default()
            }))
            (sidebar_menu_item(SidebarMenuItem {
                title: "Important Links",
                url: "/website/important-links/",
                ..Default::default()
            }))
            (sidebar_menu_item(SidebarMenuItem {
                title: "Contact page (PDF)",
                url: "/website/contact-page/settings/1/",
                ..Default::default()
            }))
        },
    })
}

pub fn website_home_crumbs() -> Markup {
    current_crumb("Website")
}

pub fn website_crumbs(leaf: &str) -> Markup {
    let home = WebsiteAppLandingRouteTag.url();
    list_crumbs("Website", &home, leaf)
}

pub fn important_links_crumbs() -> Markup {
    website_crumbs("Important Links")
}

pub fn important_link_detail_crumbs(title: &str) -> Markup {
    let home = WebsiteAppLandingRouteTag.url();
    let list_url = WebsiteImportantLinksListRouteTag.url();
    nested_crumbs("Website", &home, "Important Links", &list_url, title)
}

pub fn student_zone_sections_crumbs() -> Markup {
    website_crumbs("Student Zone Sections")
}

pub fn student_zone_section_detail_crumbs(title: &str) -> Markup {
    let home = WebsiteAppLandingRouteTag.url();
    let list_url = WebsiteStudentZoneSectionsListRouteTag.url();
    nested_crumbs("Website", &home, "Student Zone Sections", &list_url, title)
}

pub fn student_zone_items_crumbs() -> Markup {
    website_crumbs("Student Zone Items")
}

pub fn student_zone_item_detail_crumbs(title: &str) -> Markup {
    let home = WebsiteAppLandingRouteTag.url();
    let list_url = WebsiteStudentZoneItemsListRouteTag.url();
    nested_crumbs("Website", &home, "Student Zone Items", &list_url, title)
}

pub fn tblfee_crumbs() -> Markup {
    website_crumbs("Fee records")
}

pub fn tblfee_detail_crumbs(leaf: &str) -> Markup {
    let home = WebsiteAppLandingRouteTag.url();
    let list_url = WebsiteTblfeeListRouteTag.url();
    nested_crumbs("Website", &home, "Fee records", &list_url, leaf)
}

pub fn contact_page_crumbs() -> Markup {
    website_crumbs("Contact page")
}
