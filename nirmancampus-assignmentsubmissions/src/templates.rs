use frunk::Generic;
use maud::{Markup, html};

use lariv_rs::{
    components::{
        ButtonClear, ButtonSubmit, DeleteConfirmation, DetailHeader, FieldText, FormOpts,
        InputNumber, ManyToManyItem, ObjectList, ShellChrome, SlotCapability, SlotRegistrar,
        SwapKey, TableButtonFilter, TableColumnHeader, TableRow, button_clear,
        button_modal_form_route, button_submit, container_column, container_row,
        data_table_list_grid_with_subtitle, detail, detail_header, field_text, form,
        form_hx_get_route, form_hx_post_selector, form_hx_post_url, hx_nav_app_layout,
        input_number, label, modal, modal_keyed, table_button_filter, table_create_button,
    },
    html_form::{FormCtx, HtmlForm},
    http::{ProvideRequestCaps, RouteQueryBuilder},
    template::{RenderTemplate, TemplateCapability, TemplateOf, TemplateRegistrar},
    web::{modal_create_post_query, modal_edit_post_url},
};

use super::forms::{
    AssignmentFilterForm, AssignmentFilterFormField, AssignmentForm, AssignmentFormField,
};
use super::keys::{
    AssignmentBulkCreateModalKey, AssignmentBulkMarksModalKey, AssignmentCreateModalKey,
    AssignmentDeleteModalKey, AssignmentEditModalKey, AssignmentTableKey,
};
use super::routes::{
    AssignmentSubmissionsBulkCreatePostRouteTag, AssignmentSubmissionsBulkMarksPostRouteTag,
    AssignmentSubmissionsCreatePostRouteTag, AssignmentSubmissionsDeleteGetRouteTag,
    AssignmentSubmissionsDeletePostRouteTag, AssignmentSubmissionsDetailRouteTag,
    AssignmentSubmissionsEditGetRouteTag, AssignmentSubmissionsEditPostRouteTag,
    AssignmentSubmissionsListRouteTag,
};
use crate::{assignment_detail_crumbs, assignment_detail_menu, assignments_crumbs};
use nirmancampus_academicrecords::routes::AcademicRecordsDetailRouteTag;
use nirmancampus_common::{
    assignment_status_choice_pairs,
    env::ASSIGNMENT_SUBMISSIONS_SESSION_KEY,
    ui::{
        APP_TITLE, app_scaffold, empty_dash, field_related, field_vnode_many, render_pagination,
        scaffold_main, scaffold_pane, session_environment_selector, students_hub_menu,
        SessionOption,
    },
};
use nirmancampus_courses::routes::CoursesDetailRouteTag;

lariv_rs::define_register_items! {
    plugin: NirmancampusAssignmentSubmissionsTag;
    capability: TemplateCapability;
    trait: TemplateRegistrar;
    method: register_templates;
    wrapper: TemplateOf;
    bounds: [Clone, ProvideRequestCaps, Send, Sync];
    hook: Hook;
    items: [
        AssignmentListIdx: AssignmentListPageTag => AssignmentListPage,
        AssignmentDetailIdx: AssignmentDetailPageTag => AssignmentDetailPage,
        AssignmentFormIdx: AssignmentFormPageTag => AssignmentFormPage,
        ConfirmDeleteIdx: AssignmentConfirmDeletePageTag => ConfirmDeletePage,
        BulkCreateIdx: BulkCreatePageTag => BulkCreatePage,
        BulkMarksIdx: BulkMarksPageTag => BulkMarksPage,
    ]
}

lariv_rs::define_register_items! {
    plugin: NirmancampusAssignmentSubmissionsTag;
    capability: SlotCapability;
    trait: SlotRegistrar;
    method: register_slots;
    bounds: [];
    items: [];
    hook: SlotsHook;
}

#[derive(Clone)]
pub struct AssignmentRow {
    pub id: i64,
    pub assignment_title: String,
    pub course_name: String,
    pub submission_status: String,
    pub academic_record_display: String,
}

#[derive(Generic)]
pub struct AssignmentListPage {
    pub assignments: ObjectList<AssignmentRow>,
    pub filter_assignment_title: String,
    pub filter_submission_status: String,
    pub filter_academic_record_id: i64,
    pub filter_academic_record_display: String,
    pub path_and_query: String,
    pub is_admin: bool,
    pub sessions: Vec<SessionOption>,
    pub selected_session_id: i64,
}

