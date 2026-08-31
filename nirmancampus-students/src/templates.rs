use frunk::Generic;
use maud::{Markup, PreEscaped, html};

use lariv_rs::{
    components::{
        ButtonClear, ButtonSubmit, DeleteConfirmation, DetailHeader, FieldPhone, FieldText,
        FormOpts, ManyToManyItem, ObjectList, ShellChrome, SlotCapability, SlotRegistrar, SwapKey,
        TableButtonFilter, TableColumnHeader, TableRow, button_clear, button_modal_form_route,
        button_submit, column_sort_url, container_column, container_row,
        data_table_list_grid_with_subtitle, data_table_list_refresh, detail, detail_header,
        field_phone, field_text, form, form_hx_get_picker_route, form_hx_get_route,
        form_hx_post_selector, form_hx_post_url, hx_nav_app_layout, label, modal, modal_keyed,
        row_attr_select, sort_indicator, table_button_filter, table_create_button,
    },
    html_form::{FormCtx, HtmlForm},
    http::ProvideRequestCaps,
    picker::RenderPickerSelect,
    plugins::filesystem::routes::{VNodeDetailRouteTag, VNodeDownloadRouteTag},
    template::{RenderTemplate, TemplateCapability, TemplateOf, TemplateRegistrar},
    web::{modal_create_post_query, modal_edit_post_url},
};

use crate::{student_detail_crumbs, student_detail_menu, students_crumbs, students_menu};
use super::forms::{
    StudentFilterForm, StudentFilterFormField, StudentForm, StudentFormField,
    StudentSelectFilterForm, StudentSelectFilterFormField,
};
use super::keys::{
    StudentCreateModalKey, StudentDeleteModalKey, StudentEditModalKey, StudentSelectModalKey,
    StudentSelectTableKey, StudentTableKey,
};
use super::routes::{
    StudentsCreatePostRouteTag, StudentsDeleteGetRouteTag, StudentsDeletePostRouteTag,
    StudentsDetailRouteTag, StudentsEditGetRouteTag, StudentsEditPostRouteTag, StudentsListRouteTag,
    StudentsSelectRouteTag,
};
use nirmancampus_common::{
    category_choice_pairs,
    ui::{
        app_scaffold, empty_dash, field_vnode_many, render_pagination, render_picker_pagination,
        scaffold_main, scaffold_pane, yes_no, APP_TITLE,
    },
};

lariv_rs::define_register_items! {
    plugin: NirmancampusStudentsTag;
    capability: TemplateCapability;
    trait: TemplateRegistrar;
    method: register_templates;
    wrapper: TemplateOf;
    bounds: [Clone, ProvideRequestCaps, Send, Sync];
    hook: Hook;
    items: [
        StudentListIdx: StudentListPageTag => StudentListPage,
        StudentDetailIdx: StudentDetailPageTag => StudentDetailPage,
        StudentFormIdx: StudentFormPageTag => StudentFormPage,
        StudentSelectIdx: StudentSelectPageTag => StudentSelectPage,
        ConfirmDeleteIdx: StudentConfirmDeletePageTag => ConfirmDeletePage,
    ]
}

lariv_rs::define_register_items! {
    plugin: NirmancampusStudentsTag;
    capability: SlotCapability;
    trait: SlotRegistrar;
    method: register_slots;
    bounds: [];
    items: [];
    hook: SlotsHook;
}

#[derive(Clone)]
pub struct StudentRow {
    pub id: i64,
    pub name: String,
    pub student_no: String,
    pub aadhar_card: String,
    pub abc_id: String,
    pub email: String,
    pub phone: String,
}

#[derive(Clone)]
pub struct DocumentLink {
    pub id: i64,
    pub name: String,
}

fn category_label(key: &str) -> String {
    category_choice_pairs()
        .into_iter()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v)
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| key.to_string())
}

fn student_filter_form<K: SwapKey, R: lariv_rs::http::FragmentGet<K> + lariv_rs::http::RouteUrl + Copy + Default>(
    page: &StudentListPage,
) -> Markup {
    form(FormOpts {
        attrs: form_hx_get_route::<K, R>(R::default()),
        inputs: StudentFilterForm::render_inputs(
            &FormCtx::form::<StudentFilterForm>()
                .value(StudentFilterFormField::StudentNo, &page.filter_student_no)
                .value(StudentFilterFormField::AadharCard, &page.filter_aadhar_card)
                .value(StudentFilterFormField::AbcId, &page.filter_abc_id)
                .value(StudentFilterFormField::Name, &page.filter_name)
                .value(StudentFilterFormField::Email, &page.filter_email)
                .value(StudentFilterFormField::Phone, &page.filter_phone)
                .value(StudentFilterFormField::MotherName, &page.filter_mother_name)
                .value(StudentFilterFormField::FathersName, &page.filter_fathers_name)
                .value(StudentFilterFormField::Category, &page.filter_category),
        ),
        actions: html! {
            (container_row(
                "flex gap-2",
                html! {
                    (button_submit(ButtonSubmit { label: "Apply Filters", ..Default::default() }))
                    (button_clear(ButtonClear { label: "Clear", ..Default::default() }))
                },
            ))
        },
        ..Default::default()
    })
}

