use frunk::Generic;
use maud::{Markup, PreEscaped, html};

use lariv_rs::{
    components::{
        ButtonClear, ButtonSubmit, DeleteConfirmation, DetailHeader, FieldManyToMany, FieldSubtitle,
        FieldText, FieldTitle, FormOpts, HtmlAttrs, InputDate, InputManyToMany, InputSelect,
        InputSelectOption, ManyToManyItem, ObjectList, ShellChrome, SlotCapability, SlotRegistrar,
        SwapKey, TableButtonFilter, TableColumnHeader, TableRow, button_clear,
        button_download_route, button_modal_form_route, button_submit, container_column,
        container_row, data_table_list_grid_with_subtitle, data_table_list_refresh, detail,
        detail_header, field_many_to_many, field_subtitle, field_text, field_title, form,
        form_hx_get_picker_route, form_hx_get_route, form_hx_post_selector, form_hx_post_url,
        hx_nav_app_layout, input_date, input_many_to_many, input_select, label, modal, modal_keyed,
        row_attr_select, table_button_filter, table_create_button,
    },
    html_form::{FormCtx, FormFieldKey, HtmlForm},
    http::ProvideRequestCaps,
    picker::RenderPickerSelect,
    template::{RenderTemplate, TemplateCapability, TemplateOf, TemplateRegistrar},
    web::{modal_create_post_query, modal_edit_post_url},
};

use crate::{academic_record_detail_crumbs, academic_record_detail_menu, academic_records_crumbs};
use super::forms::{
    AcademicRecordFilterForm, AcademicRecordFilterFormField, AcademicRecordForm,
    AcademicRecordFormField,
};
use super::keys::{
    AcademicRecordCreateModalKey, AcademicRecordDeleteModalKey, AcademicRecordEditModalKey,
    AcademicRecordSelectModalKey, AcademicRecordSelectTableKey, AcademicRecordTableKey,
    PsuSelectModalKey, PsuSelectTableKey,
};
use super::routes::{
    AcademicRecordsCreatePostRouteTag, AcademicRecordsDeleteGetRouteTag,
    AcademicRecordsDeletePostRouteTag, AcademicRecordsDetailRouteTag,
    AcademicRecordsDownloadPdfRouteTag, AcademicRecordsEditGetRouteTag,
    AcademicRecordsEditPostRouteTag, AcademicRecordsListRouteTag, AcademicRecordsSelectRouteTag,
};
use nirmancampus_common::{
    academic_record_status_choice_pairs,
    env::ACADEMIC_RECORDS_SESSION_KEY,
    ui::{
        app_scaffold, empty_dash, field_related, field_related_many, render_pagination,
        render_picker_pagination, scaffold_main, scaffold_pane, session_environment_selector,
        students_hub_menu, SessionOption, APP_TITLE,
    },
};
use nirmancampus_courses::routes::CoursesDetailRouteTag;
use nirmancampus_programs::routes::ProgramsDetailRouteTag;
use nirmancampus_sessions::routes::SessionsDetailRouteTag;
use nirmancampus_students::routes::StudentsDetailRouteTag;

lariv_rs::define_register_items! {
    plugin: NirmancampusAcademicRecordsTag;
    capability: TemplateCapability;
    trait: TemplateRegistrar;
    method: register_templates;
    wrapper: TemplateOf;
    bounds: [Clone, ProvideRequestCaps, Send, Sync];
    hook: Hook;
    items: [
        AcademicRecordListIdx: AcademicRecordListPageTag => AcademicRecordListPage,
        AcademicRecordDetailIdx: AcademicRecordDetailPageTag => AcademicRecordDetailPage,
        AcademicRecordFormIdx: AcademicRecordFormPageTag => AcademicRecordFormPage,
        AcademicRecordSelectIdx: AcademicRecordSelectPageTag => AcademicRecordSelectPage,
        PsuSelectIdx: PsuSelectPageTag => PsuSelectPage,
        ConfirmDeleteIdx: AcademicRecordConfirmDeletePageTag => ConfirmDeletePage,
    ]
}