impl AssignmentListPage {
    pub fn render_table(&self) -> Markup {
        let headers = [
            TableColumnHeader {
                key: "Assignment",
                label: "Assignment",
                sort_url: None,
                push_url: false,
            },
            TableColumnHeader {
                key: "Course",
                label: "Course",
                sort_url: None,
                push_url: false,
            },
            TableColumnHeader {
                key: "Status",
                label: "Status",
                sort_url: None,
                push_url: false,
            },
            TableColumnHeader {
                key: "AcademicRecord",
                label: "Academic record",
                sort_url: None,
                push_url: false,
            },
        ];
        let rows: Vec<TableRow> = self
            .assignments
            .items
            .iter()
            .map(|a| TableRow {
                attrs: hx_nav_app_layout(AssignmentSubmissionsDetailRouteTag::new(a.id)),
                cells: vec![
                    field_text(FieldText {
                        value: &a.assignment_title,
                        classes: "",
                    }),
                    field_text(FieldText {
                        value: empty_dash(&a.course_name),
                        classes: "",
                    }),
                    field_text(FieldText {
                        value: &a.submission_status,
                        classes: "",
                    }),
                    field_text(FieldText {
                        value: &a.academic_record_display,
                        classes: "",
                    }),
                ],
            })
            .collect();
        let choices = assignment_status_choice_pairs();
        let academic_record_id = if self.filter_academic_record_id > 0 {
            self.filter_academic_record_id.to_string()
        } else {
            String::new()
        };
        let actions = html! {
            (table_button_filter(TableButtonFilter {
                panel: form(FormOpts {
                    attrs: form_hx_get_route::<AssignmentTableKey, AssignmentSubmissionsListRouteTag>(AssignmentSubmissionsListRouteTag),
                    inputs: AssignmentFilterForm::render_inputs(
                        &FormCtx::form::<AssignmentFilterForm>()
                            .value(AssignmentFilterFormField::AssignmentTitle, &self.filter_assignment_title)
                            .value(AssignmentFilterFormField::SubmissionStatus, &self.filter_submission_status)
                            .choices(AssignmentFilterFormField::SubmissionStatus, &choices)
                            .value(AssignmentFilterFormField::AcademicRecordId, &academic_record_id)
                            .display(AssignmentFilterFormField::AcademicRecordId, &self.filter_academic_record_display),
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
                (table_create_button::<AssignmentTableKey, AssignmentCreateModalKey>(
                    Some("plus"),
                    "btn-square btn-outline btn-sm",
                ))
            }
        };
        html! {
            (session_environment_selector(
                ASSIGNMENT_SUBMISSIONS_SESSION_KEY,
                &self.sessions,
                self.selected_session_id,
            ))
            (data_table_list_grid_with_subtitle::<AssignmentTableKey>(
            "Assignment Submissions",
            "Student assignment submissions",
            actions,
            &headers,
            &rows,
            render_pagination::<AssignmentTableKey>(
                &self.path_and_query,
                self.assignments.number,
                self.assignments.num_pages,
                true,
            ),
            ))
        }
    }
}

impl lariv_rs::template::RenderAppPane for AssignmentListPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        scaffold_pane(
            students_hub_menu(),
            assignments_crumbs("All Submissions"),
            self.render_table(),
        )
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        scaffold_main(assignments_crumbs("All Submissions"), self.render_table())
    }
}

impl RenderTemplate for AssignmentListPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold(
            &format!("Assignment Submissions — {APP_TITLE}"),
            chrome,
            students_hub_menu(),
            assignments_crumbs("All Submissions"),
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
pub struct AssignmentDetailPage {
    pub id: i64,
    pub assignment_title: String,
    pub submission_status: String,
    pub max_marks: i64,
    pub marks: i64,
    pub course_id: i64,
    pub course_name: String,
    pub academic_record_id: i64,
    pub academic_record_display: String,
    pub assets: Vec<AssetLink>,
    pub is_admin: bool,
}

impl AssignmentDetailPage {
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
                    title: &self.assignment_title,
                    actions: html! {
                        @if self.is_admin {
                            (button_modal_form_route(
                                AssignmentSubmissionsEditGetRouteTag::new(self.id),
                                AssignmentSubmissionsEditPostRouteTag::new(self.id),
                                "Edit",
                                AssignmentEditModalKey::ID,
                                "btn btn-outline btn-sm",
                            ))
                        }
                    },
                }))
                (label("Course", field_related(&self.course_name, &course_href)))
                (label("Submission status", field_text(FieldText { value: &self.submission_status, classes: "" })))
                @if self.is_admin {
                    (label("Marks", field_text(FieldText { value: &marks, classes: "" })))
                }
                (label("Academic record", field_related(&self.academic_record_display, &record_href)))
                (label("Assets", field_vnode_many(&assets)))
            }))
        })
    }
}

