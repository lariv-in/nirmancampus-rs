use frunk::Generic;
use maud::{Markup, PreEscaped, html};

use lariv_rs::{
    components::{
        ButtonClear, ButtonSubmit, DeleteConfirmation, DetailHeader, FieldText, FormOpts,
        ObjectList, ShellChrome, SlotCapability, SlotRegistrar, SwapKey, TableButtonFilter,
        TableColumnHeader, TableRow, button_clear, button_modal_form_route, button_submit,
        column_sort_url, container_column, container_row, data_table_list_grid_with_subtitle,
        data_table_list_refresh, detail, detail_header, field_text, form, form_hx_get_picker_route,
        form_hx_get_route, form_hx_post_selector, form_hx_post_url, hx_nav_app_layout, label, modal,
        modal_keyed, row_attr_select, row_attr_select_multi, sort_indicator,
        table_button_filter, table_create_button,
    },
    html_form::{FormCtx, HtmlForm},
    http::ProvideRequestCaps,
    picker::RenderPickerSelect,
    template::{RenderTemplate, TemplateCapability, TemplateOf, TemplateRegistrar},
    web::{modal_create_post_query, modal_edit_post_url},
};

use crate::{course_detail_crumbs, course_detail_menu, courses_crumbs, courses_menu};
use super::forms::{CourseFilterForm, CourseFilterFormField, CourseForm, CourseFormField};
use super::keys::{
    CourseCreateModalKey, CourseDeleteModalKey, CourseEditModalKey, CourseMultiSelectModalKey,
    CourseMultiSelectTableKey, CourseSelectModalKey, CourseSelectTableKey, CourseTableKey,
};
use super::routes::{
    CoursesCreatePostRouteTag, CoursesDeleteGetRouteTag,
    CoursesDeletePostRouteTag, CoursesDetailRouteTag, CoursesEditGetRouteTag,
    CoursesEditPostRouteTag, CoursesListRouteTag, CoursesMultiSelectRouteTag, CoursesSelectRouteTag,
};
use nirmancampus_common::{
    format_inr,
    ui::{
        app_scaffold, empty_dash, render_pagination, render_picker_pagination, scaffold_main,
        scaffold_pane, yes_no, APP_TITLE,
    },
};

lariv_rs::define_register_items! {
    plugin: NirmancampusCoursesTag;
    capability: TemplateCapability;
    trait: TemplateRegistrar;
    method: register_templates;
    wrapper: TemplateOf;
    bounds: [Clone, ProvideRequestCaps, Send, Sync];
    hook: Hook;
    items: [
        CourseListIdx: CourseListPageTag => CourseListPage,
        CourseDetailIdx: CourseDetailPageTag => CourseDetailPage,
        CourseFormIdx: CourseFormPageTag => CourseFormPage,
        CourseSelectIdx: CourseSelectPageTag => CourseSelectPage,
        CourseMultiSelectIdx: CourseMultiSelectPageTag => CourseMultiSelectPage,
        ConfirmDeleteIdx: CourseConfirmDeletePageTag => ConfirmDeletePage,
    ]
}

lariv_rs::define_register_items! {
    plugin: NirmancampusCoursesTag;
    capability: SlotCapability;
    trait: SlotRegistrar;
    method: register_slots;
    bounds: [];
    items: [];
    hook: SlotsHook;
}

#[derive(Clone)]
pub struct CourseRow {
    pub id: i64,
    pub name: String,
    pub code: String,
    pub course_type: String,
    pub fee: i64,
    pub is_active: bool,
}

