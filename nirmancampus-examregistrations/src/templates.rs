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
    http::{ProvideRequestCaps, RouteQueryBuilder},
    picker::RenderPickerSelect,
    template::{RenderTemplate, TemplateCapability, TemplateOf, TemplateRegistrar},
    web::{modal_create_post_query, modal_edit_post_url},
};

use crate::{exam_detail_crumbs, exam_detail_menu, exams_crumbs};
use nirmancampus_academicrecords::routes::AcademicRecordsDetailRouteTag;
use nirmancampus_common::{
    env::EXAM_REGISTRATIONS_SESSION_KEY,
    exam_status_choice_pairs, format_inr,
    ui::{
        app_scaffold, empty_dash, field_related, field_vnode_many, render_pagination,
        render_picker_pagination, scaffold_main, scaffold_pane, session_environment_selector,
        students_hub_menu, SessionOption, APP_TITLE,
    },
};
use nirmancampus_courses::routes::CoursesDetailRouteTag;
use super::forms::{ExamFilterForm, ExamFilterFormField, ExamForm, ExamFormField};
use super::keys::{
    ExamBulkModalKey, ExamCreateModalKey, ExamDeleteModalKey, ExamEditModalKey, ExamSelectModalKey,
    ExamSelectTableKey, ExamTableKey,
};
use super::routes::{
    ExamRegistrationsBulkPostRouteTag, ExamRegistrationsCreatePostRouteTag,
    ExamRegistrationsDeleteGetRouteTag, ExamRegistrationsDeletePostRouteTag,
    ExamRegistrationsDetailRouteTag, ExamRegistrationsEditGetRouteTag,
    ExamRegistrationsEditPostRouteTag, ExamRegistrationsListRouteTag,
    ExamRegistrationsSelectRouteTag,
};

lariv_rs::define_register_items! {
    plugin: NirmancampusExamRegistrationsTag;
    capability: TemplateCapability;
    trait: TemplateRegistrar;
    method: register_templates;
    wrapper: TemplateOf;
    bounds: [Clone, ProvideRequestCaps, Send, Sync];
    hook: Hook;
    items: [
        ExamListIdx: ExamListPageTag => ExamListPage,
        ExamDetailIdx: ExamDetailPageTag => ExamDetailPage,
        ExamFormIdx: ExamFormPageTag => ExamFormPage,
        ExamSelectIdx: ExamSelectPageTag => ExamSelectPage,
        ConfirmDeleteIdx: ExamConfirmDeletePageTag => ConfirmDeletePage,
        BulkFromRecordIdx: ExamBulkFromRecordPageTag => BulkFromRecordPage,
    ]
}

lariv_rs::define_register_items! {
    plugin: NirmancampusExamRegistrationsTag;
    capability: SlotCapability;
    trait: SlotRegistrar;
    method: register_slots;
    bounds: [];
    items: [];
    hook: SlotsHook;
}

#[derive(Clone)]
pub struct ExamRow {
    pub id: i64,
    pub exam_title: String,
    pub course_name: String,
    pub registration_status: String,
    pub academic_record_display: String,
}

#[derive(Generic)]
pub struct ExamListPage {
    pub exams: ObjectList<ExamRow>,
    pub filter_exam_title: String,
    pub filter_registration_status: String,
    pub filter_academic_record_id: i64,
    pub filter_academic_record_display: String,
    pub path_and_query: String,
    pub is_admin: bool,
    pub sessions: Vec<SessionOption>,
    pub selected_session_id: i64,
}

