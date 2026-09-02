pub mod public;

pub use public::{
    ContactPage, HomeAnnouncement, HomePage, ImportantLinkItem, PrivacyPage, ProgramsPage,
    PublicProgram, PublicShell, StudentZonePage, StudentZonePublicItem, StudentZonePublicSection,
};

use frunk::Generic;
use maud::{Markup, html};

use lariv_rs::{
    components::{
        ButtonClear, ButtonLink, ButtonSubmit, DeleteConfirmation, DetailHeader, FieldText,
        FormOpts, ObjectList, ShellChrome, SlotCapability, SlotRegistrar, SwapKey,
        TableButtonFilter, TableColumnHeader, TableRow, button_clear, button_link,
        button_modal_form_route, button_submit, container_column, container_row,
        data_table_list_grid_with_subtitle, data_table_list_refresh, detail, detail_header,
        field_text, form, form_hx_get_picker_route, form_hx_get_route, form_hx_post_main,
        form_hx_post_selector, form_hx_post_url, hx_nav_app_layout, label, modal, modal_keyed,
        row_attr_select, table_button_filter, table_create_button,
    },
    html_form::{FormCtx, HtmlForm},
    http::ProvideRequestCaps,
    picker::RenderPickerSelect,
    template::{RenderTemplate, TemplateCapability, TemplateOf, TemplateRegistrar},
    web::{modal_create_post_query, modal_edit_post_url},
};

use crate::forms::{
    ContactPageSettingsForm, ContactPageSettingsFormField, ImportantLinkFilterForm,
    ImportantLinkFilterFormField, ImportantLinkForm, ImportantLinkFormField,
    StudentZoneItemFilterForm, StudentZoneItemFilterFormField, StudentZoneItemForm,
    StudentZoneItemFormField, StudentZoneSectionFilterForm, StudentZoneSectionFilterFormField,
    StudentZoneSectionForm, StudentZoneSectionFormField,
};
use crate::keys::{
    ImportantLinkCreateModalKey, ImportantLinkDeleteModalKey, ImportantLinkEditModalKey,
    ImportantLinksTableKey, StudentZoneItemCreateModalKey, StudentZoneItemDeleteModalKey,
    StudentZoneItemEditModalKey, StudentZoneItemTableKey, StudentZoneSectionCreateModalKey,
    StudentZoneSectionDeleteModalKey, StudentZoneSectionEditModalKey,
    StudentZoneSectionSelectModalKey, StudentZoneSectionSelectTableKey, StudentZoneSectionTableKey,
};
use crate::menus::{
    contact_page_crumbs, important_link_detail_crumbs, important_links_crumbs,
    student_zone_item_detail_crumbs, student_zone_items_crumbs, student_zone_section_detail_crumbs,
    student_zone_sections_crumbs, website_home_crumbs,
    website_menu,
};
use crate::routes::{
    WebsiteContactPageSettingsEditPostRouteTag, WebsiteImportantLinksCreatePostRouteTag,
    WebsiteImportantLinksDeleteGetRouteTag, WebsiteImportantLinksDeletePostRouteTag,
    WebsiteImportantLinksDetailRouteTag, WebsiteImportantLinksEditGetRouteTag,
    WebsiteImportantLinksEditPostRouteTag, WebsiteImportantLinksListRouteTag,
    WebsiteStudentZoneItemsCreatePostRouteTag, WebsiteStudentZoneItemsDeleteGetRouteTag,
    WebsiteStudentZoneItemsDeletePostRouteTag, WebsiteStudentZoneItemsDetailRouteTag,
    WebsiteStudentZoneItemsEditGetRouteTag, WebsiteStudentZoneItemsEditPostRouteTag,
    WebsiteStudentZoneItemsListRouteTag, WebsiteStudentZoneSectionsCreatePostRouteTag,
    WebsiteStudentZoneSectionsDeleteGetRouteTag, WebsiteStudentZoneSectionsDeletePostRouteTag,
    WebsiteStudentZoneSectionsDetailRouteTag, WebsiteStudentZoneSectionsEditGetRouteTag,
    WebsiteStudentZoneSectionsEditPostRouteTag, WebsiteStudentZoneSectionsListRouteTag,
    WebsiteStudentZoneSectionsSelectRouteTag,
};
use nirmancampus_common::ui::{
    APP_TITLE, app_scaffold, empty_dash, field_related, field_vnode, render_pagination,
    render_picker_pagination, scaffold_main, scaffold_pane, yes_no,
};

