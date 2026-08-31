use frunk::Generic;
use maud::{Markup, html};

use lariv_rs::{
    components::{
        ButtonClear, ButtonSubmit, DeleteConfirmation, DetailHeader, FieldMarkdown, FieldText,
        FormOpts, ManyToManyItem, ObjectList, ShellChrome, SlotCapability, SlotRegistrar, SwapKey,
        TableButtonFilter, TableColumnHeader, TableRow, button_clear, button_modal_form_route,
        button_submit, column_sort_url, container_column, container_row,
        data_table_list_grid_with_subtitle, data_table_list_refresh, detail, detail_header,
        field_markdown, field_text, form, form_hx_get_picker_route, form_hx_get_route,
        form_hx_post_selector, form_hx_post_url, hx_nav_app_layout, label, modal, modal_keyed,
        row_attr_select, sort_indicator, table_button_filter, table_create_button,
    },
    html_form::{FormCtx, HtmlForm},
    http::ProvideRequestCaps,
    picker::RenderPickerSelect,
    template::{RenderTemplate, TemplateCapability, TemplateOf, TemplateRegistrar},
    web::{modal_create_post_query, modal_edit_post_url},
};

use crate::{
    announcement_detail_crumbs, announcement_detail_menu, announcements_crumbs, announcements_menu,
};
use super::forms::{
    AnnouncementFilterForm, AnnouncementFilterFormField, AnnouncementForm, AnnouncementFormField,
};
use super::keys::{
    AnnouncementCreateModalKey, AnnouncementDeleteModalKey, AnnouncementEditModalKey,
    AnnouncementSelectModalKey, AnnouncementSelectTableKey, AnnouncementTableKey,
};
use super::routes::{
    AnnouncementsCreatePostRouteTag, AnnouncementsDeleteGetRouteTag, AnnouncementsDeletePostRouteTag,
    AnnouncementsDetailRouteTag, AnnouncementsEditGetRouteTag, AnnouncementsEditPostRouteTag,
    AnnouncementsListRouteTag, AnnouncementsSelectRouteTag,
};
use nirmancampus_common::ui::{
    app_scaffold, empty_dash, field_vnode_many, render_pagination, render_picker_pagination,
    scaffold_main, scaffold_pane, APP_TITLE,
};

lariv_rs::define_register_items! {
    plugin: NirmancampusAnnouncementsTag;
    capability: TemplateCapability;
    trait: TemplateRegistrar;
    method: register_templates;
    wrapper: TemplateOf;
    bounds: [Clone, ProvideRequestCaps, Send, Sync];
    hook: Hook;
    items: [
        AnnouncementListIdx: AnnouncementListPageTag => AnnouncementListPage,
        AnnouncementDetailIdx: AnnouncementDetailPageTag => AnnouncementDetailPage,
        AnnouncementFormIdx: AnnouncementFormPageTag => AnnouncementFormPage,
        AnnouncementSelectIdx: AnnouncementSelectPageTag => AnnouncementSelectPage,
        ConfirmDeleteIdx: AnnouncementConfirmDeletePageTag => ConfirmDeletePage,
    ]
}

lariv_rs::define_register_items! {
    plugin: NirmancampusAnnouncementsTag;
    capability: SlotCapability;
    trait: SlotRegistrar;
    method: register_slots;
    bounds: [];
    items: [];
    hook: SlotsHook;
}

#[derive(Clone)]
pub struct AnnouncementRow {
    pub id: i64,
    pub title: String,
    pub url: String,
    pub release_at: String,
    pub expiry_at: String,
}

#[derive(Generic)]
pub struct AnnouncementListPage {
    pub announcements: ObjectList<AnnouncementRow>,
    pub filter_title: String,
    pub filter_description: String,
    pub path_and_query: String,
    pub sort: String,
    pub is_admin: bool,
}