fn course_filter_form<K: SwapKey, R: lariv_rs::http::FragmentGet<K> + lariv_rs::http::RouteUrl + Copy + Default>(
    name: &str,
    code: &str,
    course_type: &str,
) -> Markup {
    form(FormOpts {
        attrs: form_hx_get_route::<K, R>(R::default()),
        inputs: CourseFilterForm::render_inputs(
            &FormCtx::form::<CourseFilterForm>()
                .value(CourseFilterFormField::Name, name)
                .value(CourseFilterFormField::Code, code)
                .value(CourseFilterFormField::CourseType, course_type),
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
pub struct CourseListPage {
    pub courses: ObjectList<CourseRow>,
    pub filter_name: String,
    pub filter_code: String,
    pub filter_course_type: String,
    pub path_and_query: String,
    pub sort: String,
    pub is_admin: bool,
}

impl CourseListPage {
    pub fn render_table(&self) -> Markup {
        let name_sort = column_sort_url(&self.path_and_query, "Name", &self.sort);
        let name_label = format!("Name{}", sort_indicator(&self.sort, "Name"));
        let code_sort = column_sort_url(&self.path_and_query, "Code", &self.sort);
        let code_label = format!("Code{}", sort_indicator(&self.sort, "Code"));
        let fee_sort = column_sort_url(&self.path_and_query, "Fee", &self.sort);
        let fee_label = format!("Fee{}", sort_indicator(&self.sort, "Fee"));
        let headers = [
            TableColumnHeader {
                key: "Name",
                label: &name_label,
                sort_url: Some(&name_sort),
                push_url: true,
            },
            TableColumnHeader {
                key: "Code",
                label: &code_label,
                sort_url: Some(&code_sort),
                push_url: true,
            },
            TableColumnHeader {
                key: "Type",
                label: "Type",
                sort_url: None,
                push_url: false,
            },
            TableColumnHeader {
                key: "Fee",
                label: &fee_label,
                sort_url: Some(&fee_sort),
                push_url: true,
            },
            TableColumnHeader {
                key: "Active",
                label: "Active",
                sort_url: None,
                push_url: false,
            },
        ];
        let rows: Vec<TableRow> = self
            .courses
            .items
            .iter()
            .map(|c| {
                let fee = format_inr(c.fee);
                TableRow {
                    attrs: hx_nav_app_layout(CoursesDetailRouteTag::new(c.id)),
                    cells: vec![
                        field_text(FieldText { value: &c.name, classes: "" }),
                        field_text(FieldText { value: &c.code, classes: "" }),
                        field_text(FieldText { value: &c.course_type, classes: "" }),
                        field_text(FieldText { value: &fee, classes: "" }),
                        field_text(FieldText { value: yes_no(c.is_active), classes: "" }),
                    ],
                }
            })
            .collect();
        let actions = html! {
            (table_button_filter(TableButtonFilter {
                panel: course_filter_form::<CourseTableKey, CoursesListRouteTag>(
                    &self.filter_name,
                    &self.filter_code,
                    &self.filter_course_type,
                ),
                ..Default::default()
            }))
            @if self.is_admin {
                (table_create_button::<CourseTableKey, CourseCreateModalKey>(
                    Some("plus"),
                    "btn-square btn-outline btn-sm",
                ))
            }
        };
        data_table_list_grid_with_subtitle::<CourseTableKey>(
            "Courses",
            "Course catalog",
            actions,
            &headers,
            &rows,
            render_pagination::<CourseTableKey>(
                &self.path_and_query,
                self.courses.number,
                self.courses.num_pages,
                true,
            ),
        )
    }
}

impl lariv_rs::template::RenderAppPane for CourseListPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        scaffold_pane(courses_menu(), courses_crumbs("All Courses"), self.render_table())
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        scaffold_main(courses_crumbs("All Courses"), self.render_table())
    }
}

impl RenderTemplate for CourseListPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold(
            &format!("Courses — {APP_TITLE}"),
            chrome,
            courses_menu(),
            courses_crumbs("All Courses"),
            self.render_table(),
        )
    }
}

#[derive(Generic)]
pub struct CourseDetailPage {
    pub id: i64,
    pub name: String,
    pub code: String,
    pub course_type: String,
    pub description: String,
    pub fee: i64,
    pub is_active: bool,
    pub is_admin: bool,
    pub related_sections: String,
}

impl CourseDetailPage {
    fn pane_body(&self) -> Markup {
        let fee = format_inr(self.fee);
        detail(html! {
            (container_column(
                "",
                html! {
                    (detail_header(DetailHeader {
                        title: &self.name,
                        actions: html! {
                            @if self.is_admin {
                                (button_modal_form_route(
                                    CoursesEditGetRouteTag::new(self.id),
                                    CoursesEditPostRouteTag::new(self.id),
                                    "Edit",
                                    CourseEditModalKey::ID,
                                    "btn btn-outline btn-sm",
                                ))
                            }
                        },
                    }))
                    (label("Code", field_text(FieldText { value: empty_dash(&self.code), classes: "" })))
                    (label("Type", field_text(FieldText { value: empty_dash(&self.course_type), classes: "" })))
                    (label("Fee", field_text(FieldText { value: &fee, classes: "" })))
                    (label("Active", field_text(FieldText { value: yes_no(self.is_admin && self.is_active || self.is_active), classes: "" })))
                    (label("Description", field_text(FieldText { value: empty_dash(&self.description), classes: "" })))
                    @if !self.related_sections.is_empty() {
                        (PreEscaped(&self.related_sections))
                    }
                },
            ))
        })
    }
}

