use frunk::Generic;
use maud::{Markup, html};

use lariv_rs::{
    components::{
        ButtonClear, ButtonSubmit, DeleteConfirmation, DetailHeader, FieldText, FormOpts,
        ManyToManyItem, ObjectList, ShellChrome, SlotCapability, SlotRegistrar, SwapKey,
        TableButtonFilter, TableColumnHeader, TableRow, button_clear, button_modal_form_route,
        button_submit, container_column, container_row, data_table_list_grid_with_subtitle,
        data_table_list_refresh, detail, detail_header, field_text, form, form_hx_get_picker_route,
        form_hx_get_route, form_hx_post_selector, form_hx_post_url, hx_nav_app_layout, label, modal,
        modal_keyed, row_attr_select, table_button_filter, table_create_button,
    },
    html_form::{FormCtx, HtmlForm},
    http::ProvideRequestCaps,
    picker::RenderPickerSelect,
    template::{RenderTemplate, TemplateCapability, TemplateOf, TemplateRegistrar},
    web::{modal_create_post_query, modal_edit_post_url},
};

use crate::{
    application_detail_crumbs, application_detail_menu, applications_crumbs, applications_menu,
};
use super::forms::{
    ApplicationFilterForm, ApplicationFilterFormField, ApplicationForm, ApplicationFormField,
};
use super::keys::{
    ApplicationCreateModalKey, ApplicationDeleteModalKey, ApplicationEditModalKey,
    ApplicationSelectModalKey, ApplicationSelectTableKey, ApplicationTableKey,
};
use super::routes::{
    StudentApplicationsCreatePostRouteTag, StudentApplicationsDeleteGetRouteTag,
    StudentApplicationsDeletePostRouteTag, StudentApplicationsDetailRouteTag,
    StudentApplicationsEditGetRouteTag, StudentApplicationsEditPostRouteTag,
    StudentApplicationsListRouteTag, StudentApplicationsSelectRouteTag,
};
use nirmancampus_common::{
    category_choice_pairs,
    ui::{
        app_scaffold, empty_dash, field_related, field_vnode, field_vnode_many, render_pagination,
        render_picker_pagination, scaffold_main, scaffold_pane, APP_TITLE,
    },
};

lariv_rs::define_register_items! {
    plugin: NirmancampusStudentApplicationsTag;
    capability: TemplateCapability;
    trait: TemplateRegistrar;
    method: register_templates;
    wrapper: TemplateOf;
    bounds: [Clone, ProvideRequestCaps, Send, Sync];
    hook: Hook;
    items: [
        ApplicationListIdx: ApplicationListPageTag => ApplicationListPage,
        ApplicationDetailIdx: ApplicationDetailPageTag => ApplicationDetailPage,
        ApplicationFormIdx: ApplicationFormPageTag => ApplicationFormPage,
        ApplicationSelectIdx: ApplicationSelectPageTag => ApplicationSelectPage,
        ConfirmDeleteIdx: ApplicationConfirmDeletePageTag => ConfirmDeletePage,
    ]
}

lariv_rs::define_register_items! {
    plugin: NirmancampusStudentApplicationsTag;
    capability: SlotCapability;
    trait: SlotRegistrar;
    method: register_slots;
    bounds: [];
    items: [];
    hook: SlotsHook;
}

#[derive(Clone)]
pub struct ApplicationRow {
    pub id: i64,
    pub email: String,
    pub program_display: String,
    pub student_name: String,
    pub mobile: String,
}

#[derive(Generic)]
pub struct ApplicationListPage {
    pub applications: ObjectList<ApplicationRow>,
    pub filter_email: String,
    pub filter_student_name: String,
    pub filter_mobile: String,
    pub path_and_query: String,
    pub can_create: bool,
}