lariv_rs::define_register_items! {
    plugin: NirmancampusWebsiteTag;
    capability: TemplateCapability;
    trait: TemplateRegistrar;
    method: register_templates;
    wrapper: TemplateOf;
    bounds: [Clone, ProvideRequestCaps, Send, Sync];
    hook: Hook;
    items: [
        HomeIdx: WebsiteHomePageTag => HomePage,
        ProgramsIdx: WebsiteProgramsPageTag => ProgramsPage,
        ContactIdx: WebsiteContactPageTag => ContactPage,
        PrivacyIdx: WebsitePrivacyPageTag => PrivacyPage,
        StudentZonePublicIdx: WebsiteStudentZonePublicPageTag => StudentZonePage,
        LandingIdx: WebsiteLandingPageTag => WebsiteLandingPage,
        ImportantLinkListIdx: ImportantLinkListPageTag => ImportantLinkListPage,
        ImportantLinkDetailIdx: ImportantLinkDetailPageTag => ImportantLinkDetailPage,
        ImportantLinkFormIdx: ImportantLinkFormPageTag => ImportantLinkFormPage,
        SectionListIdx: StudentZoneSectionListPageTag => StudentZoneSectionListPage,
        SectionDetailIdx: StudentZoneSectionDetailPageTag => StudentZoneSectionDetailPage,
        SectionFormIdx: StudentZoneSectionFormPageTag => StudentZoneSectionFormPage,
        SectionSelectIdx: StudentZoneSectionSelectPageTag => StudentZoneSectionSelectPage,
        ItemListIdx: StudentZoneItemListPageTag => StudentZoneItemListPage,
        ItemDetailIdx: StudentZoneItemDetailPageTag => StudentZoneItemDetailPage,
        ItemFormIdx: StudentZoneItemFormPageTag => StudentZoneItemFormPage,
        ContactSettingsIdx: ContactPageSettingsFormPageTag => ContactPageSettingsFormPage,
        ConfirmDeleteIdx: WebsiteConfirmDeletePageTag => ConfirmDeletePage,
    ]
}

lariv_rs::define_register_items! {
    plugin: NirmancampusWebsiteTag;
    capability: SlotCapability;
    trait: SlotRegistrar;
    method: register_slots;
    bounds: [];
    items: [];
    hook: SlotsHook;
}

#[derive(Generic)]
pub struct WebsiteLandingPage {}

impl WebsiteLandingPage {
    fn pane_body(&self) -> Markup {
        html! {
            (container_column("max-w-3xl", html! {
                h1 class="text-2xl font-bold" { "Website Admin" }
                p { "Use the sidebar to navigate." }
                div class="flex gap-2 flex-wrap mt-4" {
                    (button_link(ButtonLink {
                        label: "View Website",
                        href: "/",
                        classes: "btn-outline",
                        ..Default::default()
                    }))
                }
            }))
        }
    }
}

impl lariv_rs::template::RenderAppPane for WebsiteLandingPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        scaffold_pane(website_menu(), website_home_crumbs(), self.pane_body())
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        scaffold_main(website_home_crumbs(), self.pane_body())
    }
}

impl RenderTemplate for WebsiteLandingPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold(
            &format!("Website — {APP_TITLE}"),
            chrome,
            website_menu(),
            website_home_crumbs(),
            self.pane_body(),
        )
    }
}

#[derive(Clone)]
pub struct ImportantLinkRow {
    pub id: i64,
    pub title: String,
    pub order: i64,
    pub is_link: String,
    pub link: String,
}

#[derive(Generic)]
pub struct ImportantLinkListPage {
    pub links: ObjectList<ImportantLinkRow>,
    pub filter_title: String,
    pub path_and_query: String,
    pub is_admin: bool,
}