lariv_rs::define_register_items! {
    plugin: NirmancampusAcademicRecordsTag;
    capability: SlotCapability;
    trait: SlotRegistrar;
    method: register_slots;
    bounds: [];
    items: [];
    hook: SlotsHook;
}

#[derive(Clone)]
pub struct AcademicRecordRow {
    pub id: i64,
    pub student_display: String,
    pub program_display: String,
    pub term: String,
    pub session_name: String,
    pub status: String,
}

#[derive(Clone)]
pub struct CourseLink {
    pub id: i64,
    pub name: String,
}

#[derive(Clone)]
pub struct PsuRow {
    pub id: i64,
    pub term: String,
    pub optional_count: String,
}

fn course_list(courses: &[CourseLink]) -> Markup {
    let urls: Vec<String> = courses
        .iter()
        .map(|c| CoursesDetailRouteTag::new(c.id).url())
        .collect();
    let items: Vec<(&str, Option<&str>)> = courses
        .iter()
        .zip(urls.iter())
        .map(|(c, url)| (c.name.as_str(), Some(url.as_str())))
        .collect();
    field_related_many(&items)
}

#[derive(Generic)]
pub struct AcademicRecordListPage {
    pub records: ObjectList<AcademicRecordRow>,
    pub filter_status: String,
    pub filter_term: String,
    pub filter_program_id: i64,
    pub filter_program_display: String,
    pub path_and_query: String,
    pub is_admin: bool,
    pub sessions: Vec<SessionOption>,
    pub selected_session_id: i64,
}