impl ApplicationListPage {
    pub fn render_table(&self) -> Markup {
        let headers = [
            TableColumnHeader { key: "Email", label: "Email", sort_url: None, push_url: false },
            TableColumnHeader { key: "Program", label: "Program", sort_url: None, push_url: false },
            TableColumnHeader { key: "Student", label: "Student", sort_url: None, push_url: false },
            TableColumnHeader { key: "Mobile", label: "Mobile", sort_url: None, push_url: false },
        ];
        let rows: Vec<TableRow> = self
            .applications
            .items
            .iter()
            .map(|a| TableRow {
                attrs: hx_nav_app_layout(StudentApplicationsDetailRouteTag::new(a.id)),
                cells: vec![
                    field_text(FieldText { value: empty_dash(&a.email), classes: "" }),
                    field_text(FieldText { value: empty_dash(&a.program_display), classes: "" }),
                    field_text(FieldText { value: &a.student_name, classes: "" }),
                    field_text(FieldText { value: empty_dash(&a.mobile), classes: "" }),
                ],
            })
            .collect();
        let actions = html! {
            (table_button_filter(TableButtonFilter {
                panel: form(FormOpts {
                    attrs: form_hx_get_route::<ApplicationTableKey, StudentApplicationsListRouteTag>(StudentApplicationsListRouteTag),
                    inputs: ApplicationFilterForm::render_inputs(
                        &FormCtx::form::<ApplicationFilterForm>()
                            .value(ApplicationFilterFormField::Email, &self.filter_email)
                            .value(ApplicationFilterFormField::StudentName, &self.filter_student_name)
                            .value(ApplicationFilterFormField::Mobile, &self.filter_mobile),
                    ),
                    actions: html! {
                        (container_row("flex gap-2", html! {
                            (button_submit(ButtonSubmit { label: "Apply filters", ..Default::default() }))
                            (button_clear(ButtonClear { label: "Clear", ..Default::default() }))
                        }))
                    },
                    ..Default::default()
                }),
                ..Default::default()
            }))
            @if self.can_create {
                (table_create_button::<ApplicationTableKey, ApplicationCreateModalKey>(
                    Some("plus"),
                    "btn-square btn-outline btn-sm",
                ))
            }
        };
        data_table_list_grid_with_subtitle::<ApplicationTableKey>(
            "Applications",
            "Student applications",
            actions,
            &headers,
            &rows,
            render_pagination::<ApplicationTableKey>(
                &self.path_and_query,
                self.applications.number,
                self.applications.num_pages,
                true,
            ),
        )
    }
}

impl lariv_rs::template::RenderAppPane for ApplicationListPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        scaffold_pane(
            applications_menu(),
            applications_crumbs("All Applications"),
            self.render_table(),
        )
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        scaffold_main(applications_crumbs("All Applications"), self.render_table())
    }
}

impl RenderTemplate for ApplicationListPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold(
            &format!("Applications — {APP_TITLE}"),
            chrome,
            applications_menu(),
            applications_crumbs("All Applications"),
            self.render_table(),
        )
    }
}

#[derive(Generic)]
pub struct ApplicationDetailPage {
    pub id: i64,
    pub program_id: i64,
    pub program_display: String,
    pub student_name: String,
    pub email: String,
    pub dob: String,
    pub mother_name: String,
    pub father_name: String,
    pub category: String,
    pub mobile: String,
    pub address: String,
    pub photo_id: i64,
    pub photo_name: String,
    pub documents: Vec<ManyToManyItem>,
    pub is_admin: bool,
}