impl ImportantLinkListPage {
    pub fn render_table(&self) -> Markup {
        let headers = [
            TableColumnHeader {
                key: "Title",
                label: "Title",
                sort_url: None,
                push_url: false,
            },
            TableColumnHeader {
                key: "Order",
                label: "Order",
                sort_url: None,
                push_url: false,
            },
            TableColumnHeader {
                key: "IsLink",
                label: "Is link",
                sort_url: None,
                push_url: false,
            },
            TableColumnHeader {
                key: "Link",
                label: "Link",
                sort_url: None,
                push_url: false,
            },
        ];
        let rows: Vec<TableRow> = self
            .links
            .items
            .iter()
            .map(|l| {
                let order = l.order.to_string();
                TableRow {
                    attrs: hx_nav_app_layout(WebsiteImportantLinksDetailRouteTag::new(l.id)),
                    cells: vec![
                        field_text(FieldText {
                            value: &l.title,
                            classes: "",
                        }),
                        field_text(FieldText {
                            value: &order,
                            classes: "",
                        }),
                        field_text(FieldText {
                            value: &l.is_link,
                            classes: "",
                        }),
                        field_text(FieldText {
                            value: empty_dash(&l.link),
                            classes: "",
                        }),
                    ],
                }
            })
            .collect();
        let actions = html! {
            (table_button_filter(TableButtonFilter {
                panel: form(FormOpts {
                    attrs: form_hx_get_route::<ImportantLinksTableKey, WebsiteImportantLinksListRouteTag>(
                        WebsiteImportantLinksListRouteTag,
                    ),
                    inputs: ImportantLinkFilterForm::render_inputs(
                        &FormCtx::form::<ImportantLinkFilterForm>()
                            .value(ImportantLinkFilterFormField::Title, &self.filter_title),
                    ),
                    actions: html! {
                        (container_row("flex gap-2", html! {
                            (button_submit(ButtonSubmit { label: "Apply Filters", ..Default::default() }))
                            (button_clear(ButtonClear { label: "Clear", ..Default::default() }))
                        }))
                    },
                    ..Default::default()
                }),
                ..Default::default()
            }))
            @if self.is_admin {
                (table_create_button::<ImportantLinksTableKey, ImportantLinkCreateModalKey>(
                    Some("plus"),
                    "btn-square btn-outline btn-sm",
                ))
            }
        };
        data_table_list_grid_with_subtitle::<ImportantLinksTableKey>(
            "Important Links",
            "Public website links and downloads",
            actions,
            &headers,
            &rows,
            render_pagination::<ImportantLinksTableKey>(
                &self.path_and_query,
                self.links.number,
                self.links.num_pages,
                true,
            ),
        )
    }
}

impl lariv_rs::template::RenderAppPane for ImportantLinkListPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        scaffold_pane(
            website_menu(),
            important_links_crumbs(),
            self.render_table(),
        )
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        scaffold_main(important_links_crumbs(), self.render_table())
    }
}

impl RenderTemplate for ImportantLinkListPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold(
            &format!("Important Links — {APP_TITLE}"),
            chrome,
            website_menu(),
            important_links_crumbs(),
            self.render_table(),
        )
    }
}

#[derive(Generic)]
pub struct ImportantLinkDetailPage {
    pub id: i64,
    pub title: String,
    pub order: i64,
    pub is_link: bool,
    pub link: String,
    pub file_id: i64,
    pub file_name: String,
    pub is_admin: bool,
}

impl ImportantLinkDetailPage {
    fn pane_body(&self) -> Markup {
        let order = self.order.to_string();
        detail(html! {
            (container_column("", html! {
                (detail_header(DetailHeader {
                    title: &self.title,
                    actions: html! {
                        @if self.is_admin {
                            (button_modal_form_route(
                                WebsiteImportantLinksEditGetRouteTag::new(self.id),
                                WebsiteImportantLinksEditPostRouteTag::new(self.id),
                                "Edit",
                                ImportantLinkEditModalKey::ID,
                                "btn btn-outline btn-sm",
                            ))
                        }
                    },
                }))
                (label("Title", field_text(FieldText { value: &self.title, classes: "" })))
                (label("Order", field_text(FieldText { value: &order, classes: "" })))
                (label("Is link", field_text(FieldText { value: yes_no(self.is_link), classes: "" })))
                (label("Link", field_text(FieldText { value: empty_dash(&self.link), classes: "" })))
                (label("File", field_vnode(self.file_id, &self.file_name)))
            }))
        })
    }
}

impl lariv_rs::template::RenderAppPane for ImportantLinkDetailPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        scaffold_pane(
            website_menu(),
            important_link_detail_crumbs(&self.title),
            self.pane_body(),
        )
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        scaffold_main(important_link_detail_crumbs(&self.title), self.pane_body())
    }
}

impl RenderTemplate for ImportantLinkDetailPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold(
            &format!("{} — {APP_TITLE}", self.title),
            chrome,
            website_menu(),
            important_link_detail_crumbs(&self.title),
            self.pane_body(),
        )
    }
}

#[derive(Generic)]
pub struct ImportantLinkFormPage {
    pub id: i64,
    pub title: String,
    pub order: i64,
    pub is_link: bool,
    pub link: String,
    pub file_id: i64,
    pub file_display: String,
    pub error: String,
    pub form_name: String,
    pub refresh_table: String,
    pub target_input: String,
}