impl AcademicRecordListPage {
    pub fn render_table(&self) -> Markup {
        let headers = [
            TableColumnHeader {
                key: "Student",
                label: "Student",
                sort_url: None,
                push_url: false,
            },
            TableColumnHeader {
                key: "Program",
                label: "Program",
                sort_url: None,
                push_url: false,
            },
            TableColumnHeader {
                key: "Term",
                label: "Term",
                sort_url: None,
                push_url: false,
            },
            TableColumnHeader {
                key: "AdmissionSession",
                label: "Admission session",
                sort_url: None,
                push_url: false,
            },
            TableColumnHeader {
                key: "Status",
                label: "Status",
                sort_url: None,
                push_url: false,
            },
        ];
        let rows: Vec<TableRow> = self
            .records
            .items
            .iter()
            .map(|r| TableRow {
                attrs: hx_nav_app_layout(AcademicRecordsDetailRouteTag::new(r.id)),
                cells: vec![
                    field_text(FieldText {
                        value: &r.student_display,
                        classes: "",
                    }),
                    field_text(FieldText {
                        value: &r.program_display,
                        classes: "",
                    }),
                    field_text(FieldText {
                        value: empty_dash(&r.term),
                        classes: "",
                    }),
                    field_text(FieldText {
                        value: empty_dash(&r.session_name),
                        classes: "",
                    }),
                    field_text(FieldText {
                        value: &r.status,
                        classes: "",
                    }),
                ],
            })
            .collect();
        let program_id = if self.filter_program_id > 0 {
            self.filter_program_id.to_string()
        } else {
            String::new()
        };
        let choices = academic_record_status_choice_pairs();
        let actions = html! {
            (table_button_filter(TableButtonFilter {
                panel: form(FormOpts {
                    attrs: form_hx_get_route::<AcademicRecordTableKey, AcademicRecordsListRouteTag>(
                        AcademicRecordsListRouteTag,
                    ),
                    inputs: AcademicRecordFilterForm::render_inputs(
                        &FormCtx::form::<AcademicRecordFilterForm>()
                            .value(AcademicRecordFilterFormField::Status, &self.filter_status)
                            .choices(AcademicRecordFilterFormField::Status, &choices)
                            .value(AcademicRecordFilterFormField::Term, &self.filter_term)
                            .value(AcademicRecordFilterFormField::ProgramId, &program_id)
                            .display(AcademicRecordFilterFormField::ProgramId, &self.filter_program_display),
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
                (table_create_button::<AcademicRecordTableKey, AcademicRecordCreateModalKey>(
                    Some("plus"),
                    "btn-square btn-outline btn-sm",
                ))
            }
        };
        html! {
            (session_environment_selector(
                ACADEMIC_RECORDS_SESSION_KEY,
                &self.sessions,
                self.selected_session_id,
            ))
            (data_table_list_grid_with_subtitle::<AcademicRecordTableKey>(
                "Academic Records",
                "Enrolment by student, program, and term",
                actions,
                &headers,
                &rows,
                render_pagination::<AcademicRecordTableKey>(
                    &self.path_and_query,
                    self.records.number,
                    self.records.num_pages,
                    true,
                ),
            ))
        }
    }
}

impl lariv_rs::template::RenderAppPane for AcademicRecordListPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        scaffold_pane(
            students_hub_menu(),
            academic_records_crumbs("All Academic Records"),
            self.render_table(),
        )
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        scaffold_main(
            academic_records_crumbs("All Academic Records"),
            self.render_table(),
        )
    }
}

impl RenderTemplate for AcademicRecordListPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold(
            &format!("Academic Records — {APP_TITLE}"),
            chrome,
            students_hub_menu(),
            academic_records_crumbs("All Academic Records"),
            self.render_table(),
        )
    }
}

#[derive(Generic)]
pub struct AcademicRecordDetailPage {
    pub id: i64,
    pub student_id: i64,
    pub student_name: String,
    pub student_no: String,
    pub program_id: i64,
    pub program_display: String,
    pub session_id: i64,
    pub session_name: String,
    pub status: String,
    pub date: String,
    pub term: String,
    pub compulsory: Vec<CourseLink>,
    pub optional: Vec<CourseLink>,
    pub related_sections: String,
    pub is_admin: bool,
}

impl AcademicRecordDetailPage {
    fn pane_body(&self) -> Markup {
        let student_href = if self.student_id > 0 {
            StudentsDetailRouteTag::new(self.student_id).url()
        } else {
            String::new()
        };
        let program_href = if self.program_id > 0 {
            ProgramsDetailRouteTag::new(self.program_id).url()
        } else {
            String::new()
        };
        let session_href = if self.session_id > 0 {
            SessionsDetailRouteTag::new(self.session_id).url()
        } else {
            String::new()
        };
        detail(html! {
            (container_column("", html! {
                (detail_header(DetailHeader {
                    title: &self.student_name,
                    actions: html! {
                        (button_download_route(
                            AcademicRecordsDownloadPdfRouteTag::new(self.id),
                            "Download PDF",
                            "btn-outline btn-secondary btn-sm",
                        ))
                        @if self.is_admin {
                            (button_modal_form_route(
                                AcademicRecordsEditGetRouteTag::new(self.id),
                                AcademicRecordsEditPostRouteTag::new(self.id),
                                "Edit",
                                AcademicRecordEditModalKey::ID,
                                "btn btn-outline btn-sm",
                            ))
                        }
                    },
                }))
                (field_title(FieldTitle { value: &self.student_name, classes: "" }))
                (field_subtitle(FieldSubtitle { value: &self.student_no, classes: "" }))
                (label("Student", field_related(&self.student_name, &student_href)))
                (label("Program", field_related(&self.program_display, &program_href)))
                (label("Admission session", field_related(&self.session_name, &session_href)))
                (label("Status", field_text(FieldText { value: &self.status, classes: "" })))
                (label("Admission date", field_text(FieldText { value: empty_dash(&self.date), classes: "" })))
                (label("Term", field_text(FieldText { value: empty_dash(&self.term), classes: "" })))
                (label("Compulsory courses", course_list(&self.compulsory)))
                (label("Optional courses", course_list(&self.optional)))
                @if !self.related_sections.is_empty() {
                    (PreEscaped(&self.related_sections))
                }
            }))
        })
    }
}

impl lariv_rs::template::RenderAppPane for AcademicRecordDetailPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        scaffold_pane(
            academic_record_detail_menu(self.id, &self.student_name, self.is_admin),
            academic_record_detail_crumbs(self.id, &self.student_name, None),
            self.pane_body(),
        )
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        scaffold_main(
            academic_record_detail_crumbs(self.id, &self.student_name, None),
            self.pane_body(),
        )
    }
}