#[derive(Generic)]
pub struct StudentListPage {
    pub students: ObjectList<StudentRow>,
    pub filter_name: String,
    pub filter_student_no: String,
    pub filter_aadhar_card: String,
    pub filter_abc_id: String,
    pub filter_email: String,
    pub filter_phone: String,
    pub filter_mother_name: String,
    pub filter_fathers_name: String,
    pub filter_category: String,
    pub path_and_query: String,
    pub sort: String,
    pub is_admin: bool,
}

impl StudentListPage {
    pub fn render_table(&self) -> Markup {
        let name_sort = column_sort_url(&self.path_and_query, "Name", &self.sort);
        let name_label = format!("Name{}", sort_indicator(&self.sort, "Name"));
        let no_sort = column_sort_url(&self.path_and_query, "StudentNo", &self.sort);
        let no_label = format!(
            "Enrollment No / Control ID{}",
            sort_indicator(&self.sort, "StudentNo")
        );
        let headers = [
            TableColumnHeader {
                key: "Name",
                label: &name_label,
                sort_url: Some(&name_sort),
                push_url: true,
            },
            TableColumnHeader {
                key: "StudentNo",
                label: &no_label,
                sort_url: Some(&no_sort),
                push_url: true,
            },
            TableColumnHeader {
                key: "AadharCard",
                label: "Aadhar Card",
                sort_url: None,
                push_url: false,
            },
            TableColumnHeader {
                key: "ABCId",
                label: "ABC ID",
                sort_url: None,
                push_url: false,
            },
            TableColumnHeader {
                key: "Email",
                label: "Email",
                sort_url: None,
                push_url: false,
            },
            TableColumnHeader {
                key: "Phone",
                label: "Phone",
                sort_url: None,
                push_url: false,
            },
        ];
        let rows: Vec<TableRow> = self
            .students
            .items
            .iter()
            .map(|s| TableRow {
                attrs: hx_nav_app_layout(StudentsDetailRouteTag::new(s.id)),
                cells: vec![
                    field_text(FieldText { value: &s.name, classes: "" }),
                    field_text(FieldText { value: &s.student_no, classes: "" }),
                    field_text(FieldText { value: &s.aadhar_card, classes: "" }),
                    field_text(FieldText { value: &s.abc_id, classes: "" }),
                    field_text(FieldText { value: &s.email, classes: "" }),
                    field_phone(FieldPhone { value: &s.phone, classes: "" }),
                ],
            })
            .collect();
        let actions = html! {
            (table_button_filter(TableButtonFilter {
                panel: student_filter_form::<StudentTableKey, StudentsListRouteTag>(self),
                ..Default::default()
            }))
            @if self.is_admin {
                (table_create_button::<StudentTableKey, StudentCreateModalKey>(
                    Some("plus"),
                    "btn-square btn-outline btn-sm",
                ))
            }
        };
        data_table_list_grid_with_subtitle::<StudentTableKey>(
            "Students",
            "Student records",
            actions,
            &headers,
            &rows,
            render_pagination::<StudentTableKey>(
                &self.path_and_query,
                self.students.number,
                self.students.num_pages,
                true,
            ),
        )
    }
}

impl lariv_rs::template::RenderAppPane for StudentListPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        scaffold_pane(students_menu(), students_crumbs("All Students"), self.render_table())
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        scaffold_main(students_crumbs("All Students"), self.render_table())
    }
}

impl RenderTemplate for StudentListPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold(
            &format!("Students — {APP_TITLE}"),
            chrome,
            students_menu(),
            students_crumbs("All Students"),
            self.render_table(),
        )
    }
}

#[derive(Generic)]
pub struct StudentDetailPage {
    pub id: i64,
    pub name: String,
    pub student_no: String,
    pub aadhar_card: String,
    pub abc_id: String,
    pub email: String,
    pub phone: String,
    pub dob: String,
    pub mother_name: String,
    pub fathers_name: String,
    pub category: String,
    pub handicapped: bool,
    pub address: String,
    pub remarks: String,
    pub photo_id: Option<i64>,
    pub photo_name: String,
    pub documents: Vec<DocumentLink>,
    pub is_admin: bool,
    pub related_sections: String,
}