impl RenderTemplate for ImportantLinkFormPage {
    fn render(&self, _chrome: &ShellChrome) -> Markup {
        let is_create = self.id == 0;
        let form_attrs = if is_create {
            form_hx_post_url::<ImportantLinkCreateModalKey>(&modal_create_post_query(
                WebsiteImportantLinksCreatePostRouteTag,
                &self.form_name,
                &self.refresh_table,
                &self.target_input,
            ))
        } else {
            form_hx_post_url::<ImportantLinkEditModalKey>(&modal_edit_post_url(
                WebsiteImportantLinksEditPostRouteTag::new(self.id),
                &self.form_name,
            ))
        };
        let order = self.order.to_string();
        let file_id = if self.file_id > 0 {
            self.file_id.to_string()
        } else {
            String::new()
        };
        let body = form(FormOpts {
            attrs: form_attrs,
            title: if is_create {
                "Create Important Link"
            } else {
                "Edit Important Link"
            },
            subtitle: if is_create {
                "Create a new important link entry"
            } else {
                "Update this important link"
            },
            form_error: Some(self.error.as_str()).filter(|e| !e.is_empty()),
            inputs: ImportantLinkForm::render_inputs(
                &FormCtx::form::<ImportantLinkForm>()
                    .value(ImportantLinkFormField::Title, &self.title)
                    .value(ImportantLinkFormField::Order, &order)
                    .checked(ImportantLinkFormField::IsLink, self.is_link)
                    .value(ImportantLinkFormField::Link, &self.link)
                    .value(ImportantLinkFormField::FileId, &file_id)
                    .display(ImportantLinkFormField::FileId, &self.file_display),
            ),
            actions: html! {
                (button_submit(ButtonSubmit { label: "Save", ..Default::default() }))
                @if !is_create {
                    (button_modal_form_route(
                        WebsiteImportantLinksDeleteGetRouteTag::new(self.id),
                        WebsiteImportantLinksDeletePostRouteTag::new(self.id),
                        "Delete",
                        ImportantLinkDeleteModalKey::ID,
                        "btn-error",
                    ))
                }
            },
            ..Default::default()
        });
        if is_create {
            modal_keyed::<ImportantLinkCreateModalKey>("", body)
        } else {
            modal_keyed::<ImportantLinkEditModalKey>("", body)
        }
    }
}

#[derive(Clone)]
pub struct StudentZoneSectionRow {
    pub id: i64,
    pub title: String,
    pub order: i64,
}

#[derive(Generic)]
pub struct StudentZoneSectionListPage {
    pub sections: ObjectList<StudentZoneSectionRow>,
    pub filter_title: String,
    pub path_and_query: String,
    pub is_admin: bool,
}

impl StudentZoneSectionListPage {
    pub fn render_table(&self) -> Markup {
        let headers = [
            TableColumnHeader {
                key: "Title",
                label: "Title",
                sort_url: None,
                push_url: false,
            },
            TableColumnHeader {
                key: "Order",
                label: "Order",
                sort_url: None,
                push_url: false,
            },
        ];
        let rows: Vec<TableRow> = self
            .sections
            .items
            .iter()
            .map(|s| {
                let order = s.order.to_string();
                TableRow {
                    attrs: hx_nav_app_layout(WebsiteStudentZoneSectionsDetailRouteTag::new(s.id)),
                    cells: vec![
                        field_text(FieldText {
                            value: &s.title,
                            classes: "",
                        }),
                        field_text(FieldText {
                            value: &order,
                            classes: "",
                        }),
                    ],
                }
            })
            .collect();
        let actions = html! {
            (table_button_filter(TableButtonFilter {
                panel: form(FormOpts {
                    attrs: form_hx_get_route::<
                        StudentZoneSectionTableKey,
                        WebsiteStudentZoneSectionsListRouteTag,
                    >(WebsiteStudentZoneSectionsListRouteTag),
                    inputs: StudentZoneSectionFilterForm::render_inputs(
                        &FormCtx::form::<StudentZoneSectionFilterForm>()
                            .value(StudentZoneSectionFilterFormField::Title, &self.filter_title),
                    ),
                    actions: html! {
                        (container_row("flex gap-2", html! {
                            (button_submit(ButtonSubmit { label: "Apply Filters", ..Default::default() }))
                            (button_clear(ButtonClear { label: "Clear", ..Default::default() }))
                        }))
                    },
                    ..Default::default()
                }),
                ..Default::default()
            }))
            @if self.is_admin {
                (table_create_button::<StudentZoneSectionTableKey, StudentZoneSectionCreateModalKey>(
                    Some("plus"),
                    "btn-square btn-outline btn-sm",
                ))
            }
        };
        data_table_list_grid_with_subtitle::<StudentZoneSectionTableKey>(
            "Student Zone Sections",
            "Group student-zone downloads and links",
            actions,
            &headers,
            &rows,
            render_pagination::<StudentZoneSectionTableKey>(
                &self.path_and_query,
                self.sections.number,
                self.sections.num_pages,
                true,
            ),
        )
    }
}