impl lariv_rs::template::RenderAppPane for CourseDetailPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        scaffold_pane(
            course_detail_menu(self.id, &self.name, self.is_admin),
            course_detail_crumbs(self.id, &self.name, None),
            self.pane_body(),
        )
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        scaffold_main(course_detail_crumbs(self.id, &self.name, None), self.pane_body())
    }
}

impl RenderTemplate for CourseDetailPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold(
            &format!("{} — {APP_TITLE}", self.name),
            chrome,
            course_detail_menu(self.id, &self.name, self.is_admin),
            course_detail_crumbs(self.id, &self.name, None),
            self.pane_body(),
        )
    }
}

#[derive(Generic)]
pub struct CourseFormPage {
    pub id: i64,
    pub name: String,
    pub code: String,
    pub course_type: String,
    pub description: String,
    pub fee: i64,
    pub is_active: bool,
    pub error: String,
    pub form_name: String,
    pub refresh_table: String,
    pub target_input: String,
}

impl RenderTemplate for CourseFormPage {
    fn render(&self, _chrome: &ShellChrome) -> Markup {
        let is_create = self.id == 0;
        let form_attrs = if is_create {
            form_hx_post_url::<CourseCreateModalKey>(&modal_create_post_query(
                CoursesCreatePostRouteTag,
                &self.form_name,
                &self.refresh_table,
                &self.target_input,
            ))
        } else {
            form_hx_post_url::<CourseEditModalKey>(&modal_edit_post_url(
                CoursesEditPostRouteTag::new(self.id),
                &self.form_name,
            ))
        };
        let fee = self.fee.to_string();
        let body = form(FormOpts {
            attrs: form_attrs,
            title: if is_create { "Create Course" } else { "Edit Course" },
            subtitle: if is_create {
                "Create a new course"
            } else {
                "Update course details"
            },
            form_error: Some(self.error.as_str()).filter(|e| !e.is_empty()),
            inputs: CourseForm::render_inputs(
                &FormCtx::form::<CourseForm>()
                    .value(CourseFormField::Name, &self.name)
                    .value(CourseFormField::Code, &self.code)
                    .value(CourseFormField::CourseType, &self.course_type)
                    .value(CourseFormField::Fee, &fee)
                    .checked(CourseFormField::IsActive, self.is_active)
                    .value(CourseFormField::Description, &self.description),
            ),
            actions: html! {
                (button_submit(ButtonSubmit { label: "Save Course", ..Default::default() }))
                @if !is_create {
                    (button_modal_form_route(
                        CoursesDeleteGetRouteTag::new(self.id),
                        CoursesDeletePostRouteTag::new(self.id),
                        "Delete",
                        CourseDeleteModalKey::ID,
                        "btn-error",
                    ))
                }
            },
            ..Default::default()
        });
        if is_create {
            modal_keyed::<CourseCreateModalKey>("", body)
        } else {
            modal_keyed::<CourseEditModalKey>("", body)
        }
    }
}

#[derive(Generic)]
pub struct CourseSelectPage {
    pub courses: ObjectList<CourseRow>,
    pub filter_name: String,
    pub filter_code: String,
    pub path_and_query: String,
    pub sort: String,
    pub target_input: String,
    pub is_admin: bool,
}

