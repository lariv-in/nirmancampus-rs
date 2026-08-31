use frunk::Generic;
use maud::{Markup, html};

use lariv_rs::{
    components::{
        ButtonClear, ButtonSubmit, DeleteConfirmation, DetailHeader, FieldText, FormOpts,
        ObjectList, ShellChrome, SlotCapability, SlotRegistrar, SwapKey, TableButtonFilter,
        TableColumnHeader, TableRow, button_clear, button_modal_form_route, button_submit,
        column_sort_url, container_column, container_row, data_table_list_grid_with_subtitle,
        data_table_list_refresh, detail, detail_header, field_text, form, form_hx_get_picker_route,
        form_hx_get_route, form_hx_post_selector, form_hx_post_url, hx_nav_app_layout, label, modal,
        modal_keyed, row_attr_select, sort_indicator, table_button_filter, table_create_button,
    },
    html_form::{FormCtx, HtmlForm},
    http::ProvideRequestCaps,
    picker::RenderPickerSelect,
    template::{RenderTemplate, TemplateCapability, TemplateOf, TemplateRegistrar},
    web::{modal_create_post_query, modal_edit_post_url},
};

use crate::{session_detail_crumbs, session_detail_menu, sessions_crumbs, sessions_menu};
use super::forms::{SessionFilterForm, SessionFilterFormField, SessionForm, SessionFormField};
use super::keys::{
    SessionCreateModalKey, SessionDeleteModalKey, SessionEditModalKey, SessionSelectModalKey,
    SessionSelectTableKey, SessionTableKey,
};
use super::routes::{
    SessionsCreatePostRouteTag, SessionsDeleteGetRouteTag, SessionsDeletePostRouteTag,
    SessionsDetailRouteTag, SessionsEditGetRouteTag, SessionsEditPostRouteTag, SessionsListRouteTag,
    SessionsSelectRouteTag,
};
use nirmancampus_common::ui::{
    app_scaffold, render_pagination, render_picker_pagination, scaffold_main, scaffold_pane, yes_no,
    APP_TITLE,
};

lariv_rs::define_register_items! {
    plugin: NirmancampusSessionsTag;
    capability: TemplateCapability;
    trait: TemplateRegistrar;
    method: register_templates;
    wrapper: TemplateOf;
    bounds: [Clone, ProvideRequestCaps, Send, Sync];
    hook: Hook;
    items: [
        SessionListIdx: SessionListPageTag => SessionListPage,
        SessionDetailIdx: SessionDetailPageTag => SessionDetailPage,
        SessionFormIdx: SessionFormPageTag => SessionFormPage,
        SessionSelectIdx: SessionSelectPageTag => SessionSelectPage,
        ConfirmDeleteIdx: SessionConfirmDeletePageTag => ConfirmDeletePage,
    ]
}

lariv_rs::define_register_items! {
    plugin: NirmancampusSessionsTag;
    capability: SlotCapability;
    trait: SlotRegistrar;
    method: register_slots;
    bounds: [];
    items: [];
    hook: SlotsHook;
}

#[derive(Clone)]
pub struct SessionRow {
    pub id: i64,
    pub name: String,
    pub code: String,
    pub start: String,
    pub end: String,
    pub is_active: bool,
}

#[derive(Generic)]
pub struct SessionListPage {
    pub sessions: ObjectList<SessionRow>,
    pub filter_name: String,
    pub filter_code: String,
    pub path_and_query: String,
    pub sort: String,
    pub is_admin: bool,
}