impl AnnouncementListPage {
    pub fn render_table(&self) -> Markup {
        let title_sort = column_sort_url(&self.path_and_query, "Title", &self.sort);
        let title_label = format!("Title{}", sort_indicator(&self.sort, "Title"));
        let release_sort = column_sort_url(&self.path_and_query, "ReleaseAt", &self.sort);
        let release_label = format!("Release At{}", sort_indicator(&self.sort, "ReleaseAt"));
        let headers = [
            TableColumnHeader { key: "Title", label: &title_label, sort_url: Some(&title_sort), push_url: true },
            TableColumnHeader { key: "URL", label: "URL", sort_url: None, push_url: false },
            TableColumnHeader { key: "ReleaseAt", label: &release_label, sort_url: Some(&release_sort), push_url: true },
            TableColumnHeader { key: "ExpiryAt", label: "Expiry At", sort_url: None, push_url: false },
        ];
        let rows: Vec<TableRow> = self
            .announcements
            .items
            .iter()
            .map(|a| TableRow {
                attrs: hx_nav_app_layout(AnnouncementsDetailRouteTag::new(a.id)),
                cells: vec![
                    field_text(FieldText { value: &a.title, classes: "" }),
                    field_text(FieldText { value: empty_dash(&a.url), classes: "" }),
                    field_text(FieldText { value: empty_dash(&a.release_at), classes: "" }),
                    field_text(FieldText { value: empty_dash(&a.expiry_at), classes: "" }),
                ],
            })
            .collect();
        let actions = html! {
            (table_button_filter(TableButtonFilter {
                panel: form(FormOpts {
                    attrs: form_hx_get_route::<AnnouncementTableKey, AnnouncementsListRouteTag>(AnnouncementsListRouteTag),
                    inputs: AnnouncementFilterForm::render_inputs(
                        &FormCtx::form::<AnnouncementFilterForm>()
                            .value(AnnouncementFilterFormField::Title, &self.filter_title)
                            .value(AnnouncementFilterFormField::Description, &self.filter_description),
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
                (table_create_button::<AnnouncementTableKey, AnnouncementCreateModalKey>(
                    Some("plus"),
                    "btn-square btn-outline btn-sm",
                ))
            }
        };
        data_table_list_grid_with_subtitle::<AnnouncementTableKey>(
            "Announcements",
            "Campus announcements",
            actions,
            &headers,
            &rows,
            render_pagination::<AnnouncementTableKey>(
                &self.path_and_query,
                self.announcements.number,
                self.announcements.num_pages,
                true,
            ),
        )
    }
}

impl lariv_rs::template::RenderAppPane for AnnouncementListPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        scaffold_pane(
            announcements_menu(),
            announcements_crumbs("All Announcements"),
            self.render_table(),
        )
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        scaffold_main(announcements_crumbs("All Announcements"), self.render_table())
    }
}

impl RenderTemplate for AnnouncementListPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold(
            &format!("Announcements — {APP_TITLE}"),
            chrome,
            announcements_menu(),
            announcements_crumbs("All Announcements"),
            self.render_table(),
        )
    }
}

#[derive(Generic)]
pub struct AnnouncementDetailPage {
    pub id: i64,
    pub title: String,
    pub description: String,
    pub url: String,
    pub release_at: String,
    pub expiry_at: String,
    pub assets: Vec<ManyToManyItem>,
    pub is_admin: bool,
}

impl AnnouncementDetailPage {
    fn pane_body(&self) -> Markup {
        detail(html! {
            (container_column("", html! {
                (detail_header(DetailHeader {
                    title: &self.title,
                    actions: html! {
                        @if self.is_admin {
                            (button_modal_form_route(
                                AnnouncementsEditGetRouteTag::new(self.id),
                                AnnouncementsEditPostRouteTag::new(self.id),
                                "Edit",
                                AnnouncementEditModalKey::ID,
                                "btn btn-outline btn-sm",
                            ))
                        }
                    },
                }))
                (label("Description", field_markdown(FieldMarkdown { value: &self.description, classes: "" })))
                (label("URL", field_text(FieldText { value: empty_dash(&self.url), classes: "" })))
                (label("Release At", field_text(FieldText { value: empty_dash(&self.release_at), classes: "" })))
                (label("Expiry At", field_text(FieldText { value: empty_dash(&self.expiry_at), classes: "" })))
                (label("Assets", {
                    let assets: Vec<(i64, &str)> = self
                        .assets
                        .iter()
                        .filter_map(|item| {
                            item.key.parse::<i64>().ok().map(|id| (id, item.value.as_str()))
                        })
                        .collect();
                    field_vnode_many(&assets)
                }))
            }))
        })
    }
}

impl lariv_rs::template::RenderAppPane for AnnouncementDetailPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        scaffold_pane(
            announcement_detail_menu(self.id, &self.title),
            announcement_detail_crumbs(self.id, &self.title, None),
            self.pane_body(),
        )
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        scaffold_main(
            announcement_detail_crumbs(self.id, &self.title, None),
            self.pane_body(),
        )
    }
}

impl RenderTemplate for AnnouncementDetailPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold(
            &format!("{} — {APP_TITLE}", self.title),
            chrome,
            announcement_detail_menu(self.id, &self.title),
            announcement_detail_crumbs(self.id, &self.title, None),
            self.pane_body(),
        )
    }
}

#[derive(Generic)]
pub struct AnnouncementFormPage {
    pub id: i64,
    pub title: String,
    pub description: String,
    pub url: String,
    pub release_at: String,
    pub expiry_at: String,
    pub assets: Vec<ManyToManyItem>,
    pub error: String,
    pub form_name: String,
    pub refresh_table: String,
    pub target_input: String,
}