impl ApplicationDetailPage {
    fn pane_body(&self) -> Markup {
        detail(html! {
            (container_column("", html! {
                (detail_header(DetailHeader {
                    title: &self.student_name,
                    actions: html! {
                        @if self.is_admin {
                            (button_modal_form_route(
                                StudentApplicationsEditGetRouteTag::new(self.id),
                                StudentApplicationsEditPostRouteTag::new(self.id),
                                "Edit",
                                ApplicationEditModalKey::ID,
                                "btn btn-outline btn-sm",
                            ))
                        }
                    },
                }))
                (label("Program", {
                    let href = if self.program_id > 0 {
                        format!("/programs/{}/", self.program_id)
                    } else {
                        String::new()
                    };
                    field_related(&self.program_display, &href)
                }))
                (label("Email", field_text(FieldText { value: empty_dash(&self.email), classes: "" })))
                (label("Date of birth", field_text(FieldText { value: empty_dash(&self.dob), classes: "" })))
                (label("Mother name", field_text(FieldText { value: empty_dash(&self.mother_name), classes: "" })))
                (label("Father name", field_text(FieldText { value: empty_dash(&self.father_name), classes: "" })))
                (label("Category", field_text(FieldText { value: empty_dash(&self.category), classes: "" })))
                (label("Mobile", field_text(FieldText { value: empty_dash(&self.mobile), classes: "" })))
                (label("Address", field_text(FieldText { value: empty_dash(&self.address), classes: "" })))
                (label("Photo", field_vnode(self.photo_id, &self.photo_name)))
                (label("Documents", {
                    let docs: Vec<(i64, &str)> = self
                        .documents
                        .iter()
                        .filter_map(|item| {
                            item.key.parse::<i64>().ok().map(|id| (id, item.value.as_str()))
                        })
                        .collect();
                    field_vnode_many(&docs)
                }))
            }))
        })
    }
}

impl lariv_rs::template::RenderAppPane for ApplicationDetailPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        scaffold_pane(
            application_detail_menu(self.id, &self.student_name),
            application_detail_crumbs(self.id, &self.student_name, None),
            self.pane_body(),
        )
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        scaffold_main(
            application_detail_crumbs(self.id, &self.student_name, None),
            self.pane_body(),
        )
    }
}

impl RenderTemplate for ApplicationDetailPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold(
            &format!("{} — {APP_TITLE}", self.student_name),
            chrome,
            application_detail_menu(self.id, &self.student_name),
            application_detail_crumbs(self.id, &self.student_name, None),
            self.pane_body(),
        )
    }
}

#[derive(Generic)]
pub struct ApplicationFormPage {
    pub id: i64,
    pub program_id: i64,
    pub program_display: String,
    pub student_name: String,
    pub dob: String,
    pub mother_name: String,
    pub father_name: String,
    pub category: String,
    pub mobile: String,
    pub email: String,
    pub address: String,
    pub photo_id: i64,
    pub photo_display: String,
    pub documents: Vec<ManyToManyItem>,
    pub error: String,
    pub form_name: String,
    pub refresh_table: String,
    pub target_input: String,
    pub can_delete: bool,
}