impl StudentDetailPage {
    fn pane_body(&self) -> Markup {
        let category = category_label(&self.category);
        detail(html! {
            (container_column(
                "",
                html! {
                    (detail_header(DetailHeader {
                        title: &self.name,
                        actions: html! {
                            @if self.is_admin {
                                (button_modal_form_route(
                                    StudentsEditGetRouteTag::new(self.id),
                                    StudentsEditPostRouteTag::new(self.id),
                                    "Edit",
                                    StudentEditModalKey::ID,
                                    "btn btn-outline btn-sm",
                                ))
                            }
                        },
                    }))
                    (label("Enrollment No / Control ID", field_text(FieldText { value: empty_dash(&self.student_no), classes: "" })))
                    (label("Aadhar Card", field_text(FieldText { value: empty_dash(&self.aadhar_card), classes: "" })))
                    (label("ABC ID", field_text(FieldText { value: empty_dash(&self.abc_id), classes: "" })))
                    (label("Email", field_text(FieldText { value: empty_dash(&self.email), classes: "" })))
                    (label("Phone", field_phone(FieldPhone { value: &self.phone, classes: "" })))
                    (label("Date of Birth", field_text(FieldText { value: empty_dash(&self.dob), classes: "" })))
                    (label("Mother's Name", field_text(FieldText { value: empty_dash(&self.mother_name), classes: "" })))
                    (label("Father's Name", field_text(FieldText { value: empty_dash(&self.fathers_name), classes: "" })))
                    (label("Category", field_text(FieldText { value: empty_dash(&category), classes: "" })))
                    (label("Handicapped", field_text(FieldText { value: yes_no(self.handicapped), classes: "" })))
                    (label("Address", field_text(FieldText { value: empty_dash(&self.address), classes: "" })))
                    (label("Remarks", field_text(FieldText { value: empty_dash(&self.remarks), classes: "" })))
                    (label("Photo", html! {
                        @if let Some(fid) = self.photo_id.filter(|&id| id > 0) {
                            a href=(VNodeDetailRouteTag::new(fid).url()) {
                                img class="w-42 rounded" src=(VNodeDownloadRouteTag::new(fid).url()) alt=(self.photo_name) {}
                            }
                        } @else {
                            (field_text(FieldText { value: "—", classes: "" }))
                        }
                    }))
                    (label("Documents", {
                        let docs: Vec<(i64, &str)> = self
                            .documents
                            .iter()
                            .map(|doc| (doc.id, doc.name.as_str()))
                            .collect();
                        field_vnode_many(&docs)
                    }))
                    @if !self.related_sections.is_empty() {
                        (PreEscaped(&self.related_sections))
                    }
                },
            ))
        })
    }
}

impl lariv_rs::template::RenderAppPane for StudentDetailPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        scaffold_pane(
            student_detail_menu(self.id, &self.name, self.is_admin),
            student_detail_crumbs(self.id, &self.name, None),
            self.pane_body(),
        )
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        scaffold_main(student_detail_crumbs(self.id, &self.name, None), self.pane_body())
    }
}

impl RenderTemplate for StudentDetailPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold(
            &format!("{} — {APP_TITLE}", self.name),
            chrome,
            student_detail_menu(self.id, &self.name, self.is_admin),
            student_detail_crumbs(self.id, &self.name, None),
            self.pane_body(),
        )
    }
}

#[derive(Generic)]
pub struct StudentFormPage {
    pub id: i64,
    pub name: String,
    pub student_no: String,
    pub aadhar_card: String,
    pub abc_id: String,
    pub email: String,
    pub phone: String,
    pub dob: String,
    pub mother_name: String,
    pub fathers_name: String,
    pub category: String,
    pub address: String,
    pub remarks: String,
    pub handicapped: bool,
    pub photo_id: String,
    pub photo_display: String,
    pub documents: Vec<ManyToManyItem>,
    pub error: String,
    pub form_name: String,
    pub refresh_table: String,
    pub target_input: String,
}

