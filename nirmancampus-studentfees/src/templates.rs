use frunk::Generic;
use maud::{Markup, html};

use lariv_rs::{
    components::{
        ButtonClear, ButtonSubmit, DeleteConfirmation, DetailHeader, FieldText, FormOpts,
        ObjectList, ShellChrome, SlotCapability, SlotRegistrar, SwapKey, TableButtonFilter,
        TableColumnHeader, TableRow, button_clear, button_modal_form_route, button_submit,
        container_column, container_row, data_table_list_grid_with_subtitle, detail, detail_header,
        field_text, form, form_hx_get_route, form_hx_post_main, form_hx_post_selector,
        form_hx_post_url, hx_nav_app_layout, label, modal, modal_keyed, table_button_filter,
        table_create_button,
    },
    html_form::{FormCtx, HtmlForm},
    http::ProvideRequestCaps,
    template::{RenderTemplate, TemplateCapability, TemplateOf, TemplateRegistrar},
    web::{modal_create_post_query, modal_edit_post_url},
};

use crate::forms::{
    FeeFilterForm, FeeFilterFormField, FeeForm, FeeFormField, FeeUploadForm, PreferencesForm,
    PreferencesFormField,
};
use crate::keys::{FeeCreateModalKey, FeeDeleteModalKey, FeeEditModalKey, FeeTableKey};
use crate::menus::{fee_detail_crumbs, fee_detail_menu, fees_crumbs, fees_menu, prefs_crumbs};
use crate::parse::{flag_label, opt_str};
use crate::routes::{
    StudentFeesCreatePostRouteTag, StudentFeesDeleteGetRouteTag, StudentFeesDeletePostRouteTag,
    StudentFeesDetailRouteTag, StudentFeesEditGetRouteTag, StudentFeesEditPostRouteTag,
    StudentFeesListRouteTag, StudentFeesPrefsPostRouteTag, StudentFeesSyncRouteTag,
};
use nirmancampus_common::ui::{
    APP_TITLE, app_scaffold, empty_dash, render_pagination, scaffold_main, scaffold_pane,
};

lariv_rs::define_register_items! {
    plugin: NirmancampusStudentFeesTag;
    capability: TemplateCapability;
    trait: TemplateRegistrar;
    method: register_templates;
    wrapper: TemplateOf;
    bounds: [Clone, ProvideRequestCaps, Send, Sync];
    hook: Hook;
    items: [
        FeeListIdx: FeeListPageTag => FeeListPage,
        FeeDetailIdx: FeeDetailPageTag => FeeDetailPage,
        FeeFormIdx: FeeFormPageTag => FeeFormPage,
        PrefsIdx: FeePreferencesPageTag => FeePreferencesPage,
        ConfirmDeleteIdx: FeeConfirmDeletePageTag => ConfirmDeletePage,
    ]
}

lariv_rs::define_register_items! {
    plugin: NirmancampusStudentFeesTag;
    capability: SlotCapability;
    trait: SlotRegistrar;
    method: register_slots;
    bounds: [];
    items: [];
    hook: SlotsHook;
}

#[derive(Clone)]
pub struct FeeRow {
    pub id: i64,
    pub session_with_year: String,
    pub submit: String,
    pub student: String,
    pub enroll: String,
    pub prog: String,
    pub contact: String,
    pub dob: String,
    pub category: String,
    pub father_name: String,
    pub courses: String,
    pub dod: String,
    pub deposit: String,
    pub university: String,
    pub remarks: String,
}

impl FeeRow {
    pub fn from_model(row: &crate::entities::fee::Model) -> Self {
        Self {
            id: i64::from(row.id),
            session_with_year: row.session_with_year(),
            submit: opt_str(&row.submit).to_string(),
            student: opt_str(&row.student).to_string(),
            enroll: opt_str(&row.enroll).to_string(),
            prog: opt_str(&row.prog).to_string(),
            contact: opt_str(&row.contact).to_string(),
            dob: opt_str(&row.dob).to_string(),
            category: opt_str(&row.category).to_string(),
            father_name: opt_str(&row.father_name).to_string(),
            courses: opt_str(&row.courses).to_string(),
            dod: row.dod_display(),
            deposit: opt_str(&row.deposit).to_string(),
            university: opt_str(&row.university).to_string(),
            remarks: opt_str(&row.remarks).to_string(),
        }
    }
}

