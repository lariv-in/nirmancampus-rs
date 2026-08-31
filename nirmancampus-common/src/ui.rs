//! Shared Maud scaffold helpers for Nirmancampus plugins.

use maud::{html, Markup};

use lariv_rs::components::{
    swap::AppLayoutKey, breadcrumbs, Crumb, FieldLink, FieldManyToMany, FieldText, LayoutMain,
    LayoutSidebar, PaginationPage, ShellChrome, ShellScaffold, SidebarMenu, SidebarMenuItem,
    SwapKey, TablePagination, field_link, field_many_to_many, field_text, layout_main,
    layout_sidebar, pagination_pages, shell_scaffold, sidebar_menu, sidebar_menu_item,
    table_pagination, table_pagination_picker,
};
use lariv_rs::plugins::filesystem::routes::VNodeDetailRouteTag;

pub const APP_TITLE: &str = "Nirman Campus";

/// Single current-page crumb (app home / landing).
pub fn current_crumb(label: &str) -> Markup {
    breadcrumbs(&[Crumb {
        label,
        href: None,
    }])
}

/// Linked section › current leaf (list pages).
pub fn list_crumbs(section: &str, section_href: &str, leaf: &str) -> Markup {
    breadcrumbs(&[
        Crumb {
            label: section,
            href: Some(section_href),
        },
        Crumb {
            label: leaf,
            href: None,
        },
    ])
}

/// Linked section › item, optionally › action (detail / nested pages).
pub fn item_crumbs(
    section: &str,
    section_href: &str,
    name: &str,
    item_href: Option<&str>,
    action: Option<&str>,
) -> Markup {
    match (action, item_href) {
        (Some(act), Some(href)) => breadcrumbs(&[
            Crumb {
                label: section,
                href: Some(section_href),
            },
            Crumb {
                label: name,
                href: Some(href),
            },
            Crumb {
                label: act,
                href: None,
            },
        ]),
        (Some(act), None) => breadcrumbs(&[
            Crumb {
                label: section,
                href: Some(section_href),
            },
            Crumb {
                label: name,
                href: None,
            },
            Crumb {
                label: act,
                href: None,
            },
        ]),
        (None, _) => breadcrumbs(&[
            Crumb {
                label: section,
                href: Some(section_href),
            },
            Crumb {
                label: name,
                href: None,
            },
        ]),
    }
}

/// Root › section › current leaf (website sub-pages).
pub fn nested_crumbs(
    root: &str,
    root_href: &str,
    section: &str,
    section_href: &str,
    leaf: &str,
) -> Markup {
    breadcrumbs(&[
        Crumb {
            label: root,
            href: Some(root_href),
        },
        Crumb {
            label: section,
            href: Some(section_href),
        },
        Crumb {
            label: leaf,
            href: None,
        },
    ])
}

/// Shared Students hub sidebar (Go `students.StudentMenu` plus plugin patches).
pub fn students_hub_menu() -> Markup {
    sidebar_menu(SidebarMenu {
        title: "Students",
        children: html! {
            (sidebar_menu_item(SidebarMenuItem {
                title: "All Students",
                url: "/students/",
                ..Default::default()
            }))
            (sidebar_menu_item(SidebarMenuItem {
                title: "Academic Records",
                url: "/academic-records/",
                ..Default::default()
            }))
            (sidebar_menu_item(SidebarMenuItem {
                title: "All Payments",
                url: "/student-payments/",
                ..Default::default()
            }))
            (sidebar_menu_item(SidebarMenuItem {
                title: "All Exam Registrations",
                url: "/exam-registrations/",
                ..Default::default()
            }))
            (sidebar_menu_item(SidebarMenuItem {
                title: "All Assignment Submissions",
                url: "/assignment-submissions/",
                ..Default::default()
            }))
        },
    })
}

pub fn app_scaffold(
    title: &str,
    chrome: &ShellChrome,
    sidebar: Markup,
    crumbs: Markup,
    body: Markup,
) -> Markup {
    shell_scaffold(ShellScaffold {
        title,
        registry_head: chrome.head.clone(),
        topbar_items: chrome.topbar_items.clone(),
        right_sidebar: chrome.right_sidebar.clone(),
        sidebar,
        breadcrumbs: crumbs,
        body,
        ..Default::default()
    })
}

pub fn scaffold_pane(
    sidebar: Markup,
    crumbs: Markup,
    body: Markup,
) -> lariv_rs::components::AppLayoutHtml {
    layout_sidebar(LayoutSidebar {
        sidebar,
        breadcrumbs: crumbs,
        content: body,
    })
}

pub fn scaffold_main(crumbs: Markup, body: Markup) -> lariv_rs::components::MainContentHtml {
    layout_main(LayoutMain {
        breadcrumbs: crumbs,
        content: body,
    })
}