impl lariv_rs::template::RenderAppPane for AssignmentDetailPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        scaffold_pane(
            assignment_detail_menu(self.id, &self.assignment_title, self.is_admin),
            assignment_detail_crumbs(self.id, &self.assignment_title, None),
            self.pane_body(),
        )
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        scaffold_main(
            assignment_detail_crumbs(self.id, &self.assignment_title, None),
            self.pane_body(),
        )
    }
}

impl RenderTemplate for AssignmentDetailPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold(
            &format!("{} — {APP_TITLE}", self.assignment_title),
            chrome,
            assignment_detail_menu(self.id, &self.assignment_title, self.is_admin),
            assignment_detail_crumbs(self.id, &self.assignment_title, None),
            self.pane_body(),
        )
    }
}

#[derive(Generic)]
pub struct AssignmentFormPage {
    pub id: i64,
    pub assignment_title: String,
    pub submission_status: String,
    pub max_marks: i64,
    pub marks: i64,
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

impl RenderTemplate for AssignmentFormPage {
    fn render(&self, _chrome: &ShellChrome) -> Markup {
        let is_create = self.id == 0;
        let form_attrs = if is_create {
            form_hx_post_url::<AssignmentCreateModalKey>(&modal_create_post_query(
                AssignmentSubmissionsCreatePostRouteTag,
                &self.form_name,
                &self.refresh_table,
                &self.target_input,
            ))
        } else {
            form_hx_post_url::<AssignmentEditModalKey>(&modal_edit_post_url(
                AssignmentSubmissionsEditPostRouteTag::new(self.id),
                &self.form_name,
            ))
        };
        let choices = assignment_status_choice_pairs();
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
        let body = form(FormOpts {
            attrs: form_attrs,
            title: if is_create {
                "Create submission"
            } else {
                "Edit submission"
            },
            subtitle: if is_create {
                "Record a new assignment submission."
            } else {
                "Update assignment submission details."
            },
            form_error: Some(self.error.as_str()).filter(|e| !e.is_empty()),
            inputs: AssignmentForm::render_inputs(
                &FormCtx::form::<AssignmentForm>()
                    .value(AssignmentFormField::AssignmentTitle, &self.assignment_title)
                    .value(
                        AssignmentFormField::SubmissionStatus,
                        &self.submission_status,
                    )
                    .choices(AssignmentFormField::SubmissionStatus, &choices)
                    .value(AssignmentFormField::MaxMarks, &max_marks)
                    .value(AssignmentFormField::Marks, &marks)
                    .value(AssignmentFormField::CourseId, &course_id)
                    .display(AssignmentFormField::CourseId, &self.course_display)
                    .value(AssignmentFormField::AcademicRecordId, &academic_record_id)
                    .display(
                        AssignmentFormField::AcademicRecordId,
                        &self.academic_record_display,
                    )
                    .m2m(AssignmentFormField::Assets, &self.assets),
            ),
            actions: html! {
                (button_submit(ButtonSubmit { label: "Save submission", ..Default::default() }))
                @if !is_create {
                    (button_modal_form_route(
                        AssignmentSubmissionsDeleteGetRouteTag::new(self.id),
                        AssignmentSubmissionsDeletePostRouteTag::new(self.id),
                        "Delete",
                        AssignmentDeleteModalKey::ID,
                        "btn-error",
                    ))
                }
            },
            ..Default::default()
        });
        if is_create {
            modal_keyed::<AssignmentCreateModalKey>("", body)
        } else {
            modal_keyed::<AssignmentEditModalKey>("", body)
        }
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
                    &AssignmentSubmissionsDeletePostRouteTag::new(self.id).url(),
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
    pub already: bool,
}

#[derive(Generic)]
pub struct BulkCreatePage {
    pub academic_record_id: i64,
    pub student_line: String,
    pub courses: Vec<BulkCourseRow>,
    pub error: String,
}

impl RenderTemplate for BulkCreatePage {
    fn render(&self, _chrome: &ShellChrome) -> Markup {
        let post = RouteQueryBuilder::new(AssignmentSubmissionsBulkCreatePostRouteTag)
            .query("AcademicRecordID", self.academic_record_id)
            .build_with_query();
        let aid = self.academic_record_id.to_string();
        let body = form(FormOpts {
            attrs: form_hx_post_url::<AssignmentBulkCreateModalKey>(&post),
            title: "Create submissions for student",
            subtitle: "Select compulsory and/or optional courses. Title defaults to course name; marks stay zero until you edit each submission.",
            form_error: Some(self.error.as_str()).filter(|e| !e.is_empty()),
            inputs: html! {
                (label("Student", field_text(FieldText { value: &self.student_line, classes: "" })))
                input type="hidden" name="AcademicRecordID" value=(aid);
                div class="my-1 flex flex-col gap-2" {
                    div class="label text-sm font-bold" { "Courses on this academic record" }
                    @if self.courses.is_empty() {
                        div class="text-sm opacity-80" { "No courses on this academic record." }
                    }
                    @for c in &self.courses {
                        @if c.already {
                            label class="label text-sm cursor-not-allowed justify-start gap-2 flex flex-row items-center opacity-80" {
                                input type="checkbox" class="checkbox" checked disabled;
                                span class="label-text" { (c.name) " (already submitted)" }
                            }
                        } @else {
                            label class="label text-sm font-bold cursor-pointer justify-start gap-2 flex flex-row items-center" {
                                input type="checkbox" name="CourseIDs" value=(c.id.to_string()) class="checkbox";
                                span class="label-text" { (c.name) }
                            }
                        }
                    }
                }
            },
            actions: html! {
                (button_submit(ButtonSubmit { label: "Create submissions", classes: "btn-primary", ..Default::default() }))
            },
            ..Default::default()
        });
        modal_keyed::<AssignmentBulkCreateModalKey>("", body)
    }
}

#[derive(Clone)]
pub struct BulkMarksRow {
    pub id: i64,
    pub assignment_title: String,
    pub course_name: String,
    pub max_marks: i64,
    pub marks: i64,
}

#[derive(Generic)]
pub struct BulkMarksPage {
    pub academic_record_id: i64,
    pub student_line: String,
    pub submissions: Vec<BulkMarksRow>,
    pub error: String,
}

impl RenderTemplate for BulkMarksPage {
    fn render(&self, _chrome: &ShellChrome) -> Markup {
        let post = RouteQueryBuilder::new(AssignmentSubmissionsBulkMarksPostRouteTag)
            .query("AcademicRecordID", self.academic_record_id)
            .build_with_query();
        let aid = self.academic_record_id.to_string();
        let body = form(FormOpts {
            attrs: form_hx_post_url::<AssignmentBulkMarksModalKey>(&post),
            title: "Add marks for student",
            subtitle: "Enter marks for each assignment submission. Values must not exceed max marks.",
            form_error: Some(self.error.as_str()).filter(|e| !e.is_empty()),
            inputs: html! {
                (label("Student", field_text(FieldText { value: &self.student_line, classes: "" })))
                input type="hidden" name="AcademicRecordID" value=(aid);
                @if self.submissions.is_empty() {
                    div class="text-sm opacity-80" { "No assignment submissions to mark for this record." }
                }
                @for s in &self.submissions {
                    @let title = if s.assignment_title.is_empty() {
                        format!("Submission {}", s.id)
                    } else {
                        s.assignment_title.clone()
                    };
                    @let sub_line = if s.course_name.is_empty() {
                        title.clone()
                    } else {
                        format!("{title} — {}", s.course_name)
                    };
                    @let max_str = if s.max_marks > 0 {
                        s.max_marks.to_string()
                    } else {
                        "—".into()
                    };
                    @let marks = s.marks.to_string();
                    div class="grid grid-cols-1 @md:grid-cols-2 gap-2 items-end border-b border-base-300 pb-2" {
                        div class="flex flex-col gap-0.5" {
                            div class="text-sm font-medium" { (sub_line) }
                            div class="text-xs opacity-70" { "Max marks: " (max_str) }
                        }
                        input type="hidden" name="SubmissionIDs" value=(s.id.to_string());
                        (input_number(InputNumber {
                            label: "Marks",
                            name: "Marks",
                            value: &marks,
                            required: true,
                            ..Default::default()
                        }))
                    }
                }
            },
            actions: html! {
                (button_submit(ButtonSubmit { label: "Save marks", classes: "btn-primary", ..Default::default() }))
            },
            ..Default::default()
        });
        modal_keyed::<AssignmentBulkMarksModalKey>("", body)
    }
}
