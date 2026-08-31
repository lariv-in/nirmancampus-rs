use frunk::Generic;
use maud::{Markup, html};

use lariv_rs::{
    components::{
        ButtonClear, ButtonLink, ButtonModalForm, ButtonSubmit, DeleteConfirmation, DetailHeader,
        FieldManyToMany, FieldSubtitle, FieldText, FieldTitle, FormOpts, ManyToManyItem, ObjectList,
        ShellChrome, SlotCapability, SlotRegistrar, SwapKey, TableButtonFilter, TableColumnHeader,
        TableRow, button_clear, button_link, button_modal_form, button_modal_form_route,
        button_submit, column_sort_url, container_column, container_row, data_table_list_grid_with_subtitle,
        data_table_list_refresh, detail, detail_header, field_many_to_many, field_subtitle,
        field_text, field_title, form, form_hx_get_picker_route, form_hx_get_route,
        form_hx_post_selector, form_hx_post_url, hx_nav_app_layout, label, modal, modal_keyed,
        row_attr_select, row_attr_select_multi, sort_indicator, table_button_filter,
        table_create_button,
    },
    html_form::{FormCtx, HtmlForm},
    http::ProvideRequestCaps,
    picker::RenderPickerSelect,
    template::{RenderTemplate, TemplateCapability, TemplateOf, TemplateRegistrar},
    web::{modal_create_post_query, modal_edit_post_url},
};

use crate::{program_detail_crumbs, program_detail_menu, programs_crumbs, programs_menu};
use super::forms::{
    ProgramFilterForm, ProgramFilterFormField, ProgramForm, ProgramFormField, StructureUnitForm,
    StructureUnitFormField,
};
use super::keys::{
    ProgramCreateModalKey, ProgramDeleteModalKey, ProgramEditModalKey,
    ProgramMediaMultiSelectModalKey, ProgramMediaMultiSelectTableKey, ProgramSelectModalKey,
    ProgramSelectTableKey, ProgramTableKey, StructureUnitCreateModalKey,
    StructureUnitDeleteModalKey, StructureUnitEditModalKey,
};
use super::routes::{
    ProgramsCreatePostRouteTag, ProgramsDeleteGetRouteTag,
    ProgramsDeletePostRouteTag, ProgramsDetailRouteTag, ProgramsEditGetRouteTag,
    ProgramsEditPostRouteTag, ProgramsListRouteTag, ProgramsSelectRouteTag,
    ProgramsStructureEditRouteTag, ProgramsStructureUnitCreateGetRouteTag,
    ProgramsStructureUnitCreatePostRouteTag, ProgramsStructureUnitDeleteGetRouteTag,
    ProgramsStructureUnitDeletePostRouteTag, ProgramsStructureUnitEditGetRouteTag,
    ProgramsStructureUnitUpdatePostRouteTag,
};
use nirmancampus_common::{
    admission_session_choice_pairs, format_inr, program_type_choice_pairs, term_type_choice_pairs,
    university_choice_pairs,
    ui::{
        app_scaffold, empty_dash, render_pagination, render_picker_pagination, scaffold_main,
        scaffold_pane, APP_TITLE,
    },
};

lariv_rs::define_register_items! {
    plugin: NirmancampusProgramsTag;
    capability: TemplateCapability;
    trait: TemplateRegistrar;
    method: register_templates;
    wrapper: TemplateOf;
    bounds: [Clone, ProvideRequestCaps, Send, Sync];
    hook: Hook;
    items: [
        ProgramListIdx: ProgramListPageTag => ProgramListPage,
        ProgramDetailIdx: ProgramDetailPageTag => ProgramDetailPage,
        ProgramFormIdx: ProgramFormPageTag => ProgramFormPage,
        ProgramSelectIdx: ProgramSelectPageTag => ProgramSelectPage,
        ProgramMediaMultiSelectIdx: ProgramMediaMultiSelectPageTag => ProgramMediaMultiSelectPage,
        ConfirmDeleteIdx: ProgramConfirmDeletePageTag => ConfirmDeletePage,
        ProgramStructureEditIdx: ProgramStructureEditPageTag => ProgramStructureEditPage,
        StructureUnitFormIdx: StructureUnitFormPageTag => StructureUnitFormPage,
    ]
}