impl lariv_rs::template::RenderAppPane for StudentZoneSectionListPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        scaffold_pane(
            website_menu(),
            student_zone_sections_crumbs(),
            self.render_table(),
        )
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        scaffold_main(student_zone_sections_crumbs(), self.render_table())
    }
}

impl RenderTemplate for StudentZoneSectionListPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold(
            &format!("Student Zone Sections — {APP_TITLE}"),
            chrome,
            website_menu(),
            student_zone_sections_crumbs(),
            self.render_table(),
        )
    }
}

#[derive(Generic)]
pub struct StudentZoneSectionDetailPage {
    pub id: i64,
    pub title: String,
    pub order: i64,
    pub is_admin: bool,
}

impl StudentZoneSectionDetailPage {
    fn pane_body(&self) -> Markup {
        let order = self.order.to_string();
        detail(html! {
            (container_column("", html! {
                (detail_header(DetailHeader {
                    title: &self.title,
                    actions: html! {
                        @if self.is_admin {
                            (button_modal_form_route(
                                WebsiteStudentZoneSectionsEditGetRouteTag::new(self.id),
                                WebsiteStudentZoneSectionsEditPostRouteTag::new(self.id),
                                "Edit",
                                StudentZoneSectionEditModalKey::ID,
                                "btn btn-outline btn-sm",
                            ))
                        }
                    },
                }))
                (label("Title", field_text(FieldText { value: &self.title, classes: "" })))
                (label("Order", field_text(FieldText { value: &order, classes: "" })))
            }))
        })
    }
}

impl lariv_rs::template::RenderAppPane for StudentZoneSectionDetailPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        scaffold_pane(
            website_menu(),
            student_zone_section_detail_crumbs(&self.title),
            self.pane_body(),
        )
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        scaffold_main(
            student_zone_section_detail_crumbs(&self.title),
            self.pane_body(),
        )
    }
}

impl RenderTemplate for StudentZoneSectionDetailPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold(
            &format!("{} — {APP_TITLE}", self.title),
            chrome,
            website_menu(),
            student_zone_section_detail_crumbs(&self.title),
            self.pane_body(),
        )
    }
}

#[derive(Generic)]
pub struct StudentZoneSectionFormPage {
    pub id: i64,
    pub title: String,
    pub order: i64,
    pub error: String,
    pub form_name: String,
    pub refresh_table: String,
    pub target_input: String,
}

impl RenderTemplate for StudentZoneSectionFormPage {
    fn render(&self, _chrome: &ShellChrome) -> Markup {
        let is_create = self.id == 0;
        let form_attrs = if is_create {
            form_hx_post_url::<StudentZoneSectionCreateModalKey>(&modal_create_post_query(
                WebsiteStudentZoneSectionsCreatePostRouteTag,
                &self.form_name,
                &self.refresh_table,
                &self.target_input,
            ))
        } else {
            form_hx_post_url::<StudentZoneSectionEditModalKey>(&modal_edit_post_url(
                WebsiteStudentZoneSectionsEditPostRouteTag::new(self.id),
                &self.form_name,
            ))
        };
        let order = self.order.to_string();
        let body = form(FormOpts {
            attrs: form_attrs,
            title: if is_create {
                "Create section"
            } else {
                "Edit section"
            },
            subtitle: if is_create {
                "Add a student-zone section"
            } else {
                "Update this section"
            },
            form_error: Some(self.error.as_str()).filter(|e| !e.is_empty()),
            inputs: StudentZoneSectionForm::render_inputs(
                &FormCtx::form::<StudentZoneSectionForm>()
                    .value(StudentZoneSectionFormField::Title, &self.title)
                    .value(StudentZoneSectionFormField::Order, &order),
            ),
            actions: html! {
                (button_submit(ButtonSubmit { label: "Save", ..Default::default() }))
                @if !is_create {
                    (button_modal_form_route(
                        WebsiteStudentZoneSectionsDeleteGetRouteTag::new(self.id),
                        WebsiteStudentZoneSectionsDeletePostRouteTag::new(self.id),
                        "Delete",
                        StudentZoneSectionDeleteModalKey::ID,
                        "btn-error",
                    ))
                }
            },
            ..Default::default()
        });
        if is_create {
            modal_keyed::<StudentZoneSectionCreateModalKey>("", body)
        } else {
            modal_keyed::<StudentZoneSectionEditModalKey>("", body)
        }
    }
}