#[derive(Generic)]
pub struct FeeListPage {
    pub records: ObjectList<FeeRow>,
    pub filter_search: String,
    pub path_and_query: String,
    pub is_admin: bool,
    pub connection_error: String,
    pub sync_message: String,
    pub sync_error: String,
}

impl FeeListPage {
    pub fn render_table(&self) -> Markup {
        let headers = [
            TableColumnHeader { key: "Id", label: "Receipt ID", sort_url: None, push_url: false },
            TableColumnHeader { key: "Session", label: "Session", sort_url: None, push_url: false },
            TableColumnHeader { key: "Student", label: "Name", sort_url: None, push_url: false },
            TableColumnHeader { key: "Enroll", label: "Enrollment", sort_url: None, push_url: false },
            TableColumnHeader { key: "Prog", label: "Program", sort_url: None, push_url: false },
            TableColumnHeader { key: "Contact", label: "Mobile", sort_url: None, push_url: false },
            TableColumnHeader { key: "Dob", label: "DOB", sort_url: None, push_url: false },
            TableColumnHeader { key: "Category", label: "Category", sort_url: None, push_url: false },
            TableColumnHeader { key: "Father", label: "Father", sort_url: None, push_url: false },
            TableColumnHeader { key: "Courses", label: "Courses", sort_url: None, push_url: false },
            TableColumnHeader { key: "Dod", label: "Date of Deposit", sort_url: None, push_url: false },
            TableColumnHeader { key: "Submit", label: "Submit type", sort_url: None, push_url: false },
            TableColumnHeader { key: "Deposit", label: "Deposit", sort_url: None, push_url: false },
            TableColumnHeader { key: "University", label: "University", sort_url: None, push_url: false },
            TableColumnHeader { key: "Remarks", label: "Remarks", sort_url: None, push_url: false },
        ];
        let rows: Vec<TableRow> = self
            .records
            .items
            .iter()
            .map(|r| {
                let id = r.id.to_string();
                TableRow {
                    attrs: hx_nav_app_layout(StudentFeesDetailRouteTag::new(r.id)),
                    cells: vec![
                        field_text(FieldText { value: &id, classes: "" }),
                        field_text(FieldText { value: empty_dash(&r.session_with_year), classes: "" }),
                        field_text(FieldText { value: empty_dash(&r.student), classes: "" }),
                        field_text(FieldText { value: empty_dash(&r.enroll), classes: "" }),
                        field_text(FieldText { value: empty_dash(&r.prog), classes: "" }),
                        field_text(FieldText { value: empty_dash(&r.contact), classes: "" }),
                        field_text(FieldText { value: empty_dash(&r.dob), classes: "" }),
                        field_text(FieldText { value: empty_dash(&r.category), classes: "" }),
                        field_text(FieldText { value: empty_dash(&r.father_name), classes: "" }),
                        field_text(FieldText { value: empty_dash(&r.courses), classes: "" }),
                        field_text(FieldText { value: empty_dash(&r.dod), classes: "" }),
                        field_text(FieldText { value: empty_dash(&r.submit), classes: "" }),
                        field_text(FieldText { value: empty_dash(&r.deposit), classes: "" }),
                        field_text(FieldText { value: empty_dash(&r.university), classes: "" }),
                        field_text(FieldText { value: empty_dash(&r.remarks), classes: "" }),
                    ],
                }
            })
            .collect();
        let actions = html! {
            (table_button_filter(TableButtonFilter {
                panel: form(FormOpts {
                    attrs: form_hx_get_route::<FeeTableKey, StudentFeesListRouteTag>(
                        StudentFeesListRouteTag,
                    ),
                    inputs: FeeFilterForm::render_inputs(
                        &FormCtx::form::<FeeFilterForm>()
                            .value(FeeFilterFormField::Search, &self.filter_search),
                    ),
                    actions: html! {
                        (container_row("flex gap-2", html! {
                            (button_submit(ButtonSubmit { label: "Search", ..Default::default() }))
                            (button_clear(ButtonClear { label: "Clear", ..Default::default() }))
                        }))
                    },
                    ..Default::default()
                }),
                ..Default::default()
            }))
            @if self.is_admin {
                (table_create_button::<FeeTableKey, FeeCreateModalKey>(
                    Some("plus"),
                    "btn-square btn-outline btn-sm",
                ))
            }
        };
        data_table_list_grid_with_subtitle::<FeeTableKey>(
            "Fee records",
            "Live rows from MySQL tblfee. Upload tblfee.xlsx to insert or update by Receipt ID.",
            actions,
            &headers,
            &rows,
            render_pagination::<FeeTableKey>(
                &self.path_and_query,
                self.records.number,
                self.records.num_pages,
                true,
            ),
        )
    }