impl RenderTemplate for ApplicationFormPage {
    fn render(&self, _chrome: &ShellChrome) -> Markup {
        let is_create = self.id == 0;
        let form_attrs = if is_create {
            form_hx_post_url::<ApplicationCreateModalKey>(&modal_create_post_query(
                StudentApplicationsCreatePostRouteTag,
                &self.form_name,
                &self.refresh_table,
                &self.target_input,
            ))
        } else {
            form_hx_post_url::<ApplicationEditModalKey>(&modal_edit_post_url(
                StudentApplicationsEditPostRouteTag::new(self.id),
                &self.form_name,
            ))
        };
        let choices = category_choice_pairs();
        let program_id = if self.program_id > 0 { self.program_id.to_string() } else { String::new() };
        let photo_id = if self.photo_id > 0 { self.photo_id.to_string() } else { String::new() };
        let body = form(FormOpts {
            attrs: form_attrs,
            title: if is_create { "Create application" } else { "Edit application" },
            subtitle: if is_create {
                "Record a new student application"
            } else {
                "Update application details"
            },
            form_error: Some(self.error.as_str()).filter(|e| !e.is_empty()),
            inputs: ApplicationForm::render_inputs(
                &FormCtx::form::<ApplicationForm>()
                    .value(ApplicationFormField::ProgramId, &program_id)
                    .display(ApplicationFormField::ProgramId, &self.program_display)
                    .value(ApplicationFormField::StudentName, &self.student_name)
                    .value(ApplicationFormField::Dob, &self.dob)
                    .value(ApplicationFormField::MotherName, &self.mother_name)
                    .value(ApplicationFormField::FatherName, &self.father_name)
                    .value(ApplicationFormField::Category, &self.category)
                    .choices(ApplicationFormField::Category, &choices)
                    .value(ApplicationFormField::Mobile, &self.mobile)
                    .value(ApplicationFormField::Email, &self.email)
                    .value(ApplicationFormField::Address, &self.address)
                    .value(ApplicationFormField::PhotoId, &photo_id)
                    .display(ApplicationFormField::PhotoId, &self.photo_display)
                    .m2m(ApplicationFormField::Documents, &self.documents),
            ),
            actions: html! {
                (button_submit(ButtonSubmit { label: "Save application", ..Default::default() }))
                @if self.can_delete {
                    (button_modal_form_route(
                        StudentApplicationsDeleteGetRouteTag::new(self.id),
                        StudentApplicationsDeletePostRouteTag::new(self.id),
                        "Delete",
                        ApplicationDeleteModalKey::ID,
                        "btn-error",
                    ))
                }
            },
            ..Default::default()
        });
        if is_create {
            modal_keyed::<ApplicationCreateModalKey>("", body)
        } else {
            modal_keyed::<ApplicationEditModalKey>("", body)
        }
    }
}

#[derive(Generic)]
pub struct ApplicationSelectPage {
    pub applications: ObjectList<ApplicationRow>,
    pub filter_student_name: String,
    pub path_and_query: String,
    pub target_input: String,
}

impl RenderPickerSelect<ApplicationSelectTableKey, ApplicationSelectModalKey> for ApplicationSelectPage {
    fn render_table(&self) -> Markup {
        let headers = [
            TableColumnHeader { key: "Student", label: "Student", sort_url: None, push_url: false },
            TableColumnHeader { key: "Program", label: "Program", sort_url: None, push_url: false },
        ];
        let target = if self.target_input.is_empty() { "StudentApplicationID" } else { self.target_input.as_str() };
        let rows: Vec<TableRow> = self
            .applications
            .items
            .iter()
            .map(|a| TableRow {
                attrs: row_attr_select(target, &a.id.to_string(), &a.student_name),
                cells: vec![
                    field_text(FieldText { value: &a.student_name, classes: "" }),
                    field_text(FieldText { value: empty_dash(&a.program_display), classes: "" }),
                ],
            })
            .collect();
        data_table_list_refresh::<ApplicationSelectTableKey>(
            "Select Application",
            table_button_filter(TableButtonFilter {
                panel: form(FormOpts {
                    attrs: form_hx_get_picker_route::<
                        ApplicationSelectTableKey,
                        ApplicationSelectModalKey,
                        StudentApplicationsSelectRouteTag,
                    >(StudentApplicationsSelectRouteTag)
                        .set("hx-push-url", "false"),
                    inputs: html! {
                        (ApplicationFilterForm::render_inputs(
                            &FormCtx::form::<ApplicationFilterForm>()
                                .value(ApplicationFilterFormField::StudentName, &self.filter_student_name),
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
            render_picker_pagination::<ApplicationSelectModalKey>(
                &self.path_and_query,
                self.applications.number,
                self.applications.num_pages,
            ),
            &self.path_and_query,
        )
    }
}

impl RenderTemplate for ApplicationSelectPage {
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
                attrs: form_hx_post_selector(&StudentApplicationsDeletePostRouteTag::new(self.id).url(), &target),
                form_error: Some(self.error.as_str()).filter(|e| !e.is_empty()),
                ..Default::default()
            }),
            ..Default::default()
        })
    }
}