impl ExamListPage {
    pub fn render_table(&self) -> Markup {
        let headers = [
            TableColumnHeader { key: "Exam", label: "Exam", sort_url: None, push_url: false },
            TableColumnHeader { key: "Course", label: "Course", sort_url: None, push_url: false },
            TableColumnHeader {
                key: "AcademicRecord",
                label: "Academic record",
                sort_url: None,
                push_url: false,
            },
            TableColumnHeader { key: "Status", label: "Status", sort_url: None, push_url: false },
        ];
        let rows: Vec<TableRow> = self
            .exams
            .items
            .iter()
            .map(|e| TableRow {
                attrs: hx_nav_app_layout(ExamRegistrationsDetailRouteTag::new(e.id)),
                cells: vec![
                    field_text(FieldText { value: &e.exam_title, classes: "" }),
                    field_text(FieldText { value: empty_dash(&e.course_name), classes: "" }),
                    field_text(FieldText {
                        value: empty_dash(&e.academic_record_display),
                        classes: "",
                    }),
                    field_text(FieldText { value: &e.registration_status, classes: "" }),
                ],
            })
            .collect();
        let academic_record_id = if self.filter_academic_record_id > 0 {
            self.filter_academic_record_id.to_string()
        } else {
            String::new()
        };
        let status_choices = exam_status_choice_pairs();
        let actions = html! {
            (table_button_filter(TableButtonFilter {
                panel: form(FormOpts {
                    attrs: form_hx_get_route::<ExamTableKey, ExamRegistrationsListRouteTag>(
                        ExamRegistrationsListRouteTag,
                    ),
                    inputs: ExamFilterForm::render_inputs(
                        &FormCtx::form::<ExamFilterForm>()
                            .value(ExamFilterFormField::ExamTitle, &self.filter_exam_title)
                            .value(
                                ExamFilterFormField::RegistrationStatus,
                                &self.filter_registration_status,
                            )
                            .choices(ExamFilterFormField::RegistrationStatus, &status_choices)
                            .value(ExamFilterFormField::AcademicRecordId, &academic_record_id)
                            .display(
                                ExamFilterFormField::AcademicRecordId,
                                &self.filter_academic_record_display,
                            ),
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
                (table_create_button::<ExamTableKey, ExamCreateModalKey>(
                    Some("plus"),
                    "btn-square btn-outline btn-sm",
                ))
            }
        };
        html! {
            (session_environment_selector(
                EXAM_REGISTRATIONS_SESSION_KEY,
                &self.sessions,
                self.selected_session_id,
            ))
            (data_table_list_grid_with_subtitle::<ExamTableKey>(
                "Exam Registrations",
                "Course exam registrations",
                actions,
                &headers,
                &rows,
                render_pagination::<ExamTableKey>(
                    &self.path_and_query,
                    self.exams.number,
                    self.exams.num_pages,
                    true,
                ),
            ))
        }
    }
}

impl lariv_rs::template::RenderAppPane for ExamListPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        scaffold_pane(
            students_hub_menu(),
            exams_crumbs("All Registrations"),
            self.render_table(),
        )
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        scaffold_main(exams_crumbs("All Registrations"), self.render_table())
    }
}

impl RenderTemplate for ExamListPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold(
            &format!("Exam Registrations — {APP_TITLE}"),
            chrome,
            students_hub_menu(),
            exams_crumbs("All Registrations"),
            self.render_table(),
        )
    }
}

#[derive(Clone)]
pub struct AssetLink {
    pub id: i64,
    pub name: String,
}

#[derive(Generic)]
pub struct ExamDetailPage {
    pub id: i64,
    pub exam_title: String,
    pub registration_status: String,
    pub max_marks: i64,
    pub marks: i64,
    pub fee: String,
    pub course_id: i64,
    pub course_name: String,
    pub academic_record_id: i64,
    pub academic_record_display: String,
    pub assets: Vec<AssetLink>,
    pub is_admin: bool,
}

impl ExamDetailPage {
    fn pane_body(&self) -> Markup {
        let marks = format!("{} / {}", self.marks, self.max_marks);
        let course_href = if self.course_id > 0 {
            CoursesDetailRouteTag::new(self.course_id).url()
        } else {
            String::new()
        };
        let record_href = if self.academic_record_id > 0 {
            AcademicRecordsDetailRouteTag::new(self.academic_record_id).url()
        } else {
            String::new()
        };
        let assets: Vec<(i64, &str)> = self
            .assets
            .iter()
            .map(|a| (a.id, a.name.as_str()))
            .collect();
        detail(html! {
            (container_column("", html! {
                (detail_header(DetailHeader {
                    title: &self.exam_title,
                    actions: html! {
                        @if self.is_admin {
                            (button_modal_form_route(
                                ExamRegistrationsEditGetRouteTag::new(self.id),
                                ExamRegistrationsEditPostRouteTag::new(self.id),
                                "Edit",
                                ExamEditModalKey::ID,
                                "btn btn-outline btn-sm",
                            ))
                        }
                    },
                }))
                (label("Course", field_related(&self.course_name, &course_href)))
                (label("Registration status", field_text(FieldText { value: &self.registration_status, classes: "" })))
                (label("Fee", field_text(FieldText { value: &self.fee, classes: "" })))
                @if self.is_admin {
                    (label("Marks", field_text(FieldText { value: &marks, classes: "" })))
                }
                (label("Academic record", field_related(&self.academic_record_display, &record_href)))
                (label("Assets", field_vnode_many(&assets)))
            }))
        })
    }
}

impl lariv_rs::template::RenderAppPane for ExamDetailPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        scaffold_pane(
            exam_detail_menu(self.id, &self.exam_title, self.is_admin),
            exam_detail_crumbs(self.id, &self.exam_title, None),
            self.pane_body(),
        )
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        scaffold_main(
            exam_detail_crumbs(self.id, &self.exam_title, None),
            self.pane_body(),
        )
    }
}