lariv_rs::define_register_items! {
    plugin: NirmancampusProgramsTag;
    capability: SlotCapability;
    trait: SlotRegistrar;
    method: register_slots;
    bounds: [];
    items: [];
    hook: SlotsHook;
}

pub(crate) fn choice_label(pairs: &[(String, String)], key: &str) -> String {
    pairs
        .iter()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v.clone())
        .unwrap_or_else(|| key.to_string())
}

pub(crate) fn program_display_label(name: &str, university: &str) -> String {
    if university.is_empty() {
        name.to_string()
    } else {
        let uni = choice_label(&university_choice_pairs(), university);
        format!("{name} ({uni})")
    }
}

fn media_field_items(items: &[ManyToManyItem]) -> Vec<(&str, Option<&str>)> {
    items.iter().map(|i| (i.value.as_str(), None)).collect()
}

#[derive(Clone)]
pub struct ProgramRow {
    pub id: i64,
    pub name: String,
    pub code: String,
    pub university: String,
    pub program_type: String,
    pub fee: i64,
    pub description: String,
    pub display_label: String,
}

#[derive(Clone)]
pub struct ProgramMediaRow {
    pub id: i64,
    pub language: String,
}

#[derive(Clone)]
pub struct ProgramStructureUnitView {
    pub id: i64,
    pub term_number: i64,
    pub optional_course_count: i64,
    pub compulsory_label: String,
    pub optional_label: String,
    pub compulsory_items: Vec<ManyToManyItem>,
    pub optional_items: Vec<ManyToManyItem>,
}

#[derive(Generic)]
pub struct ProgramListPage {
    pub programs: ObjectList<ProgramRow>,
    pub filter_name: String,
    pub filter_code: String,
    pub filter_university: String,
    pub filter_program_type: String,
    pub path_and_query: String,
    pub sort: String,
    pub is_admin: bool,
}

impl ProgramListPage {
    pub fn render_table(&self) -> Markup {
        let university_choices = university_choice_pairs();
        let program_type_choices = program_type_choice_pairs();
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
                key: "University",
                label: "University",
                sort_url: None,
                push_url: false,
            },
            TableColumnHeader {
                key: "ProgramType",
                label: "Program type",
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
                key: "Description",
                label: "Description",
                sort_url: None,
                push_url: false,
            },
        ];
        let rows: Vec<TableRow> = self
            .programs
            .items
            .iter()
            .map(|p| {
                let fee = format_inr(p.fee);
                TableRow {
                    attrs: hx_nav_app_layout(ProgramsDetailRouteTag::new(p.id)),
                    cells: vec![
                        field_text(FieldText { value: &p.name, classes: "" }),
                        field_text(FieldText { value: &p.code, classes: "" }),
                        field_text(FieldText { value: &p.university, classes: "" }),
                        field_text(FieldText { value: &p.program_type, classes: "" }),
                        field_text(FieldText { value: &fee, classes: "" }),
                        field_text(FieldText { value: &p.description, classes: "" }),
                    ],
                }
            })
            .collect();
        let actions = html! {
            (table_button_filter(TableButtonFilter {
                panel: form(FormOpts {
                    attrs: form_hx_get_route::<ProgramTableKey, ProgramsListRouteTag>(ProgramsListRouteTag),
                    inputs: ProgramFilterForm::render_inputs(
                        &FormCtx::form::<ProgramFilterForm>()
                            .value(ProgramFilterFormField::Name, &self.filter_name)
                            .value(ProgramFilterFormField::Code, &self.filter_code)
                            .value(ProgramFilterFormField::University, &self.filter_university)
                            .value(ProgramFilterFormField::ProgramType, &self.filter_program_type)
                            .choices(ProgramFilterFormField::University, &university_choices)
                            .choices(ProgramFilterFormField::ProgramType, &program_type_choices),
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
                (table_create_button::<ProgramTableKey, ProgramCreateModalKey>(
                    Some("plus"),
                    "btn-square btn-outline btn-sm",
                ))
            }
        };
        data_table_list_grid_with_subtitle::<ProgramTableKey>(
            "Programs",
            "Program catalog",
            actions,
            &headers,
            &rows,
            render_pagination::<ProgramTableKey>(
                &self.path_and_query,
                self.programs.number,
                self.programs.num_pages,
                true,
            ),
        )
    }
}

impl lariv_rs::template::RenderAppPane for ProgramListPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        scaffold_pane(programs_menu(), programs_crumbs("All Programs"), self.render_table())
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        scaffold_main(programs_crumbs("All Programs"), self.render_table())
    }
}

impl RenderTemplate for ProgramListPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold(
            &format!("Programs — {APP_TITLE}"),
            chrome,
            programs_menu(),
            programs_crumbs("All Programs"),
            self.render_table(),
        )
    }
}