    fn pane_body(&self) -> Markup {
        html! {
            (container_column("max-w-full", html! {
                @if !self.connection_error.is_empty() {
                    p class="text-error mb-2" {
                        (self.connection_error)
                        " "
                        a href="/student-fees/preferences/" class="link" { "Open Preferences" }
                    }
                }
                @if !self.sync_message.is_empty() {
                    p class="text-success mb-2" { (self.sync_message) }
                }
                @if self.is_admin {
                    (form(FormOpts {
                        attrs: form_hx_post_main(StudentFeesSyncRouteTag)
                            .set("hx-encoding", "multipart/form-data"),
                        enctype: Some("multipart/form-data"),
                        form_error: if self.sync_error.is_empty() {
                            None
                        } else {
                            Some(self.sync_error.as_str())
                        },
                        inputs: FeeUploadForm::render_inputs(&FormCtx::form::<FeeUploadForm>()),
                        actions: html! {
                            (button_submit(ButtonSubmit { label: "Upload and import", ..Default::default() }))
                        },
                        ..Default::default()
                    }))
                }
                (self.render_table())
            }))
        }
    }
}

impl lariv_rs::template::RenderAppPane for FeeListPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        scaffold_pane(fees_menu(), fees_crumbs("Fee records"), self.pane_body())
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        scaffold_main(fees_crumbs("Fee records"), self.pane_body())
    }
}

impl RenderTemplate for FeeListPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold(
            &format!("Student Fees — {APP_TITLE}"),
            chrome,
            fees_menu(),
            fees_crumbs("Fee records"),
            self.pane_body(),
        )
    }
}

#[derive(Generic)]
pub struct FeeDetailPage {
    pub id: i64,
    pub session_with_year: String,
    pub submit: String,
    pub student: String,
    pub enroll: String,
    pub prog: String,
    pub year_sem: String,
    pub contact: String,
    pub dob: String,
    pub category: String,
    pub mother_name: String,
    pub father_name: String,
    pub courses: String,
    pub dod: String,
    pub deposit: String,
    pub nsd: String,
    pub fee: String,
    pub deposit_by: String,
    pub ts: String,
    pub medium: String,
    pub username: String,
    pub control_id: String,
    pub descrepency: String,
    pub university: String,
    pub payment_mode: String,
    pub trans_id: String,
    pub bank: String,
    pub rm: String,
    pub is_reconciled: String,
    pub online_exported: String,
    pub is_admin: bool,
}