impl RenderTemplate for ExamDetailPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold(
            &format!("{} — {APP_TITLE}", self.exam_title),
            chrome,
            exam_detail_menu(self.id, &self.exam_title, self.is_admin),
            exam_detail_crumbs(self.id, &self.exam_title, None),
            self.pane_body(),
        )
    }
}

#[derive(Generic)]
pub struct ExamFormPage {
    pub id: i64,
    pub exam_title: String,
    pub registration_status: String,
    pub max_marks: i64,
    pub marks: i64,
    pub fee: i64,
    pub course_id: i64,
    pub course_display: String,
    pub academic_record_id: i64,
    pub academic_record_display: String,
    pub assets: Vec<ManyToManyItem>,
    pub error: String,
    pub form_name: String,
    pub refresh_table: String,
    pub target_input: String,
}

impl RenderTemplate for ExamFormPage {
    fn render(&self, _chrome: &ShellChrome) -> Markup {
        let is_create = self.id == 0;
        let form_attrs = if is_create {
            form_hx_post_url::<ExamCreateModalKey>(&modal_create_post_query(
                ExamRegistrationsCreatePostRouteTag,
                &self.form_name,
                &self.refresh_table,
                &self.target_input,
            ))
        } else {
            form_hx_post_url::<ExamEditModalKey>(&modal_edit_post_url(
                ExamRegistrationsEditPostRouteTag::new(self.id),
                &self.form_name,
            ))
        };
        let choices = exam_status_choice_pairs();
        let course_id = if self.course_id > 0 {
            self.course_id.to_string()
        } else {
            String::new()
        };
        let academic_record_id = if self.academic_record_id > 0 {
            self.academic_record_id.to_string()
        } else {
            String::new()
        };
        let max_marks = self.max_marks.to_string();
        let marks = self.marks.to_string();
        let fee = self.fee.to_string();
        let body = form(FormOpts {
            attrs: form_attrs,
            title: if is_create {
                "Create exam registration"
            } else {
                "Edit exam registration"
            },
            subtitle: if is_create {
                "Register a student for an exam."
            } else {
                "Update exam registration details."
            },
            form_error: Some(self.error.as_str()).filter(|e| !e.is_empty()),
            inputs: ExamForm::render_inputs(
                &FormCtx::form::<ExamForm>()
                    .value(ExamFormField::ExamTitle, &self.exam_title)
                    .value(ExamFormField::RegistrationStatus, &self.registration_status)
                    .choices(ExamFormField::RegistrationStatus, &choices)
                    .value(ExamFormField::MaxMarks, &max_marks)
                    .value(ExamFormField::Marks, &marks)
                    .value(ExamFormField::Fee, &fee)
                    .value(ExamFormField::CourseId, &course_id)
                    .display(ExamFormField::CourseId, &self.course_display)
                    .value(ExamFormField::AcademicRecordId, &academic_record_id)
                    .display(ExamFormField::AcademicRecordId, &self.academic_record_display)
                    .m2m(ExamFormField::Assets, &self.assets),
            ),
            actions: html! {
                (button_submit(ButtonSubmit { label: "Save registration", ..Default::default() }))
                @if !is_create {
                    (button_modal_form_route(
                        ExamRegistrationsDeleteGetRouteTag::new(self.id),
                        ExamRegistrationsDeletePostRouteTag::new(self.id),
                        "Delete",
                        ExamDeleteModalKey::ID,
                        "btn-error",
                    ))
                }
            },
            ..Default::default()
        });
        if is_create {
            modal_keyed::<ExamCreateModalKey>("", body)
        } else {
            modal_keyed::<ExamEditModalKey>("", body)
        }
    }
}