#[derive(Generic)]
pub struct ProgramDetailPage {
    pub id: i64,
    pub name: String,
    pub code: String,
    pub university: String,
    pub program_type: String,
    pub admission_sessions: String,
    pub term_type: String,
    pub fee: i64,
    pub description: String,
    pub media_items: Vec<ManyToManyItem>,
    pub units: Vec<ProgramStructureUnitView>,
    pub is_admin: bool,
}

impl ProgramDetailPage {
    fn pane_body(&self) -> Markup {
        let fee = format_inr(self.fee);
        let university = choice_label(&university_choice_pairs(), &self.university);
        let program_type = choice_label(&program_type_choice_pairs(), &self.program_type);
        let admission = choice_label(&admission_session_choice_pairs(), &self.admission_sessions);
        let term_type = choice_label(&term_type_choice_pairs(), &self.term_type);
        let media_pairs = media_field_items(&self.media_items);
        let structure_url = ProgramsStructureEditRouteTag::new(self.id).url();
        detail(html! {
            (container_column("", html! {
                (detail_header(DetailHeader {
                    title: &self.name,
                    actions: html! {
                        @if self.is_admin {
                            (button_modal_form_route(
                                ProgramsEditGetRouteTag::new(self.id),
                                ProgramsEditPostRouteTag::new(self.id),
                                "Edit",
                                ProgramEditModalKey::ID,
                                "btn btn-outline btn-sm",
                            ))
                        }
                    },
                }))
                (field_subtitle(FieldSubtitle { value: empty_dash(&self.code), classes: "" }))
                (label("University", field_text(FieldText { value: empty_dash(&university), classes: "" })))
                (label("Program type", field_text(FieldText { value: empty_dash(&program_type), classes: "" })))
                (label("Admission sessions", field_text(FieldText { value: empty_dash(&admission), classes: "" })))
                (label("Term type", field_text(FieldText { value: empty_dash(&term_type), classes: "" })))
                (label("Fee", field_text(FieldText { value: &fee, classes: "" })))
                (label("Media languages", field_many_to_many(FieldManyToMany {
                    items: &media_pairs,
                    classes: "w-full",
                })))
                (label("Description", field_text(FieldText { value: empty_dash(&self.description), classes: "" })))
                (label("Program structure", html! {
                    @if self.units.is_empty() {
                        @if self.is_admin {
                            (button_link(ButtonLink {
                                label: "Add Program Structure",
                                href: &structure_url,
                                classes: "btn-primary btn-sm w-fit",
                                ..Default::default()
                            }))
                        } @else {
                            (field_text(FieldText { value: "—", classes: "" }))
                        }
                    } @else {
                        div class="flex flex-col gap-2" {
                            @for unit in &self.units {
                                (structure_unit_card(unit, None))
                            }
                        }
                    }
                }))
            }))
        })
    }
}