impl SessionListPage {
    pub fn render_table(&self) -> Markup {
        let name_sort = column_sort_url(&self.path_and_query, "Name", &self.sort);
        let name_label = format!("Name{}", sort_indicator(&self.sort, "Name"));
        let start_sort = column_sort_url(&self.path_and_query, "Start", &self.sort);
        let start_label = format!("Start{}", sort_indicator(&self.sort, "Start"));
        let headers = [
            TableColumnHeader { key: "Name", label: &name_label, sort_url: Some(&name_sort), push_url: true },
            TableColumnHeader { key: "Code", label: "Code", sort_url: None, push_url: false },
            TableColumnHeader { key: "Start", label: &start_label, sort_url: Some(&start_sort), push_url: true },
            TableColumnHeader { key: "End", label: "End", sort_url: None, push_url: false },
            TableColumnHeader { key: "Active", label: "Active", sort_url: None, push_url: false },
        ];
        let rows: Vec<TableRow> = self
            .sessions
            .items
            .iter()
            .map(|s| TableRow {
                attrs: hx_nav_app_layout(SessionsDetailRouteTag::new(s.id)),
                cells: vec![
                    field_text(FieldText { value: &s.name, classes: "" }),
                    field_text(FieldText { value: &s.code, classes: "" }),
                    field_text(FieldText { value: &s.start, classes: "" }),
                    field_text(FieldText { value: &s.end, classes: "" }),
                    field_text(FieldText { value: yes_no(s.is_active), classes: "" }),
                ],
            })
            .collect();
        let actions = html! {
            (table_button_filter(TableButtonFilter {
                panel: form(FormOpts {
                    attrs: form_hx_get_route::<SessionTableKey, SessionsListRouteTag>(SessionsListRouteTag),
                    inputs: SessionFilterForm::render_inputs(
                        &FormCtx::form::<SessionFilterForm>()
                            .value(SessionFilterFormField::Name, &self.filter_name)
                            .value(SessionFilterFormField::Code, &self.filter_code),
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
                (table_create_button::<SessionTableKey, SessionCreateModalKey>(
                    Some("plus"),
                    "btn-square btn-outline btn-sm",
                ))
            }
        };
        data_table_list_grid_with_subtitle::<SessionTableKey>(
            "Sessions",
            "Admission sessions",
            actions,
            &headers,
            &rows,
            render_pagination::<SessionTableKey>(
                &self.path_and_query,
                self.sessions.number,
                self.sessions.num_pages,
                true,
            ),
        )
    }
}

impl lariv_rs::template::RenderAppPane for SessionListPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        scaffold_pane(sessions_menu(), sessions_crumbs("All Sessions"), self.render_table())
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        scaffold_main(sessions_crumbs("All Sessions"), self.render_table())
    }
}

impl RenderTemplate for SessionListPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold(
            &format!("Sessions — {APP_TITLE}"),
            chrome,
            sessions_menu(),
            sessions_crumbs("All Sessions"),
            self.render_table(),
        )
    }
}

#[derive(Generic)]
pub struct SessionDetailPage {
    pub id: i64,
    pub name: String,
    pub code: String,
    pub start: String,
    pub end: String,
    pub is_active: bool,
    pub is_admin: bool,
}

impl SessionDetailPage {
    fn pane_body(&self) -> Markup {
        detail(html! {
            (container_column("", html! {
                (detail_header(DetailHeader {
                    title: &self.name,
                    actions: html! {
                        @if self.is_admin {
                            (button_modal_form_route(
                                SessionsEditGetRouteTag::new(self.id),
                                SessionsEditPostRouteTag::new(self.id),
                                "Edit",
                                SessionEditModalKey::ID,
                                "btn btn-outline btn-sm",
                            ))
                        }
                    },
                }))
                (label("Code", field_text(FieldText { value: &self.code, classes: "" })))
                (label("Start", field_text(FieldText { value: &self.start, classes: "" })))
                (label("End", field_text(FieldText { value: &self.end, classes: "" })))
                (label("Active", field_text(FieldText { value: yes_no(self.is_active), classes: "" })))
            }))
        })
    }
}

impl lariv_rs::template::RenderAppPane for SessionDetailPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        scaffold_pane(
            session_detail_menu(self.id, &self.name),
            session_detail_crumbs(self.id, &self.name, None),
            self.pane_body(),
        )
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        scaffold_main(session_detail_crumbs(self.id, &self.name, None), self.pane_body())
    }
}

impl RenderTemplate for SessionDetailPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold(
            &format!("{} — {APP_TITLE}", self.name),
            chrome,
            session_detail_menu(self.id, &self.name),
            session_detail_crumbs(self.id, &self.name, None),
            self.pane_body(),
        )
    }
}

#[derive(Generic)]
pub struct SessionFormPage {
    pub id: i64,
    pub name: String,
    pub code: String,
    pub start: String,
    pub end: String,
    pub is_active: bool,
    pub error: String,
    pub form_name: String,
    pub refresh_table: String,
    pub target_input: String,
}