pub fn render_pagination<K: SwapKey>(
    path_and_query: &str,
    number: u32,
    num_pages: u32,
    push_url: bool,
) -> Markup {
    let owned = pagination_pages(path_and_query, number, num_pages, push_url);
    let pages: Vec<PaginationPage<'_>> = owned
        .iter()
        .map(|(ellipsis, url, push_url, active, label)| PaginationPage {
            ellipsis: *ellipsis,
            url: url.as_str(),
            push_url: *push_url,
            active: *active,
            label: label.as_str(),
        })
        .collect();
    table_pagination(TablePagination {
        pages: &pages,
        hx_target: K::SELECTOR,
    })
}

pub fn render_picker_pagination<M: SwapKey>(
    path_and_query: &str,
    number: u32,
    num_pages: u32,
) -> Markup {
    let owned = pagination_pages(path_and_query, number, num_pages, false);
    let pages: Vec<PaginationPage<'_>> = owned
        .iter()
        .map(|(ellipsis, url, push_url, active, label)| PaginationPage {
            ellipsis: *ellipsis,
            url: url.as_str(),
            push_url: *push_url,
            active: *active,
            label: label.as_str(),
        })
        .collect();
    table_pagination_picker(TablePagination {
        pages: &pages,
        hx_target: M::SELECTOR,
    })
}

pub fn yes_no(v: bool) -> &'static str {
    if v { "Yes" } else { "No" }
}

pub fn empty_dash(s: &str) -> &str {
    if s.trim().is_empty() { "—" } else { s }
}

/// Read-only foreign-key field: a link when both label and href are set.
pub fn field_related(label: &str, href: &str) -> Markup {
    if label.trim().is_empty() {
        field_text(FieldText {
            value: "—",
            classes: "",
        })
    } else if href.trim().is_empty() {
        field_text(FieldText {
            value: label,
            classes: "",
        })
    } else {
        field_link(FieldLink {
            href,
            label,
            classes: "",
        })
    }
}

/// Read-only many-to-many field: linked chips, or a dash when empty.
pub fn field_related_many(items: &[(&str, Option<&str>)]) -> Markup {
    if items.is_empty() {
        field_text(FieldText {
            value: "—",
            classes: "",
        })
    } else {
        field_many_to_many(FieldManyToMany {
            items,
            classes: "w-full",
        })
    }
}

/// Read-only filesystem-node foreign key: file name linking to the vnode.
pub fn field_vnode(id: i64, name: &str) -> Markup {
    if id <= 0 {
        return field_text(FieldText {
            value: "—",
            classes: "",
        });
    }
    let href = VNodeDetailRouteTag::new(id).url();
    let label = if name.trim().is_empty() {
        format!("File #{id}")
    } else {
        name.to_string()
    };
    field_related(&label, &href)
}

/// Read-only filesystem-node many-to-many: named chips linking to each vnode.
pub fn field_vnode_many(items: &[(i64, &str)]) -> Markup {
    if items.is_empty() {
        return field_text(FieldText {
            value: "—",
            classes: "",
        });
    }
    let urls: Vec<String> = items
        .iter()
        .map(|(id, _)| VNodeDetailRouteTag::new(*id).url())
        .collect();
    let pairs: Vec<(&str, Option<&str>)> = items
        .iter()
        .zip(urls.iter())
        .map(|((_, name), url)| (*name, Some(url.as_str())))
        .collect();
    field_related_many(&pairs)
}

#[derive(Clone)]
pub struct SessionOption {
    pub id: i64,
    pub name: String,
}

/// Admission-session selector that writes the Lariv `environment` JSON cookie.
pub fn session_environment_selector(
    cookie_key: &str,
    sessions: &[SessionOption],
    selected_id: i64,
) -> Markup {
    let selected = if selected_id > 0 {
        selected_id.to_string()
    } else {
        String::new()
    };
    let reload_js = format!(
        "htmx.ajax('GET',window.location.pathname+window.location.search,{{target:'{target}',select:'{target}',swap:'outerHTML',pushUrl:false}})",
        target = AppLayoutKey::SELECTOR,
    );
    let on_change = format!(
        r#"(function(){{
        var env={{}};
        try{{
            var c=document.cookie.split('; ').find(function(r){{return r.startsWith('environment=')}});
            if(c) env=JSON.parse(decodeURIComponent(c.split('=').slice(1).join('=')));
        }}catch(e){{}}
        env[{key:?}]=this.value;
        document.cookie='environment='+encodeURIComponent(JSON.stringify(env))+'; path=/';
        {reload_js};
    }}).call(this)"#,
        key = cookie_key,
    );
    html! {
        div class="my-1 w-full" {
            label class="label text-sm font-bold" { "Admission session" }
            select class="select select-bordered w-full" name=(cookie_key) onchange=(on_change) {
                option value="" selected[selected.is_empty()] { "—" }
                @for s in sessions {
                    option value=(s.id.to_string()) selected[selected == s.id.to_string()] {
                        (s.name.as_str())
                    }
                }
            }
        }
    }
}