impl lariv_rs::template::RenderAppPane for ProgramDetailPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        scaffold_pane(
            program_detail_menu(self.id, &self.name, self.is_admin),
            program_detail_crumbs(self.id, &self.name, None),
            self.pane_body(),
        )
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        scaffold_main(program_detail_crumbs(self.id, &self.name, None), self.pane_body())
    }
}

impl RenderTemplate for ProgramDetailPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold(
            &format!("{} — {APP_TITLE}", self.name),
            chrome,
            program_detail_menu(self.id, &self.name, self.is_admin),
            program_detail_crumbs(self.id, &self.name, None),
            self.pane_body(),
        )
    }
}

fn structure_unit_card(unit: &ProgramStructureUnitView, program_id: Option<i64>) -> Markup {
    let term = format!("Term {}", unit.term_number);
    let compulsory = if unit.compulsory_label.is_empty() {
        "—"
    } else {
        unit.compulsory_label.as_str()
    };
    let optional = if unit.optional_label.is_empty() {
        "—"
    } else {
        unit.optional_label.as_str()
    };
    let count = unit.optional_course_count.to_string();
    html! {
        div class="rounded-box border border-base-300 p-2 flex flex-col gap-2 @md:flex-row @md:items-start @md:justify-between" {
            div class="flex flex-col gap-1 min-w-0" {
                div class="font-semibold" { (term) }
                div class="text-sm text-base-content/80" { "Compulsory: " (compulsory) }
                div class="text-sm text-base-content/80" { "Optional count: " (count) }
                div class="text-sm text-base-content/80" { "Optional pool: " (optional) }
            }
            @if let Some(pid) = program_id {
                div class="flex flex-wrap gap-2 shrink-0" {
                    (button_modal_form_route(
                        ProgramsStructureUnitEditGetRouteTag::new(pid, unit.id),
                        ProgramsStructureUnitUpdatePostRouteTag::new(pid, unit.id),
                        "Edit",
                        StructureUnitEditModalKey::ID,
                        "btn-outline btn-sm",
                    ))
                    (button_modal_form_route(
                        ProgramsStructureUnitDeleteGetRouteTag::new(pid, unit.id),
                        ProgramsStructureUnitDeletePostRouteTag::new(pid, unit.id),
                        "Remove",
                        StructureUnitDeleteModalKey::ID,
                        "btn-outline btn-error btn-sm",
                    ))
                }
            }
        }
    }
}

#[derive(Generic)]
pub struct ProgramFormPage {
    pub id: i64,
    pub name: String,
    pub code: String,
    pub description: String,
    pub university: String,
    pub program_type: String,
    pub admission_sessions: String,
    pub term_type: String,
    pub fee: i64,
    pub media_items: Vec<ManyToManyItem>,
    pub error: String,
    pub form_name: String,
    pub refresh_table: String,
    pub target_input: String,
}