impl RenderTemplate for StudentFormPage {
    fn render(&self, _chrome: &ShellChrome) -> Markup {
        let is_create = self.id == 0;
        let form_attrs = if is_create {
            form_hx_post_url::<StudentCreateModalKey>(&modal_create_post_query(
                StudentsCreatePostRouteTag,
                &self.form_name,
                &self.refresh_table,
                &self.target_input,
            ))
        } else {
            form_hx_post_url::<StudentEditModalKey>(&modal_edit_post_url(
                StudentsEditPostRouteTag::new(self.id),
                &self.form_name,
            ))
        };
        let category_choices = category_choice_pairs();
        let body = form(FormOpts {
            attrs: form_attrs,
            title: if is_create { "Create Student" } else { "Edit Student" },
            subtitle: if is_create {
                "Create a new student"
            } else {
                "Update student details"
            },
            form_error: Some(self.error.as_str()).filter(|e| !e.is_empty()),
            inputs: StudentForm::render_inputs(
                &FormCtx::form::<StudentForm>()
                    .value(StudentFormField::Name, &self.name)
                    .value(StudentFormField::StudentNo, &self.student_no)
                    .value(StudentFormField::AadharCard, &self.aadhar_card)
                    .value(StudentFormField::AbcId, &self.abc_id)
                    .value(StudentFormField::Email, &self.email)
                    .value(StudentFormField::Phone, &self.phone)
                    .value(StudentFormField::Dob, &self.dob)
                    .value(StudentFormField::MotherName, &self.mother_name)
                    .value(StudentFormField::FathersName, &self.fathers_name)
                    .value(StudentFormField::Category, &self.category)
                    .choices(StudentFormField::Category, &category_choices)
                    .value(StudentFormField::Address, &self.address)
                    .value(StudentFormField::Remarks, &self.remarks)
                    .checked(StudentFormField::Handicapped, self.handicapped)
                    .value(StudentFormField::PhotoId, &self.photo_id)
                    .display(StudentFormField::PhotoId, &self.photo_display)
                    .m2m(StudentFormField::Documents, &self.documents),
            ),
            actions: html! {
                (button_submit(ButtonSubmit { label: "Save Student", ..Default::default() }))
                @if !is_create {
                    (button_modal_form_route(
                        StudentsDeleteGetRouteTag::new(self.id),
                        StudentsDeletePostRouteTag::new(self.id),
                        "Delete",
                        StudentDeleteModalKey::ID,
                        "btn-error",
                    ))
                }
            },
            ..Default::default()
        });
        if is_create {
            modal_keyed::<StudentCreateModalKey>("", body)
        } else {
            modal_keyed::<StudentEditModalKey>("", body)
        }
    }
}

#[derive(Generic)]
pub struct StudentSelectPage {
    pub students: ObjectList<StudentRow>,
    pub filter_name: String,
    pub filter_student_no: String,
    pub filter_phone: String,
    pub path_and_query: String,
    pub sort: String,
    pub target_input: String,
}

impl RenderPickerSelect<StudentSelectTableKey, StudentSelectModalKey> for StudentSelectPage {
    fn render_table(&self) -> Markup {
        let headers = [
            TableColumnHeader { key: "Name", label: "Name", sort_url: None, push_url: false },
            TableColumnHeader {
                key: "StudentNo",
                label: "Enrollment No / Control ID",
                sort_url: None,
                push_url: false,
            },
            TableColumnHeader { key: "Phone", label: "Phone", sort_url: None, push_url: false },
        ];
        let target = if self.target_input.is_empty() {
            "StudentID"
        } else {
            self.target_input.as_str()
        };
        let rows: Vec<TableRow> = self
            .students
            .items
            .iter()
            .map(|s| TableRow {
                attrs: row_attr_select(target, &s.id.to_string(), &s.name),
                cells: vec![
                    field_text(FieldText { value: &s.name, classes: "" }),
                    field_text(FieldText { value: &s.student_no, classes: "" }),
                    field_phone(FieldPhone { value: &s.phone, classes: "" }),
                ],
            })
            .collect();
        data_table_list_refresh::<StudentSelectTableKey>(
            "Select Student",
            table_button_filter(TableButtonFilter {
                panel: form(FormOpts {
                    attrs: form_hx_get_picker_route::<
                        StudentSelectTableKey,
                        StudentSelectModalKey,
                        StudentsSelectRouteTag,
                    >(StudentsSelectRouteTag)
                        .set("hx-push-url", "false"),
                    inputs: html! {
                        (StudentSelectFilterForm::render_inputs(
                            &FormCtx::form::<StudentSelectFilterForm>()
                                .value(StudentSelectFilterFormField::Name, &self.filter_name)
                                .value(StudentSelectFilterFormField::StudentNo, &self.filter_student_no)
                                .value(StudentSelectFilterFormField::Phone, &self.filter_phone),
                        ))
                        input type="hidden" name="target_input" value=(self.target_input) {}
                    },
                    actions: html! {
                        (button_submit(ButtonSubmit { label: "Apply", ..Default::default() }))
                    },
                    ..Default::default()
                }),
                ..Default::default()
            }),
            &headers,
            &rows,
            render_picker_pagination::<StudentSelectModalKey>(
                &self.path_and_query,
                self.students.number,
                self.students.num_pages,
            ),
            &self.path_and_query,
        )
    }
}

impl RenderTemplate for StudentSelectPage {
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
                    &StudentsDeletePostRouteTag::new(self.id).url(),
                    &target,
                ),
                form_error: Some(self.error.as_str()).filter(|e| !e.is_empty()),
                ..Default::default()
            }),
            ..Default::default()
        })
    }
}