impl RenderPickerSelect<CourseSelectTableKey, CourseSelectModalKey> for CourseSelectPage {
    fn render_table(&self) -> Markup {
        let headers = [
            TableColumnHeader { key: "Name", label: "Name", sort_url: None, push_url: false },
            TableColumnHeader { key: "Code", label: "Code", sort_url: None, push_url: false },
            TableColumnHeader { key: "Fee", label: "Fee", sort_url: None, push_url: false },
        ];
        let target = if self.target_input.is_empty() {
            "CourseID"
        } else {
            self.target_input.as_str()
        };
        let rows: Vec<TableRow> = self
            .courses
            .items
            .iter()
            .map(|c| {
                let fee = format_inr(c.fee);
                TableRow {
                    attrs: row_attr_select(target, &c.id.to_string(), &c.name),
                    cells: vec![
                        field_text(FieldText { value: &c.name, classes: "" }),
                        field_text(FieldText { value: &c.code, classes: "" }),
                        field_text(FieldText { value: &fee, classes: "" }),
                    ],
                }
            })
            .collect();
        data_table_list_refresh::<CourseSelectTableKey>(
            "Select Course",
            table_button_filter(TableButtonFilter {
                panel: form(FormOpts {
                    attrs: form_hx_get_picker_route::<
                        CourseSelectTableKey,
                        CourseSelectModalKey,
                        CoursesSelectRouteTag,
                    >(CoursesSelectRouteTag)
                        .set("hx-push-url", "false"),
                    inputs: html! {
                        (CourseFilterForm::render_inputs(
                            &FormCtx::form::<CourseFilterForm>()
                                .value(CourseFilterFormField::Name, &self.filter_name)
                                .value(CourseFilterFormField::Code, &self.filter_code),
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
            render_picker_pagination::<CourseSelectModalKey>(
                &self.path_and_query,
                self.courses.number,
                self.courses.num_pages,
            ),
            &self.path_and_query,
        )
    }
}

impl RenderTemplate for CourseSelectPage {
    fn render(&self, _chrome: &ShellChrome) -> Markup {
        self.render_modal().into_inner()
    }
}

#[derive(Generic)]
pub struct CourseMultiSelectPage {
    pub courses: ObjectList<CourseRow>,
    pub filter_name: String,
    pub filter_code: String,
    pub path_and_query: String,
    pub sort: String,
    pub target_input: String,
    pub pool_course_ids: String,
    pub is_admin: bool,
}

impl RenderPickerSelect<CourseMultiSelectTableKey, CourseMultiSelectModalKey>
    for CourseMultiSelectPage
{
    fn render_table(&self) -> Markup {
        let headers = [
            TableColumnHeader { key: "Name", label: "Name", sort_url: None, push_url: false },
            TableColumnHeader { key: "Code", label: "Code", sort_url: None, push_url: false },
            TableColumnHeader { key: "Fee", label: "Fee", sort_url: None, push_url: false },
        ];
        let target = if self.target_input.is_empty() {
            "Courses"
        } else {
            self.target_input.as_str()
        };
        let rows: Vec<TableRow> = self
            .courses
            .items
            .iter()
            .map(|c| {
                let fee = format_inr(c.fee);
                TableRow {
                    attrs: row_attr_select_multi(target, &c.id.to_string(), &c.name),
                    cells: vec![
                        field_text(FieldText { value: &c.name, classes: "" }),
                        field_text(FieldText { value: &c.code, classes: "" }),
                        field_text(FieldText { value: &fee, classes: "" }),
                    ],
                }
            })
            .collect();
        data_table_list_refresh::<CourseMultiSelectTableKey>(
            "Select Courses",
            table_button_filter(TableButtonFilter {
                panel: form(FormOpts {
                    attrs: form_hx_get_picker_route::<
                        CourseMultiSelectTableKey,
                        CourseMultiSelectModalKey,
                        CoursesMultiSelectRouteTag,
                    >(CoursesMultiSelectRouteTag)
                        .set("hx-push-url", "false"),
                    inputs: html! {
                        (CourseFilterForm::render_inputs(
                            &FormCtx::form::<CourseFilterForm>()
                                .value(CourseFilterFormField::Name, &self.filter_name)
                                .value(CourseFilterFormField::Code, &self.filter_code),
                        ))
                        input type="hidden" name="target_input" value=(self.target_input) {}
                        input type="hidden" name="pool_course_ids" value=(self.pool_course_ids) {}
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
            render_picker_pagination::<CourseMultiSelectModalKey>(
                &self.path_and_query,
                self.courses.number,
                self.courses.num_pages,
            ),
            &self.path_and_query,
        )
    }
}

impl RenderTemplate for CourseMultiSelectPage {
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
                    &CoursesDeletePostRouteTag::new(self.id).url(),
                    &target,
                ),
                form_error: Some(self.error.as_str()).filter(|e| !e.is_empty()),
                ..Default::default()
            }),
            ..Default::default()
        })
    }
}