impl RenderTemplate for ProgramFormPage {
    fn render(&self, _chrome: &ShellChrome) -> Markup {
        let is_create = self.id == 0;
        let form_attrs = if is_create {
            form_hx_post_url::<ProgramCreateModalKey>(&modal_create_post_query(
                ProgramsCreatePostRouteTag,
                &self.form_name,
                &self.refresh_table,
                &self.target_input,
            ))
        } else {
            form_hx_post_url::<ProgramEditModalKey>(&modal_edit_post_url(
                ProgramsEditPostRouteTag::new(self.id),
                &self.form_name,
            ))
        };
        let fee = self.fee.to_string();
        let university_choices = university_choice_pairs();
        let program_type_choices = program_type_choice_pairs();
        let admission_choices = admission_session_choice_pairs();
        let term_choices = term_type_choice_pairs();
        let body = form(FormOpts {
            attrs: form_attrs,
            title: if is_create { "Create Program" } else { "Edit Program" },
            subtitle: if is_create {
                "Create a new program"
            } else {
                "Update program details"
            },
            form_error: Some(self.error.as_str()).filter(|e| !e.is_empty()),
            inputs: ProgramForm::render_inputs(
                &FormCtx::form::<ProgramForm>()
                    .value(ProgramFormField::Name, &self.name)
                    .value(ProgramFormField::Code, &self.code)
                    .value(ProgramFormField::Description, &self.description)
                    .value(ProgramFormField::University, &self.university)
                    .value(ProgramFormField::ProgramType, &self.program_type)
                    .value(ProgramFormField::AdmissionSessions, &self.admission_sessions)
                    .value(ProgramFormField::TermType, &self.term_type)
                    .value(ProgramFormField::Fee, &fee)
                    .m2m(ProgramFormField::ProgramMedia, &self.media_items)
                    .choices(ProgramFormField::University, &university_choices)
                    .choices(ProgramFormField::ProgramType, &program_type_choices)
                    .choices(ProgramFormField::AdmissionSessions, &admission_choices)
                    .choices(ProgramFormField::TermType, &term_choices),
            ),
            actions: html! {
                (button_submit(ButtonSubmit { label: "Save Program", ..Default::default() }))
                @if !is_create {
                    (button_modal_form_route(
                        ProgramsDeleteGetRouteTag::new(self.id),
                        ProgramsDeletePostRouteTag::new(self.id),
                        "Delete",
                        ProgramDeleteModalKey::ID,
                        "btn-error",
                    ))
                }
            },
            ..Default::default()
        });
        if is_create {
            modal_keyed::<ProgramCreateModalKey>("", body)
        } else {
            modal_keyed::<ProgramEditModalKey>("", body)
        }
    }
}

#[derive(Generic)]
pub struct ProgramSelectPage {
    pub programs: ObjectList<ProgramRow>,
    pub filter_name: String,
    pub filter_code: String,
    pub filter_university: String,
    pub filter_program_type: String,
    pub path_and_query: String,
    pub target_input: String,
}