#[derive(Generic)]
pub struct StudentZoneSectionSelectPage {
    pub sections: ObjectList<StudentZoneSectionRow>,
    pub filter_title: String,
    pub path_and_query: String,
    pub target_input: String,
}

impl RenderPickerSelect<StudentZoneSectionSelectTableKey, StudentZoneSectionSelectModalKey>
    for StudentZoneSectionSelectPage
{
    fn render_table(&self) -> Markup {
        let headers = [TableColumnHeader {
            key: "Title",
            label: "Title",
            sort_url: None,
            push_url: false,
        }];
        let target = if self.target_input.is_empty() {
            "StudentZoneSectionID"
        } else {
            self.target_input.as_str()
        };
        let rows: Vec<TableRow> = self
            .sections
            .items
            .iter()
            .map(|s| TableRow {
                attrs: row_attr_select(target, &s.id.to_string(), &s.title),
                cells: vec![field_text(FieldText {
                    value: &s.title,
                    classes: "",
                })],
            })
            .collect();
        data_table_list_refresh::<StudentZoneSectionSelectTableKey>(
            "Select Section",
            table_button_filter(TableButtonFilter {
                panel: form(FormOpts {
                    attrs: form_hx_get_picker_route::<
                        StudentZoneSectionSelectTableKey,
                        StudentZoneSectionSelectModalKey,
                        WebsiteStudentZoneSectionsSelectRouteTag,
                    >(WebsiteStudentZoneSectionsSelectRouteTag)
                    .set("hx-push-url", "false"),
                    inputs: html! {
                        (StudentZoneSectionFilterForm::render_inputs(
                            &FormCtx::form::<StudentZoneSectionFilterForm>()
                                .value(StudentZoneSectionFilterFormField::Title, &self.filter_title),
                        ))
                        input type="hidden" name="target_input" value=(self.target_input) {}
                    },
                    actions: html! { (button_submit(ButtonSubmit { label: "Apply", ..Default::default() })) },
                    ..Default::default()
                }),
                ..Default::default()
            }),
            &headers,
            &rows,
            render_picker_pagination::<StudentZoneSectionSelectModalKey>(
                &self.path_and_query,
                self.sections.number,
                self.sections.num_pages,
            ),
            &self.path_and_query,
        )
    }
}

impl RenderTemplate for StudentZoneSectionSelectPage {
    fn render(&self, _chrome: &ShellChrome) -> Markup {
        self.render_modal().into_inner()
    }
}

#[derive(Clone)]
pub struct StudentZoneItemRow {
    pub id: i64,
    pub title: String,
    pub is_link: String,
    pub section: String,
}

#[derive(Generic)]
pub struct StudentZoneItemListPage {
    pub items: ObjectList<StudentZoneItemRow>,
    pub filter_title: String,
    pub path_and_query: String,
    pub is_admin: bool,
}

impl StudentZoneItemListPage {
    pub fn render_table(&self) -> Markup {
        let headers = [
            TableColumnHeader {
                key: "Title",
                label: "Title",
                sort_url: None,
                push_url: false,
            },
            TableColumnHeader {
                key: "IsLink",
                label: "Is link",
                sort_url: None,
                push_url: false,
            },
            TableColumnHeader {
                key: "Section",
                label: "Section",
                sort_url: None,
                push_url: false,
            },
        ];
        let rows: Vec<TableRow> = self
            .items
            .items
            .iter()
            .map(|i| TableRow {
                attrs: hx_nav_app_layout(WebsiteStudentZoneItemsDetailRouteTag::new(i.id)),
                cells: vec![
                    field_text(FieldText {
                        value: &i.title,
                        classes: "",
                    }),
                    field_text(FieldText {
                        value: &i.is_link,
                        classes: "",
                    }),
                    field_text(FieldText {
                        value: empty_dash(&i.section),
                        classes: "",
                    }),
                ],
            })
            .collect();
        let actions = html! {
            (table_button_filter(TableButtonFilter {
                panel: form(FormOpts {
                    attrs: form_hx_get_route::<
                        StudentZoneItemTableKey,
                        WebsiteStudentZoneItemsListRouteTag,
                    >(WebsiteStudentZoneItemsListRouteTag),
                    inputs: StudentZoneItemFilterForm::render_inputs(
                        &FormCtx::form::<StudentZoneItemFilterForm>()
                            .value(StudentZoneItemFilterFormField::Title, &self.filter_title),
                    ),
                    actions: html! {
                        (container_row("flex gap-2", html! {
                            (button_submit(ButtonSubmit { label: "Apply Filters", ..Default::default() }))
                            (button_clear(ButtonClear { label: "Clear", ..Default::default() }))
                        }))
                    },
                    ..Default::default()
                }),
                ..Default::default()
            }))
            @if self.is_admin {
                (table_create_button::<StudentZoneItemTableKey, StudentZoneItemCreateModalKey>(
                    Some("plus"),
                    "btn-square btn-outline btn-sm",
                ))
            }
        };
        data_table_list_grid_with_subtitle::<StudentZoneItemTableKey>(
            "Student Zone Items",
            "Links and files shown in Student Zone",
            actions,
            &headers,
            &rows,
            render_pagination::<StudentZoneItemTableKey>(
                &self.path_and_query,
                self.items.number,
                self.items.num_pages,
                true,
            ),
        )
    }
}