impl RenderTemplate for AcademicRecordDetailPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold(
            &format!("{} — {APP_TITLE}", self.student_name),
            chrome,
            academic_record_detail_menu(self.id, &self.student_name, self.is_admin),
            academic_record_detail_crumbs(self.id, &self.student_name, None),
            self.pane_body(),
        )
    }
}

#[derive(Generic)]
pub struct AcademicRecordFormPage {
    pub id: i64,
    pub session_id: i64,
    pub session_display: String,
    pub student_id: i64,
    pub student_display: String,
    pub program_id: i64,
    pub program_display: String,
    pub status: String,
    pub date: String,
    pub program_structure_unit_id: i64,
    pub term_display: String,
    pub optional_course_count: String,
    pub compulsory: Vec<ManyToManyItem>,
    pub optional: Vec<ManyToManyItem>,
    pub optional_pool_url: String,
    pub error: String,
    pub form_name: String,
    pub refresh_table: String,
    pub target_input: String,
    pub locked_identity: bool,
}

impl RenderTemplate for AcademicRecordFormPage {
    fn render(&self, _chrome: &ShellChrome) -> Markup {
        let is_create = self.id == 0;
        let form_attrs = if is_create {
            form_hx_post_url::<AcademicRecordCreateModalKey>(&modal_create_post_query(
                AcademicRecordsCreatePostRouteTag,
                &self.form_name,
                &self.refresh_table,
                &self.target_input,
            ))
        } else {
            form_hx_post_url::<AcademicRecordEditModalKey>(&modal_edit_post_url(
                AcademicRecordsEditPostRouteTag::new(self.id),
                &self.form_name,
            ))
        };
        let choices = academic_record_status_choice_pairs();
        let session_id = if self.session_id > 0 {
            self.session_id.to_string()
        } else {
            String::new()
        };
        let student_id = if self.student_id > 0 {
            self.student_id.to_string()
        } else {
            String::new()
        };
        let program_id = if self.program_id > 0 {
            self.program_id.to_string()
        } else {
            String::new()
        };
        let psu_id = if self.program_structure_unit_id > 0 {
            self.program_structure_unit_id.to_string()
        } else {
            String::new()
        };
        let psu_select_url = if self.program_id > 0 {
            format!(
                "/academic-records/program-structure-units/select/?ProgramID={}",
                self.program_id
            )
        } else {
            "/academic-records/program-structure-units/select/".into()
        };
        let ctx = FormCtx::form::<AcademicRecordForm>()
            .value(AcademicRecordFormField::SessionId, &session_id)
            .display(AcademicRecordFormField::SessionId, &self.session_display)
            .value(AcademicRecordFormField::StudentId, &student_id)
            .display(AcademicRecordFormField::StudentId, &self.student_display)
            .value(AcademicRecordFormField::ProgramId, &program_id)
            .display(AcademicRecordFormField::ProgramId, &self.program_display)
            .value(AcademicRecordFormField::Status, &self.status)
            .choices(AcademicRecordFormField::Status, &choices)
            .value(AcademicRecordFormField::Date, &self.date)
            .value(AcademicRecordFormField::ProgramStructureUnitId, &psu_id)
            .display(AcademicRecordFormField::ProgramStructureUnitId, &self.term_display)
            .url(AcademicRecordFormField::ProgramStructureUnitId, &psu_select_url)
            .url(AcademicRecordFormField::OptionalCourses, &self.optional_pool_url)
            .m2m(AcademicRecordFormField::OptionalCourses, &self.optional);
        let compulsory_pairs: Vec<(&str, Option<&str>)> = self
            .compulsory
            .iter()
            .map(|c| (c.value.as_str(), None))
            .collect();
        let status_options: Vec<InputSelectOption<'_>> = choices
            .iter()
            .map(|(value, label)| InputSelectOption {
                value,
                label,
                selected: value == &self.status,
            })
            .collect();
        let identity_readout = html! {
            (label("Student", field_text(FieldText {
                value: empty_dash(&self.student_display),
                classes: "",
            })))
            (label("Program", field_text(FieldText {
                value: empty_dash(&self.program_display),
                classes: "",
            })))
            (label("Admission session", field_text(FieldText {
                value: empty_dash(&self.session_display),
                classes: "",
            })))
            (label("Term", field_text(FieldText {
                value: empty_dash(&self.term_display),
                classes: "",
            })))
        };
        let identity_hidden = html! {
            input type="hidden" name=(AcademicRecordFormField::SessionId.html_name()) value=(session_id);
            input type="hidden" name=(AcademicRecordFormField::StudentId.html_name()) value=(student_id);
            input type="hidden" name=(AcademicRecordFormField::ProgramId.html_name()) value=(program_id);
            input type="hidden" name=(AcademicRecordFormField::ProgramStructureUnitId.html_name()) value=(psu_id);
        };
        let course_extras = html! {
            (label("Compulsory courses", field_many_to_many(FieldManyToMany {
                items: &compulsory_pairs,
                classes: "w-full",
            })))
            (label("Optional course count", field_text(FieldText {
                value: empty_dash(&self.optional_course_count),
                classes: "",
            })))
        };
        let inputs = if self.locked_identity {
            html! {
                (identity_readout)
                (input_date(InputDate {
                    label: "Admission date",
                    name: AcademicRecordFormField::Date.html_name(),
                    value: &self.date,
                    required: true,
                    ..Default::default()
                }))
                (identity_hidden)
                (course_extras)
                (input_many_to_many(InputManyToMany {
                    label: "Optional courses",
                    name: AcademicRecordFormField::OptionalCourses.html_name(),
                    items: &self.optional,
                    placeholder: "Select optional courses from the program pool…",
                    url: &self.optional_pool_url,
                    attrs: HtmlAttrs::new().set("id", "fk-academic-record-optional-courses"),
                    ..Default::default()
                }))
                (input_select(InputSelect {
                    label: "Status",
                    name: AcademicRecordFormField::Status.html_name(),
                    required: true,
                    options: &status_options,
                    ..Default::default()
                }))
            }
        } else {
            html! {
                (AcademicRecordForm::render_inputs(&ctx))
                (course_extras)
            }
        };
        let body = form(FormOpts {
            attrs: form_attrs,
            title: if is_create {
                "Create Academic Record"
            } else {
                "Edit Academic Record"
            },
            subtitle: if is_create {
                "Pick student, program, admission session, term, and optional courses."
            } else {
                "Update status or course selections. Student, program, and term cannot be changed here."
            },
            form_error: Some(self.error.as_str()).filter(|e| !e.is_empty()),
            inputs,
            actions: html! {
                (button_submit(ButtonSubmit { label: "Save Academic Record", ..Default::default() }))
                @if !is_create {
                    (button_modal_form_route(
                        AcademicRecordsDeleteGetRouteTag::new(self.id),
                        AcademicRecordsDeletePostRouteTag::new(self.id),
                        "Delete",
                        AcademicRecordDeleteModalKey::ID,
                        "btn-error",
                    ))
                }
            },
            ..Default::default()
        });
        if is_create {
            modal_keyed::<AcademicRecordCreateModalKey>("", body)
        } else {
            modal_keyed::<AcademicRecordEditModalKey>("", body)
        }
    }
}