impl FeeDetailPage {
    pub fn from_model(row: crate::entities::fee::Model, is_admin: bool) -> Self {
        Self {
            id: i64::from(row.id),
            session_with_year: row.session_with_year(),
            submit: opt_str(&row.submit).to_string(),
            student: opt_str(&row.student).to_string(),
            enroll: opt_str(&row.enroll).to_string(),
            prog: opt_str(&row.prog).to_string(),
            year_sem: opt_str(&row.year_sem).to_string(),
            contact: opt_str(&row.contact).to_string(),
            dob: opt_str(&row.dob).to_string(),
            category: opt_str(&row.category).to_string(),
            mother_name: opt_str(&row.mother_name).to_string(),
            father_name: opt_str(&row.father_name).to_string(),
            courses: opt_str(&row.courses).to_string(),
            dod: row.dod_display(),
            deposit: opt_str(&row.deposit).to_string(),
            nsd: opt_str(&row.nsd).to_string(),
            fee: opt_str(&row.fee).to_string(),
            deposit_by: opt_str(&row.deposit_by).to_string(),
            ts: opt_str(&row.ts).to_string(),
            medium: opt_str(&row.medium).to_string(),
            username: opt_str(&row.username).to_string(),
            control_id: opt_str(&row.control_id).to_string(),
            descrepency: opt_str(&row.descrepency).to_string(),
            university: opt_str(&row.university).to_string(),
            payment_mode: opt_str(&row.payment_mode).to_string(),
            trans_id: opt_str(&row.trans_id).to_string(),
            bank: opt_str(&row.bank).to_string(),
            rm: opt_str(&row.rm).to_string(),
            is_reconciled: flag_label(row.is_reconciled).to_string(),
            online_exported: flag_label(row.online_exported).to_string(),
            is_admin,
        }
    }

    fn crumb_label(&self) -> String {
        if self.student.trim().is_empty() {
            format!("Receipt {}", self.id)
        } else {
            self.student.clone()
        }
    }

    fn pane_body(&self) -> Markup {
        let id = self.id.to_string();
        let title = if self.student.trim().is_empty() {
            format!("Receipt {id}")
        } else {
            format!("{} · {id}", self.student)
        };
        detail(html! {
            (container_column("", html! {
                (detail_header(DetailHeader {
                    title: &title,
                    actions: html! {
                        @if self.is_admin {
                            (button_modal_form_route(
                                StudentFeesEditGetRouteTag::new(self.id),
                                StudentFeesEditPostRouteTag::new(self.id),
                                "Edit",
                                FeeEditModalKey::ID,
                                "btn btn-outline btn-sm",
                            ))
                        }
                    },
                }))
                (label("Receipt ID", field_text(FieldText { value: &id, classes: "" })))
                (label("Session", field_text(FieldText { value: empty_dash(&self.session_with_year), classes: "" })))
                (label("Name", field_text(FieldText { value: empty_dash(&self.student), classes: "" })))
                (label("Enrollment", field_text(FieldText { value: empty_dash(&self.enroll), classes: "" })))
                (label("Program", field_text(FieldText { value: empty_dash(&self.prog), classes: "" })))
                (label("Year/Sem", field_text(FieldText { value: empty_dash(&self.year_sem), classes: "" })))
                (label("Mobile", field_text(FieldText { value: empty_dash(&self.contact), classes: "" })))
                (label("DOB", field_text(FieldText { value: empty_dash(&self.dob), classes: "" })))
                (label("Category", field_text(FieldText { value: empty_dash(&self.category), classes: "" })))
                (label("Mother", field_text(FieldText { value: empty_dash(&self.mother_name), classes: "" })))
                (label("Father", field_text(FieldText { value: empty_dash(&self.father_name), classes: "" })))
                (label("Courses", field_text(FieldText { value: empty_dash(&self.courses), classes: "" })))
                (label("Date of Deposit", field_text(FieldText { value: empty_dash(&self.dod), classes: "" })))
                (label("Submit type", field_text(FieldText { value: empty_dash(&self.submit), classes: "" })))
                (label("Deposit", field_text(FieldText { value: empty_dash(&self.deposit), classes: "" })))
                (label("NSD", field_text(FieldText { value: empty_dash(&self.nsd), classes: "" })))
                (label("Fee", field_text(FieldText { value: empty_dash(&self.fee), classes: "" })))
                (label("Deposit by", field_text(FieldText { value: empty_dash(&self.deposit_by), classes: "" })))
                (label("TS", field_text(FieldText { value: empty_dash(&self.ts), classes: "" })))
                (label("Medium", field_text(FieldText { value: empty_dash(&self.medium), classes: "" })))
                (label("Username", field_text(FieldText { value: empty_dash(&self.username), classes: "" })))
                (label("Control ID", field_text(FieldText { value: empty_dash(&self.control_id), classes: "" })))
                (label("Discrepancy", field_text(FieldText { value: empty_dash(&self.descrepency), classes: "" })))
                (label("University", field_text(FieldText { value: empty_dash(&self.university), classes: "" })))
                (label("Payment mode", field_text(FieldText { value: empty_dash(&self.payment_mode), classes: "" })))
                (label("Trans ID", field_text(FieldText { value: empty_dash(&self.trans_id), classes: "" })))
                (label("Bank", field_text(FieldText { value: empty_dash(&self.bank), classes: "" })))
                (label("RM", field_text(FieldText { value: empty_dash(&self.rm), classes: "" })))
                (label("Reconciled", field_text(FieldText { value: empty_dash(&self.is_reconciled), classes: "" })))
                (label("Online exported", field_text(FieldText { value: empty_dash(&self.online_exported), classes: "" })))
            }))
        })
    }
}