impl lariv_rs::template::RenderAppPane for StudentZoneItemListPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        scaffold_pane(
            website_menu(),
            student_zone_items_crumbs(),
            self.render_table(),
        )
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        scaffold_main(student_zone_items_crumbs(), self.render_table())
    }
}

impl RenderTemplate for StudentZoneItemListPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold(
            &format!("Student Zone Items — {APP_TITLE}"),
            chrome,
            website_menu(),
            student_zone_items_crumbs(),
            self.render_table(),
        )
    }
}

#[derive(Generic)]
pub struct StudentZoneItemDetailPage {
    pub id: i64,
    pub title: String,
    pub is_link: bool,
    pub link: String,
    pub file_id: i64,
    pub file_name: String,
    pub section: String,
    pub section_id: i64,
    pub is_admin: bool,
}

impl StudentZoneItemDetailPage {
    fn pane_body(&self) -> Markup {
        let section_href = if self.section_id > 0 {
            WebsiteStudentZoneSectionsDetailRouteTag::new(self.section_id).url()
        } else {
            String::new()
        };
        detail(html! {
            (container_column("", html! {
                (detail_header(DetailHeader {
                    title: &self.title,
                    actions: html! {
                        @if self.is_admin {
                            (button_modal_form_route(
                                WebsiteStudentZoneItemsEditGetRouteTag::new(self.id),
                                WebsiteStudentZoneItemsEditPostRouteTag::new(self.id),
                                "Edit",
                                StudentZoneItemEditModalKey::ID,
                                "btn btn-outline btn-sm",
                            ))
                        }
                    },
                }))
                (label("Title", field_text(FieldText { value: &self.title, classes: "" })))
                (label("Section", field_related(&self.section, &section_href)))
                (label("Is link", field_text(FieldText { value: yes_no(self.is_link), classes: "" })))
                (label("Link", field_text(FieldText { value: empty_dash(&self.link), classes: "" })))
                (label("File", field_vnode(self.file_id, &self.file_name)))
            }))
        })
    }
}

impl lariv_rs::template::RenderAppPane for StudentZoneItemDetailPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        scaffold_pane(
            website_menu(),
            student_zone_item_detail_crumbs(&self.title),
            self.pane_body(),
        )
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        scaffold_main(
            student_zone_item_detail_crumbs(&self.title),
            self.pane_body(),
        )
    }
}

impl RenderTemplate for StudentZoneItemDetailPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold(
            &format!("{} — {APP_TITLE}", self.title),
            chrome,
            website_menu(),
            student_zone_item_detail_crumbs(&self.title),
            self.pane_body(),
        )
    }
}

#[derive(Generic)]
pub struct StudentZoneItemFormPage {
    pub id: i64,
    pub title: String,
    pub is_link: bool,
    pub link: String,
    pub file_id: i64,
    pub file_display: String,
    pub student_zone_section_id: i64,
    pub section_display: String,
    pub error: String,
    pub form_name: String,
    pub refresh_table: String,
    pub target_input: String,
}