impl RenderTemplate for AnnouncementFormPage {
    fn render(&self, _chrome: &ShellChrome) -> Markup {
        let is_create = self.id == 0;
        let form_attrs = if is_create {
            form_hx_post_url::<AnnouncementCreateModalKey>(&modal_create_post_query(
                AnnouncementsCreatePostRouteTag,
                &self.form_name,
                &self.refresh_table,
                &self.target_input,
            ))
        } else {
            form_hx_post_url::<AnnouncementEditModalKey>(&modal_edit_post_url(
                AnnouncementsEditPostRouteTag::new(self.id),
                &self.form_name,
            ))
        };
        let body = form(FormOpts {
            attrs: form_attrs,
            title: if is_create { "Create Announcement" } else { "Edit Announcement" },
            subtitle: if is_create { "Create a new announcement" } else { "Update announcement details" },
            form_error: Some(self.error.as_str()).filter(|e| !e.is_empty()),
            inputs: AnnouncementForm::render_inputs(
                &FormCtx::form::<AnnouncementForm>()
                    .value(AnnouncementFormField::Title, &self.title)
                    .value(AnnouncementFormField::Description, &self.description)
                    .value(AnnouncementFormField::Url, &self.url)
                    .value(AnnouncementFormField::ReleaseAt, &self.release_at)
                    .value(AnnouncementFormField::ExpiryAt, &self.expiry_at)
                    .m2m(AnnouncementFormField::Assets, &self.assets),
            ),
            actions: html! {
                (button_submit(ButtonSubmit { label: "Save", ..Default::default() }))
                @if !is_create {
                    (button_modal_form_route(
                        AnnouncementsDeleteGetRouteTag::new(self.id),
                        AnnouncementsDeletePostRouteTag::new(self.id),
                        "Delete",
                        AnnouncementDeleteModalKey::ID,
                        "btn-error",
                    ))
                }
            },
            ..Default::default()
        });
        if is_create {
            modal_keyed::<AnnouncementCreateModalKey>("", body)
        } else {
            modal_keyed::<AnnouncementEditModalKey>("", body)
        }
    }
}

#[derive(Generic)]
pub struct AnnouncementSelectPage {
    pub announcements: ObjectList<AnnouncementRow>,
    pub filter_title: String,
    pub path_and_query: String,
    pub target_input: String,
}

impl RenderPickerSelect<AnnouncementSelectTableKey, AnnouncementSelectModalKey> for AnnouncementSelectPage {
    fn render_table(&self) -> Markup {
        let headers = [
            TableColumnHeader { key: "Title", label: "Title", sort_url: None, push_url: false },
            TableColumnHeader { key: "ReleaseAt", label: "Release At", sort_url: None, push_url: false },
        ];
        let target = if self.target_input.is_empty() { "AnnouncementID" } else { self.target_input.as_str() };
        let rows: Vec<TableRow> = self
            .announcements
            .items
            .iter()
            .map(|a| TableRow {
                attrs: row_attr_select(target, &a.id.to_string(), &a.title),
                cells: vec![
                    field_text(FieldText { value: &a.title, classes: "" }),
                    field_text(FieldText { value: empty_dash(&a.release_at), classes: "" }),
                ],
            })
            .collect();
        data_table_list_refresh::<AnnouncementSelectTableKey>(
            "Select Announcement",
            table_button_filter(TableButtonFilter {
                panel: form(FormOpts {
                    attrs: form_hx_get_picker_route::<
                        AnnouncementSelectTableKey,
                        AnnouncementSelectModalKey,
                        AnnouncementsSelectRouteTag,
                    >(AnnouncementsSelectRouteTag)
                        .set("hx-push-url", "false"),
                    inputs: html! {
                        (AnnouncementFilterForm::render_inputs(
                            &FormCtx::form::<AnnouncementFilterForm>()
                                .value(AnnouncementFilterFormField::Title, &self.filter_title),
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
            render_picker_pagination::<AnnouncementSelectModalKey>(
                &self.path_and_query,
                self.announcements.number,
                self.announcements.num_pages,
            ),
            &self.path_and_query,
        )
    }
}

impl RenderTemplate for AnnouncementSelectPage {
    fn render(&self, _chrome: &ShellChrome) -> Markup {
        self.render_modal().into_inner()
    }
}

#[derive(Generic)]
pub struct ConfirmDeletePage {
    pub modal_uid: String,
    pub message: String,
    pub form_name: String,
    pub id: i64,
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
                attrs: form_hx_post_selector(&AnnouncementsDeletePostRouteTag::new(self.id).url(), &target),
                form_error: Some(self.error.as_str()).filter(|e| !e.is_empty()),
                ..Default::default()
            }),
            ..Default::default()
        })
    }
}