impl lariv_rs::template::RenderAppPane for FeeDetailPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        scaffold_pane(
            fee_detail_menu(self.id, &self.crumb_label(), self.is_admin),
            fee_detail_crumbs(self.id, &self.crumb_label(), None),
            self.pane_body(),
        )
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        scaffold_main(
            fee_detail_crumbs(self.id, &self.crumb_label(), None),
            self.pane_body(),
        )
    }
}

impl RenderTemplate for FeeDetailPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        let title = if self.student.trim().is_empty() {
            format!("Receipt {} — {APP_TITLE}", self.id)
        } else {
            format!("{} — {APP_TITLE}", self.student)
        };
        app_scaffold(
            &title,
            chrome,
            fee_detail_menu(self.id, &self.crumb_label(), self.is_admin),
            fee_detail_crumbs(self.id, &self.crumb_label(), None),
            self.pane_body(),
        )
    }
}

#[derive(Generic)]
pub struct FeeFormPage {
    pub id: i64,
    pub form: FeeForm,
    pub error: String,
    pub form_name: String,
    pub refresh_table: String,
    pub target_input: String,
}

impl RenderTemplate for FeeFormPage {
    fn render(&self, _chrome: &ShellChrome) -> Markup {
        let is_create = self.id == 0;
        let form_attrs = if is_create {
            form_hx_post_url::<FeeCreateModalKey>(&modal_create_post_query(
                StudentFeesCreatePostRouteTag,
                &self.form_name,
                &self.refresh_table,
                &self.target_input,
            ))
        } else {
            form_hx_post_url::<FeeEditModalKey>(&modal_edit_post_url(
                StudentFeesEditPostRouteTag::new(self.id),
                &self.form_name,
            ))
        };
        let id = if self.form.id > 0 {
            self.form.id.to_string()
        } else {
            String::new()
        };
        let body = form(FormOpts {
            attrs: form_attrs,
            title: if is_create { "Create fee record" } else { "Edit fee record" },
            subtitle: if is_create {
                "Receipt ID is required and must be unique."
            } else {
                "Update this tblfee row."
            },
            form_error: Some(self.error.as_str()).filter(|e| !e.is_empty()),
            inputs: FeeForm::render_inputs(
                &FormCtx::form::<FeeForm>()
                    .value(FeeFormField::Id, &id)
                    .value(FeeFormField::AdmSession, &self.form.adm_session)
                    .value(FeeFormField::AdmYear, &self.form.adm_year)
                    .value(FeeFormField::Dod, &self.form.dod)
                    .value(FeeFormField::Submit, &self.form.submit)
                    .value(FeeFormField::Prog, &self.form.prog)
                    .value(FeeFormField::Enroll, &self.form.enroll)
                    .value(FeeFormField::Student, &self.form.student)
                    .value(FeeFormField::YearSem, &self.form.year_sem)
                    .value(FeeFormField::Category, &self.form.category)
                    .value(FeeFormField::Dob, &self.form.dob)
                    .value(FeeFormField::Contact, &self.form.contact)
                    .value(FeeFormField::Deposit, &self.form.deposit)
                    .value(FeeFormField::Nsd, &self.form.nsd)
                    .value(FeeFormField::Fee, &self.form.fee)
                    .value(FeeFormField::Courses, &self.form.courses)
                    .value(FeeFormField::Remarks, &self.form.remarks)
                    .value(FeeFormField::DepositBy, &self.form.deposit_by)
                    .value(FeeFormField::Ts, &self.form.ts)
                    .value(FeeFormField::Medium, &self.form.medium)
                    .value(FeeFormField::MotherName, &self.form.mother_name)
                    .value(FeeFormField::FatherName, &self.form.father_name)
                    .value(FeeFormField::Username, &self.form.username)
                    .value(FeeFormField::ControlId, &self.form.control_id)
                    .value(FeeFormField::Descrepency, &self.form.descrepency)
                    .value(FeeFormField::University, &self.form.university)
                    .value(FeeFormField::PaymentMode, &self.form.payment_mode)
                    .value(FeeFormField::TransId, &self.form.trans_id)
                    .value(FeeFormField::Bank, &self.form.bank)
                    .value(FeeFormField::Rm, &self.form.rm)
                    .checked(FeeFormField::IsReconciled, self.form.is_reconciled)
                    .checked(FeeFormField::OnlineExported, self.form.online_exported),
            ),
            actions: html! {
                (button_submit(ButtonSubmit { label: "Save fee", ..Default::default() }))
                @if !is_create {
                    (button_modal_form_route(
                        StudentFeesDeleteGetRouteTag::new(self.id),
                        StudentFeesDeletePostRouteTag::new(self.id),
                        "Delete",
                        FeeDeleteModalKey::ID,
                        "btn-error",
                    ))
                }
            },
            ..Default::default()
        });
        if is_create {
            modal_keyed::<FeeCreateModalKey>("", body)
        } else {
            modal_keyed::<FeeEditModalKey>("", body)
        }
    }
}