impl RenderTemplate for StudentZoneItemFormPage {
    fn render(&self, _chrome: &ShellChrome) -> Markup {
        let is_create = self.id == 0;
        let form_attrs = if is_create {
            form_hx_post_url::<StudentZoneItemCreateModalKey>(&modal_create_post_query(
                WebsiteStudentZoneItemsCreatePostRouteTag,
                &self.form_name,
                &self.refresh_table,
                &self.target_input,
            ))
        } else {
            form_hx_post_url::<StudentZoneItemEditModalKey>(&modal_edit_post_url(
                WebsiteStudentZoneItemsEditPostRouteTag::new(self.id),
                &self.form_name,
            ))
        };
        let file_id = if self.file_id > 0 {
            self.file_id.to_string()
        } else {
            String::new()
        };
        let section_id = if self.student_zone_section_id > 0 {
            self.student_zone_section_id.to_string()
        } else {
            String::new()
        };
        let body = form(FormOpts {
            attrs: form_attrs,
            title: if is_create {
                "Create item"
            } else {
                "Edit item"
            },
            subtitle: if is_create {
                "Add a student-zone item"
            } else {
                "Update this item"
            },
            form_error: Some(self.error.as_str()).filter(|e| !e.is_empty()),
            inputs: StudentZoneItemForm::render_inputs(
                &FormCtx::form::<StudentZoneItemForm>()
                    .value(StudentZoneItemFormField::Title, &self.title)
                    .checked(StudentZoneItemFormField::IsLink, self.is_link)
                    .value(StudentZoneItemFormField::Link, &self.link)
                    .value(StudentZoneItemFormField::FileId, &file_id)
                    .display(StudentZoneItemFormField::FileId, &self.file_display)
                    .value(StudentZoneItemFormField::StudentZoneSectionId, &section_id)
                    .display(
                        StudentZoneItemFormField::StudentZoneSectionId,
                        &self.section_display,
                    ),
            ),
            actions: html! {
                (button_submit(ButtonSubmit { label: "Save", ..Default::default() }))
                @if !is_create {
                    (button_modal_form_route(
                        WebsiteStudentZoneItemsDeleteGetRouteTag::new(self.id),
                        WebsiteStudentZoneItemsDeletePostRouteTag::new(self.id),
                        "Delete",
                        StudentZoneItemDeleteModalKey::ID,
                        "btn-error",
                    ))
                }
            },
            ..Default::default()
        });
        if is_create {
            modal_keyed::<StudentZoneItemCreateModalKey>("", body)
        } else {
            modal_keyed::<StudentZoneItemEditModalKey>("", body)
        }
    }
}

#[derive(Generic)]
pub struct ContactPageSettingsFormPage {
    pub id: i64,
    pub file_id: i64,
    pub file_display: String,
    pub error: String,
}

impl ContactPageSettingsFormPage {
    fn pane_body(&self) -> Markup {
        let file_id = if self.file_id > 0 {
            self.file_id.to_string()
        } else {
            String::new()
        };
        form(FormOpts {
            attrs: form_hx_post_main(WebsiteContactPageSettingsEditPostRouteTag::new(self.id)),
            title: "Contact page settings",
            subtitle: "Essential committees list shown on Contact Us",
            form_error: Some(self.error.as_str()).filter(|e| !e.is_empty()),
            inputs: ContactPageSettingsForm::render_inputs(
                &FormCtx::form::<ContactPageSettingsForm>()
                    .value(
                        ContactPageSettingsFormField::EssentialCommitteesListFileId,
                        &file_id,
                    )
                    .display(
                        ContactPageSettingsFormField::EssentialCommitteesListFileId,
                        &self.file_display,
                    ),
            ),
            actions: html! {
                (button_submit(ButtonSubmit { label: "Save", ..Default::default() }))
            },
            ..Default::default()
        })
    }
}

impl lariv_rs::template::RenderAppPane for ContactPageSettingsFormPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        scaffold_pane(website_menu(), contact_page_crumbs(), self.pane_body())
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        scaffold_main(contact_page_crumbs(), self.pane_body())
    }
}

impl RenderTemplate for ContactPageSettingsFormPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold(
            &format!("Contact page — {APP_TITLE}"),
            chrome,
            website_menu(),
            contact_page_crumbs(),
            self.pane_body(),
        )
    }
}

#[derive(Generic)]
pub struct ConfirmDeletePage {
    pub modal_uid: String,
    pub message: String,
    pub form_name: String,
    pub id: i64,
    pub post_url: String,
    pub error: String,
}

impl RenderTemplate for ConfirmDeletePage {
    fn render(&self, _chrome: &ShellChrome) -> Markup {
        let target = format!("#{}", self.modal_uid);
        modal(lariv_rs::components::Modal {
            uid: &self.modal_uid,
            children: lariv_rs::components::delete_confirmation(DeleteConfirmation {
                title: "Confirm Deletion",
                message: &self.message,
                attrs: form_hx_post_selector(&self.post_url, &target),
                form_error: Some(self.error.as_str()).filter(|e| !e.is_empty()),
                ..Default::default()
            }),
            ..Default::default()
        })
    }
}