#[derive(Generic)]
pub struct ExamSelectPage {
    pub exams: ObjectList<ExamRow>,
    pub filter_exam_title: String,
    pub path_and_query: String,
    pub target_input: String,
}

impl RenderPickerSelect<ExamSelectTableKey, ExamSelectModalKey> for ExamSelectPage {
    fn render_table(&self) -> Markup {
        let headers = [
            TableColumnHeader { key: "Exam", label: "Exam", sort_url: None, push_url: false },
            TableColumnHeader { key: "Course", label: "Course", sort_url: None, push_url: false },
        ];
        let target = if self.target_input.is_empty() {
            "ExamID"
        } else {
            self.target_input.as_str()
        };
        let rows: Vec<TableRow> = self
            .exams
            .items
            .iter()
            .map(|e| TableRow {
                attrs: row_attr_select(target, &e.id.to_string(), &e.exam_title),
                cells: vec![
                    field_text(FieldText { value: &e.exam_title, classes: "" }),
                    field_text(FieldText { value: empty_dash(&e.course_name), classes: "" }),
                ],
            })
            .collect();
        data_table_list_refresh::<ExamSelectTableKey>(
            "Select Exam Registration",
            table_button_filter(TableButtonFilter {
                panel: form(FormOpts {
                    attrs: form_hx_get_picker_route::<
                        ExamSelectTableKey,
                        ExamSelectModalKey,
                        ExamRegistrationsSelectRouteTag,
                    >(ExamRegistrationsSelectRouteTag)
                        .set("hx-push-url", "false"),
                    inputs: html! {
                        (ExamFilterForm::render_inputs(
                            &FormCtx::form::<ExamFilterForm>()
                                .value(ExamFilterFormField::ExamTitle, &self.filter_exam_title),
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
            render_picker_pagination::<ExamSelectModalKey>(
                &self.path_and_query,
                self.exams.number,
                self.exams.num_pages,
            ),
            &self.path_and_query,
        )
    }
}

impl RenderTemplate for ExamSelectPage {
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
                attrs: form_hx_post_selector(
                    &ExamRegistrationsDeletePostRouteTag::new(self.id).url(),
                    &target,
                ),
                form_error: Some(self.error.as_str()).filter(|e| !e.is_empty()),
                ..Default::default()
            }),
            ..Default::default()
        })
    }
}

#[derive(Clone)]
pub struct BulkCourseRow {
    pub id: i64,
    pub name: String,
    pub fee: i64,
    pub already: bool,
}

#[derive(Generic)]
pub struct BulkFromRecordPage {
    pub academic_record_id: i64,
    pub student_line: String,
    pub courses: Vec<BulkCourseRow>,
    pub error: String,
}

impl RenderTemplate for BulkFromRecordPage {
    fn render(&self, _chrome: &ShellChrome) -> Markup {
        let post = RouteQueryBuilder::new(ExamRegistrationsBulkPostRouteTag)
            .query("AcademicRecordID", self.academic_record_id)
            .build_with_query();
        let aid = self.academic_record_id.to_string();
        let body = form(FormOpts {
            attrs: form_hx_post_url::<ExamBulkModalKey>(&post),
            title: "Create exam registrations",
            subtitle: &self.student_line,
            form_error: Some(self.error.as_str()).filter(|e| !e.is_empty()),
            inputs: html! {
                input type="hidden" name="AcademicRecordID" value=(aid);
                @if self.courses.is_empty() {
                    p class="opacity-60" { "No courses on this academic record." }
                }
                @for c in &self.courses {
                    @let fee = format_inr(c.fee);
                    label class={
                        @if c.already { "flex items-center gap-2 opacity-60" }
                        @else { "flex items-center gap-2" }
                    } {
                        @if c.already {
                            input type="checkbox" disabled checked {}
                            span { (c.name) " — already registered" }
                        } @else {
                            input type="checkbox" name="CourseIDs" value=(c.id) {}
                            span { (c.name) " — " (fee) }
                        }
                    }
                }
            },
            actions: html! {
                (button_submit(ButtonSubmit {
                    label: "Create registrations",
                    ..Default::default()
                }))
            },
            ..Default::default()
        });
        modal_keyed::<ExamBulkModalKey>("", body)
    }
}