#[derive(Generic)]
pub struct FeePreferencesPage {
    pub host: String,
    pub port: String,
    pub username: String,
    pub password: String,
    pub database: String,
    pub error: String,
    pub message: String,
}

impl FeePreferencesPage {
    fn pane_body(&self) -> Markup {
        html! {
            (container_column("max-w-xl", html! {
                @if !self.message.is_empty() {
                    p class="text-success mb-2" { (self.message) }
                }
                (form(FormOpts {
                    attrs: form_hx_post_main(StudentFeesPrefsPostRouteTag),
                    title: "MySQL connection",
                    subtitle: "Saved in the app database. Used to read and write tblfee. SSL is always disabled.",
                    form_error: Some(self.error.as_str()).filter(|e| !e.is_empty()),
                    inputs: PreferencesForm::render_inputs(
                        &FormCtx::form::<PreferencesForm>()
                            .value(PreferencesFormField::Host, &self.host)
                            .value(PreferencesFormField::Port, &self.port)
                            .value(PreferencesFormField::Username, &self.username)
                            .value(PreferencesFormField::Password, &self.password)
                            .value(PreferencesFormField::Database, &self.database),
                    ),
                    actions: html! {
                        (button_submit(ButtonSubmit { label: "Save and connect", ..Default::default() }))
                    },
                    ..Default::default()
                }))
            }))
        }
    }
}

impl lariv_rs::template::RenderAppPane for FeePreferencesPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        scaffold_pane(fees_menu(), prefs_crumbs(), self.pane_body())
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        scaffold_main(prefs_crumbs(), self.pane_body())
    }
}

impl RenderTemplate for FeePreferencesPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold(
            &format!("Student Fees preferences — {APP_TITLE}"),
            chrome,
            fees_menu(),
            prefs_crumbs(),
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
                    &StudentFeesDeletePostRouteTag::new(self.id).url(),
                    &target,
                ),
                form_error: Some(self.error.as_str()).filter(|e| !e.is_empty()),
                ..Default::default()
            }),
            ..Default::default()
        })
    }
}