#[derive(Generic)]
pub struct AcademicRecordSelectPage {
    pub records: ObjectList<AcademicRecordRow>,
    pub filter_status: String,
    pub path_and_query: String,
    pub target_input: String,
}

impl RenderPickerSelect<AcademicRecordSelectTableKey, AcademicRecordSelectModalKey>
    for AcademicRecordSelectPage
{
    fn render_table(&self) -> Markup {
        let headers = [
            TableColumnHeader {
                key: "Program",
                label: "Program",
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
                key: "Term",
                label: "Term",
                sort_url: None,
                push_url: false,
            },
        ];
        let target = if self.target_input.is_empty() {
            "AcademicRecordID"
        } else {
            self.target_input.as_str()
        };
        let rows: Vec<TableRow> = self
            .records
            .items
            .iter()
            .map(|r| {
                let label = format!("{} ({}) · {}", r.program_display, r.status, r.term);
                TableRow {
                    attrs: row_attr_select(target, &r.id.to_string(), &label),
                    cells: vec![
                        field_text(FieldText {
                            value: &r.program_display,
                            classes: "",
                        }),
                        field_text(FieldText {
                            value: &r.status,
                            classes: "",
                        }),
                        field_text(FieldText {
                            value: empty_dash(&r.term),
                            classes: "",
                        }),
                    ],
                }
            })
            .collect();
        let choices = academic_record_status_choice_pairs();
        data_table_list_refresh::<AcademicRecordSelectTableKey>(
            "Select Academic Record",
            table_button_filter(TableButtonFilter {
                panel: form(FormOpts {
                    attrs: form_hx_get_picker_route::<
                        AcademicRecordSelectTableKey,
                        AcademicRecordSelectModalKey,
                        AcademicRecordsSelectRouteTag,
                    >(AcademicRecordsSelectRouteTag)
                        .set("hx-push-url", "false"),
                    inputs: html! {
                        (AcademicRecordFilterForm::render_inputs(
                            &FormCtx::form::<AcademicRecordFilterForm>()
                                .value(AcademicRecordFilterFormField::Status, &self.filter_status)
                                .choices(AcademicRecordFilterFormField::Status, &choices),
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
            render_picker_pagination::<AcademicRecordSelectModalKey>(
                &self.path_and_query,
                self.records.number,
                self.records.num_pages,
            ),
            &self.path_and_query,
        )
    }
}

impl RenderTemplate for AcademicRecordSelectPage {
    fn render(&self, _chrome: &ShellChrome) -> Markup {
        self.render_modal().into_inner()
    }
}

#[derive(Generic)]
pub struct PsuSelectPage {
    pub units: ObjectList<PsuRow>,
    pub program_id: i64,
    pub path_and_query: String,
    pub target_input: String,
}

impl RenderPickerSelect<PsuSelectTableKey, PsuSelectModalKey> for PsuSelectPage {
    fn render_table(&self) -> Markup {
        let headers = [
            TableColumnHeader {
                key: "TermNumber",
                label: "Term",
                sort_url: None,
                push_url: false,
            },
            TableColumnHeader {
                key: "OptionalCourseCount",
                label: "Optional count",
                sort_url: None,
                push_url: false,
            },
        ];
        let target = if self.target_input.is_empty() {
            "ProgramStructureUnitID"
        } else {
            self.target_input.as_str()
        };
        let rows: Vec<TableRow> = self
            .units
            .items
            .iter()
            .map(|u| TableRow {
                attrs: row_attr_select(target, &u.id.to_string(), &u.term),
                cells: vec![
                    field_text(FieldText {
                        value: &u.term,
                        classes: "",
                    }),
                    field_text(FieldText {
                        value: &u.optional_count,
                        classes: "",
                    }),
                ],
            })
            .collect();
        data_table_list_refresh::<PsuSelectTableKey>(
            "Select Term",
            html! {},
            &headers,
            &rows,
            render_picker_pagination::<PsuSelectModalKey>(
                &self.path_and_query,
                self.units.number,
                self.units.num_pages,
            ),
            &self.path_and_query,
        )
    }
}

impl RenderTemplate for PsuSelectPage {
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
                    &AcademicRecordsDeletePostRouteTag::new(self.id).url(),
                    &target,
                ),
                form_error: Some(self.error.as_str()).filter(|e| !e.is_empty()),
                ..Default::default()
            }),
            ..Default::default()
        })
    }
}