impl RenderTemplate for SessionFormPage {
    fn render(&self, _chrome: &ShellChrome) -> Markup {
        let is_create = self.id == 0;
        let form_attrs = if is_create {
            form_hx_post_url::<SessionCreateModalKey>(&modal_create_post_query(
                SessionsCreatePostRouteTag,
                &self.form_name,
                &self.refresh_table,
                &self.target_input,
            ))
        } else {
            form_hx_post_url::<SessionEditModalKey>(&modal_edit_post_url(
                SessionsEditPostRouteTag::new(self.id),
                &self.form_name,
            ))
        };
        let body = form(FormOpts {
            attrs: form_attrs,
            title: if is_create { "Create Session" } else { "Edit Session" },
            subtitle: if is_create { "Create a new admission session" } else { "Update session details" },
            form_error: Some(self.error.as_str()).filter(|e| !e.is_empty()),
            inputs: SessionForm::render_inputs(
                &FormCtx::form::<SessionForm>()
                    .value(SessionFormField::Name, &self.name)
                    .value(SessionFormField::Code, &self.code)
                    .value(SessionFormField::Start, &self.start)
                    .value(SessionFormField::End, &self.end)
                    .checked(SessionFormField::IsActive, self.is_active),
            ),
            actions: html! {
                (button_submit(ButtonSubmit { label: "Save", ..Default::default() }))
                @if !is_create {
                    (button_modal_form_route(
                        SessionsDeleteGetRouteTag::new(self.id),
                        SessionsDeletePostRouteTag::new(self.id),
                        "Delete",
                        SessionDeleteModalKey::ID,
                        "btn-error",
                    ))
                }
            },
            ..Default::default()
        });
        if is_create {
            modal_keyed::<SessionCreateModalKey>("", body)
        } else {
            modal_keyed::<SessionEditModalKey>("", body)
        }
    }
}

#[derive(Generic)]
pub struct SessionSelectPage {
    pub sessions: ObjectList<SessionRow>,
    pub filter_name: String,
    pub filter_code: String,
    pub path_and_query: String,
    pub target_input: String,
}

impl RenderPickerSelect<SessionSelectTableKey, SessionSelectModalKey> for SessionSelectPage {
    fn render_table(&self) -> Markup {
        let headers = [
            TableColumnHeader { key: "Name", label: "Name", sort_url: None, push_url: false },
            TableColumnHeader { key: "Code", label: "Code", sort_url: None, push_url: false },
        ];
        let target = if self.target_input.is_empty() { "SessionID" } else { self.target_input.as_str() };
        let rows: Vec<TableRow> = self
            .sessions
            .items
            .iter()
            .map(|s| TableRow {
                attrs: row_attr_select(target, &s.id.to_string(), &s.name),
                cells: vec![
                    field_text(FieldText { value: &s.name, classes: "" }),
                    field_text(FieldText { value: &s.code, classes: "" }),
                ],
            })
            .collect();
        data_table_list_refresh::<SessionSelectTableKey>(
            "Select Session",
            table_button_filter(TableButtonFilter {
                panel: form(FormOpts {
                    attrs: form_hx_get_picker_route::<
                        SessionSelectTableKey,
                        SessionSelectModalKey,
                        SessionsSelectRouteTag,
                    >(SessionsSelectRouteTag)
                        .set("hx-push-url", "false"),
                    inputs: html! {
                        (SessionFilterForm::render_inputs(
                            &FormCtx::form::<SessionFilterForm>()
                                .value(SessionFilterFormField::Name, &self.filter_name)
                                .value(SessionFilterFormField::Code, &self.filter_code),
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
            render_picker_pagination::<SessionSelectModalKey>(
                &self.path_and_query,
                self.sessions.number,
                self.sessions.num_pages,
            ),
            &self.path_and_query,
        )
    }
}

impl RenderTemplate for SessionSelectPage {
    fn render(&self, _chrome: &ShellChrome) -> Markup {
        self.render_modal().into_inner()
    }
}

#[derive(Generic)]
pub struct ConfirmDeletePage {
    pub modal_uid: String,
    pub message: String,
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
                attrs: form_hx_post_selector(&SessionsDeletePostRouteTag::new(self.id).url(), &target),
                form_error: Some(self.error.as_str()).filter(|e| !e.is_empty()),
                ..Default::default()
            }),
            ..Default::default()
        })
    }
}