impl RenderPickerSelect<ProgramSelectTableKey, ProgramSelectModalKey> for ProgramSelectPage {
    fn render_table(&self) -> Markup {
        let university_choices = university_choice_pairs();
        let program_type_choices = program_type_choice_pairs();
        let headers = [
            TableColumnHeader { key: "Name", label: "Name", sort_url: None, push_url: false },
            TableColumnHeader { key: "Code", label: "Code", sort_url: None, push_url: false },
            TableColumnHeader { key: "University", label: "University", sort_url: None, push_url: false },
            TableColumnHeader { key: "ProgramType", label: "Program type", sort_url: None, push_url: false },
            TableColumnHeader { key: "Fee", label: "Fee", sort_url: None, push_url: false },
        ];
        let target = if self.target_input.is_empty() {
            "ProgramID"
        } else {
            self.target_input.as_str()
        };
        let rows: Vec<TableRow> = self
            .programs
            .items
            .iter()
            .map(|p| {
                let fee = format_inr(p.fee);
                TableRow {
                    attrs: row_attr_select(target, &p.id.to_string(), &p.display_label),
                    cells: vec![
                        field_text(FieldText { value: &p.name, classes: "" }),
                        field_text(FieldText { value: &p.code, classes: "" }),
                        field_text(FieldText { value: &p.university, classes: "" }),
                        field_text(FieldText { value: &p.program_type, classes: "" }),
                        field_text(FieldText { value: &fee, classes: "" }),
                    ],
                }
            })
            .collect();
        data_table_list_refresh::<ProgramSelectTableKey>(
            "Select Program",
            table_button_filter(TableButtonFilter {
                panel: form(FormOpts {
                    attrs: form_hx_get_picker_route::<
                        ProgramSelectTableKey,
                        ProgramSelectModalKey,
                        ProgramsSelectRouteTag,
                    >(ProgramsSelectRouteTag)
                        .set("hx-push-url", "false"),
                    inputs: html! {
                        (ProgramFilterForm::render_inputs(
                            &FormCtx::form::<ProgramFilterForm>()
                                .value(ProgramFilterFormField::Name, &self.filter_name)
                                .value(ProgramFilterFormField::Code, &self.filter_code)
                                .value(ProgramFilterFormField::University, &self.filter_university)
                                .value(ProgramFilterFormField::ProgramType, &self.filter_program_type)
                                .choices(ProgramFilterFormField::University, &university_choices)
                                .choices(ProgramFilterFormField::ProgramType, &program_type_choices),
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
            render_picker_pagination::<ProgramSelectModalKey>(
                &self.path_and_query,
                self.programs.number,
                self.programs.num_pages,
            ),
            &self.path_and_query,
        )
    }
}

impl RenderTemplate for ProgramSelectPage {
    fn render(&self, _chrome: &ShellChrome) -> Markup {
        self.render_modal().into_inner()
    }
}

#[derive(Generic)]
pub struct ProgramMediaMultiSelectPage {
    pub items: ObjectList<ProgramMediaRow>,
    pub path_and_query: String,
    pub target_input: String,
}

impl RenderPickerSelect<ProgramMediaMultiSelectTableKey, ProgramMediaMultiSelectModalKey>
    for ProgramMediaMultiSelectPage
{
    fn render_table(&self) -> Markup {
        let headers = [TableColumnHeader {
            key: "Language",
            label: "Language",
            sort_url: None,
            push_url: false,
        }];
        let target = if self.target_input.is_empty() {
            "ProgramMedia"
        } else {
            self.target_input.as_str()
        };
        let rows: Vec<TableRow> = self
            .items
            .items
            .iter()
            .map(|m| TableRow {
                attrs: row_attr_select_multi(target, &m.id.to_string(), &m.language),
                cells: vec![field_text(FieldText {
                    value: &m.language,
                    classes: "",
                })],
            })
            .collect();
        data_table_list_refresh::<ProgramMediaMultiSelectTableKey>(
            "Select languages",
            html! {},
            &headers,
            &rows,
            render_picker_pagination::<ProgramMediaMultiSelectModalKey>(
                &self.path_and_query,
                self.items.number,
                self.items.num_pages,
            ),
            &self.path_and_query,
        )
    }
}

impl RenderTemplate for ProgramMediaMultiSelectPage {
    fn render(&self, _chrome: &ShellChrome) -> Markup {
        self.render_modal().into_inner()
    }
}

#[derive(Generic)]
pub struct ConfirmDeletePage {
    pub modal_uid: String,
    pub message: String,
    pub id: i64,
    pub unit_id: i64,
    pub error: String,
    pub is_unit: bool,
}

impl RenderTemplate for ConfirmDeletePage {
    fn render(&self, _chrome: &ShellChrome) -> Markup {
        let target = format!("#{}", self.modal_uid);
        let post_url = if self.is_unit {
            ProgramsStructureUnitDeletePostRouteTag::new(self.id, self.unit_id).url()
        } else {
            ProgramsDeletePostRouteTag::new(self.id).url()
        };
        let title = if self.is_unit {
            "Remove structure unit"
        } else {
            "Confirm Deletion"
        };
        modal(lariv_rs::components::Modal {
            uid: &self.modal_uid,
            children: lariv_rs::components::delete_confirmation(DeleteConfirmation {
                title,
                message: &self.message,
                attrs: form_hx_post_selector(&post_url, &target),
                form_error: Some(self.error.as_str()).filter(|e| !e.is_empty()),
                ..Default::default()
            }),
            ..Default::default()
        })
    }
}

#[derive(Generic)]
pub struct ProgramStructureEditPage {
    pub id: i64,
    pub name: String,
    pub units: Vec<ProgramStructureUnitView>,
    pub is_admin: bool,
}

impl ProgramStructureEditPage {
    fn pane_body(&self) -> Markup {
        let create_get = ProgramsStructureUnitCreateGetRouteTag::new(self.id).url();
        let create_post = ProgramsStructureUnitCreatePostRouteTag::new(self.id).path();
        html! {
            div class="max-w-3xl flex flex-col" {
                (field_title(FieldTitle { value: "Edit program structure", classes: "" }))
                (field_subtitle(FieldSubtitle { value: &self.name, classes: "" }))
                @if self.units.is_empty() {
                    p class="text-base-content/70" {
                        "No structure units yet. Use “Add new unit” to create one."
                    }
                } @else {
                    div class="flex flex-col gap-2 my-4" {
                        @for unit in &self.units {
                            (structure_unit_card(unit, Some(self.id)))
                        }
                    }
                }
                (button_modal_form(ButtonModalForm {
                    label: "Add new unit",
                    href: &create_get,
                    form_post_url: &create_post,
                    modal_uid: StructureUnitCreateModalKey::ID,
                    classes: "btn-primary",
                    ..Default::default()
                }))
            }
        }
    }
}

impl lariv_rs::template::RenderAppPane for ProgramStructureEditPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        scaffold_pane(
            program_detail_menu(self.id, &self.name, self.is_admin),
            program_detail_crumbs(self.id, &self.name, Some("Program structure")),
            self.pane_body(),
        )
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        scaffold_main(
            program_detail_crumbs(self.id, &self.name, Some("Program structure")),
            self.pane_body(),
        )
    }
}

impl RenderTemplate for ProgramStructureEditPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold(
            &format!("Edit program structure — {APP_TITLE}"),
            chrome,
            program_detail_menu(self.id, &self.name, self.is_admin),
            program_detail_crumbs(self.id, &self.name, Some("Program structure")),
            self.pane_body(),
        )
    }
}

#[derive(Generic)]
pub struct StructureUnitFormPage {
    pub program_id: i64,
    pub unit_id: i64,
    pub term_number: i64,
    pub optional_course_count: i64,
    pub compulsory_items: Vec<ManyToManyItem>,
    pub optional_items: Vec<ManyToManyItem>,
    pub error: String,
    pub form_name: String,
}

impl RenderTemplate for StructureUnitFormPage {
    fn render(&self, _chrome: &ShellChrome) -> Markup {
        let is_create = self.unit_id == 0;
        let term = self.term_number.to_string();
        let count = self.optional_course_count.to_string();
        let form_attrs = if is_create {
            form_hx_post_url::<StructureUnitCreateModalKey>(
                &ProgramsStructureUnitCreatePostRouteTag::new(self.program_id).url(),
            )
        } else {
            form_hx_post_url::<StructureUnitEditModalKey>(&modal_edit_post_url(
                ProgramsStructureUnitUpdatePostRouteTag::new(self.program_id, self.unit_id),
                &self.form_name,
            ))
        };
        let body = form(FormOpts {
            attrs: form_attrs,
            title: if is_create {
                "Add structure unit"
            } else {
                "Edit structure unit"
            },
            form_error: Some(self.error.as_str()).filter(|e| !e.is_empty()),
            inputs: StructureUnitForm::render_inputs(
                &FormCtx::form::<StructureUnitForm>()
                    .value(StructureUnitFormField::TermNumber, &term)
                    .value(StructureUnitFormField::OptionalCourseCount, &count)
                    .m2m(StructureUnitFormField::CompulsoryCourses, &self.compulsory_items)
                    .m2m(
                        StructureUnitFormField::OptionalCourseSelectionPool,
                        &self.optional_items,
                    ),
            ),
            actions: html! {
                (button_submit(ButtonSubmit {
                    label: if is_create { "Save unit" } else { "Save changes" },
                    classes: "btn-primary",
                    ..Default::default()
                }))
                @if !is_create {
                    (button_modal_form_route(
                        ProgramsStructureUnitDeleteGetRouteTag::new(self.program_id, self.unit_id),
                        ProgramsStructureUnitDeletePostRouteTag::new(self.program_id, self.unit_id),
                        "Remove unit",
                        StructureUnitDeleteModalKey::ID,
                        "btn-error",
                    ))
                }
            },
            ..Default::default()
        });
        if is_create {
            modal_keyed::<StructureUnitCreateModalKey>("", body)
        } else {
            modal_keyed::<StructureUnitEditModalKey>("", body)
        }
    }
}
