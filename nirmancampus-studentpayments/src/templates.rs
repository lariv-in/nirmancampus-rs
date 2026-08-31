use frunk::Generic;
use maud::{Markup, html};

use lariv_rs::{
    components::{
        ButtonClear, ButtonSubmit, DeleteConfirmation, DetailHeader, FieldText, FormOpts,
        ObjectList, ShellChrome, SlotCapability, SlotRegistrar, SwapKey, TableButtonFilter,
        TableColumnHeader, TableRow, button_clear, button_download_route, button_modal_form_route,
        button_submit, column_sort_url, container_column, container_row,
        data_table_list_grid_with_subtitle, data_table_list_refresh, detail, detail_header,
        field_text, form, form_hx_get_picker_route, form_hx_get_route, form_hx_post_selector,
        form_hx_post_url, hx_nav_app_layout, label, modal, modal_keyed, row_attr_select,
        sort_indicator, table_button_filter, table_create_button,
    },
    html_form::{FormCtx, HtmlForm},
    http::ProvideRequestCaps,
    picker::RenderPickerSelect,
    template::{RenderTemplate, TemplateCapability, TemplateOf, TemplateRegistrar},
    web::{modal_create_post_query, modal_edit_post_url},
};

use crate::{payment_detail_crumbs, payment_detail_menu, payments_crumbs};
use super::forms::{PaymentFilterForm, PaymentFilterFormField, PaymentForm, PaymentFormField};
use super::keys::{
    PaymentCreateModalKey, PaymentDeleteModalKey, PaymentEditModalKey, PaymentSelectModalKey,
    PaymentSelectTableKey, PaymentTableKey,
};
use super::routes::{
    StudentPaymentsCreatePostRouteTag, StudentPaymentsDeleteGetRouteTag,
    StudentPaymentsDeletePostRouteTag, StudentPaymentsDetailRouteTag, StudentPaymentsEditGetRouteTag,
    StudentPaymentsEditPostRouteTag, StudentPaymentsListRouteTag, StudentPaymentsReceiptRouteTag,
    StudentPaymentsSelectRouteTag,
};
use nirmancampus_common::{
    payment_method_choice_pairs,
    ui::{
        app_scaffold, empty_dash, field_related, render_pagination, render_picker_pagination,
        scaffold_main, scaffold_pane, students_hub_menu, APP_TITLE,
    },
};
use nirmancampus_students::routes::StudentsDetailRouteTag;

lariv_rs::define_register_items! {
    plugin: NirmancampusStudentPaymentsTag;
    capability: TemplateCapability;
    trait: TemplateRegistrar;
    method: register_templates;
    wrapper: TemplateOf;
    bounds: [Clone, ProvideRequestCaps, Send, Sync];
    hook: Hook;
    items: [
        PaymentListIdx: PaymentListPageTag => PaymentListPage,
        PaymentDetailIdx: PaymentDetailPageTag => PaymentDetailPage,
        PaymentFormIdx: PaymentFormPageTag => PaymentFormPage,
        PaymentSelectIdx: PaymentSelectPageTag => PaymentSelectPage,
        ConfirmDeleteIdx: PaymentConfirmDeletePageTag => ConfirmDeletePage,
    ]
}

lariv_rs::define_register_items! {
    plugin: NirmancampusStudentPaymentsTag;
    capability: SlotCapability;
    trait: SlotRegistrar;
    method: register_slots;
    bounds: [];
    items: [];
    hook: SlotsHook;
}

#[derive(Clone)]
pub struct PaymentRow {
    pub id: i64,
    pub student_id: i64,
    pub student_display: String,
    pub amount: String,
    pub payment_method: String,
    pub transaction_id: String,
    pub paid_at: String,
}

#[derive(Generic)]
pub struct PaymentListPage {
    pub payments: ObjectList<PaymentRow>,
    pub filter_payment_method: String,
    pub filter_transaction_id: String,
    pub filter_student_id: i64,
    pub filter_student_display: String,
    pub path_and_query: String,
    pub sort: String,
    pub is_admin: bool,
}

impl PaymentListPage {
    pub fn render_table(&self) -> Markup {
        let amount_sort = column_sort_url(&self.path_and_query, "Amount", &self.sort);
        let amount_label = format!("Amount{}", sort_indicator(&self.sort, "Amount"));
        let headers = [
            TableColumnHeader { key: "Student", label: "Student", sort_url: None, push_url: false },
            TableColumnHeader { key: "Amount", label: &amount_label, sort_url: Some(&amount_sort), push_url: true },
            TableColumnHeader { key: "Method", label: "Method", sort_url: None, push_url: false },
            TableColumnHeader { key: "PaidAt", label: "Paid on", sort_url: None, push_url: false },
            TableColumnHeader { key: "TransactionID", label: "Transaction ID", sort_url: None, push_url: false },
        ];
        let rows: Vec<TableRow> = self
            .payments
            .items
            .iter()
            .map(|p| TableRow {
                attrs: hx_nav_app_layout(StudentPaymentsDetailRouteTag::new(p.id)),
                cells: vec![
                    field_text(FieldText { value: &p.student_display, classes: "" }),
                    field_text(FieldText { value: &p.amount, classes: "" }),
                    field_text(FieldText { value: &p.payment_method, classes: "" }),
                    field_text(FieldText { value: empty_dash(&p.paid_at), classes: "" }),
                    field_text(FieldText { value: empty_dash(&p.transaction_id), classes: "" }),
                ],
            })
            .collect();
        let student_id = if self.filter_student_id > 0 {
            self.filter_student_id.to_string()
        } else {
            String::new()
        };
        let method_choices = payment_method_choice_pairs();
        let actions = html! {
            (table_button_filter(TableButtonFilter {
                panel: form(FormOpts {
                    attrs: form_hx_get_route::<PaymentTableKey, StudentPaymentsListRouteTag>(StudentPaymentsListRouteTag),
                    inputs: PaymentFilterForm::render_inputs(
                        &FormCtx::form::<PaymentFilterForm>()
                            .value(PaymentFilterFormField::PaymentMethod, &self.filter_payment_method)
                            .choices(PaymentFilterFormField::PaymentMethod, &method_choices)
                            .value(PaymentFilterFormField::StudentId, &student_id)
                            .display(PaymentFilterFormField::StudentId, &self.filter_student_display)
                            .value(PaymentFilterFormField::TransactionId, &self.filter_transaction_id),
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
                (table_create_button::<PaymentTableKey, PaymentCreateModalKey>(
                    Some("plus"),
                    "btn-square btn-outline btn-sm",
                ))
            }
        };
        data_table_list_grid_with_subtitle::<PaymentTableKey>(
            "Student Payments",
            "Fee payments",
            actions,
            &headers,
            &rows,
            render_pagination::<PaymentTableKey>(
                &self.path_and_query,
                self.payments.number,
                self.payments.num_pages,
                true,
            ),
        )
    }
}

impl lariv_rs::template::RenderAppPane for PaymentListPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        scaffold_pane(
            students_hub_menu(),
            payments_crumbs("All Payments"),
            self.render_table(),
        )
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        scaffold_main(payments_crumbs("All Payments"), self.render_table())
    }
}

impl RenderTemplate for PaymentListPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold(
            &format!("Student Payments — {APP_TITLE}"),
            chrome,
            students_hub_menu(),
            payments_crumbs("All Payments"),
            self.render_table(),
        )
    }
}

#[derive(Generic)]
pub struct PaymentDetailPage {
    pub id: i64,
    pub student_id: i64,
    pub student_display: String,
    pub amount: String,
    pub payment_method: String,
    pub transaction_id: String,
    pub paid_at: String,
    pub remarks: String,
    pub is_admin: bool,
}

impl PaymentDetailPage {
    fn pane_body(&self) -> Markup {
        let title = format!("{} · {}", self.student_display, self.amount);
        detail(html! {
            (container_column("", html! {
                (detail_header(DetailHeader {
                    title: &title,
                    actions: html! {
                        (button_download_route(
                            StudentPaymentsReceiptRouteTag::new(self.id),
                            "Download Receipt",
                            "btn-outline btn-secondary btn-sm",
                        ))
                        @if self.is_admin {
                            (button_modal_form_route(
                                StudentPaymentsEditGetRouteTag::new(self.id),
                                StudentPaymentsEditPostRouteTag::new(self.id),
                                "Edit",
                                PaymentEditModalKey::ID,
                                "btn btn-outline btn-sm",
                            ))
                        }
                    },
                }))
                (label("Student record", {
                    let href = if self.student_id > 0 {
                        StudentsDetailRouteTag::new(self.student_id).url()
                    } else {
                        String::new()
                    };
                    field_related(&self.student_display, &href)
                }))
                (label("Amount", field_text(FieldText { value: &self.amount, classes: "" })))
                (label("Method", field_text(FieldText { value: &self.payment_method, classes: "" })))
                (label("Transaction ID", field_text(FieldText { value: empty_dash(&self.transaction_id), classes: "" })))
                (label("Paid on", field_text(FieldText { value: empty_dash(&self.paid_at), classes: "" })))
                (label("Remarks", field_text(FieldText { value: empty_dash(&self.remarks), classes: "" })))
            }))
        })
    }
}

impl lariv_rs::template::RenderAppPane for PaymentDetailPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        scaffold_pane(
            payment_detail_menu(self.id, &self.student_display, self.is_admin),
            payment_detail_crumbs(self.id, &self.student_display, None),
            self.pane_body(),
        )
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        scaffold_main(
            payment_detail_crumbs(self.id, &self.student_display, None),
            self.pane_body(),
        )
    }
}

impl RenderTemplate for PaymentDetailPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold(
            &format!("{} — {APP_TITLE}", self.student_display),
            chrome,
            payment_detail_menu(self.id, &self.student_display, self.is_admin),
            payment_detail_crumbs(self.id, &self.student_display, None),
            self.pane_body(),
        )
    }
}

#[derive(Generic)]
pub struct PaymentFormPage {
    pub id: i64,
    pub student_id: i64,
    pub student_display: String,
    pub amount: String,
    pub payment_method: String,
    pub transaction_id: String,
    pub paid_at: String,
    pub remarks: String,
    pub error: String,
    pub form_name: String,
    pub refresh_table: String,
    pub target_input: String,
}

impl RenderTemplate for PaymentFormPage {
    fn render(&self, _chrome: &ShellChrome) -> Markup {
        let is_create = self.id == 0;
        let form_attrs = if is_create {
            form_hx_post_url::<PaymentCreateModalKey>(&modal_create_post_query(
                StudentPaymentsCreatePostRouteTag,
                &self.form_name,
                &self.refresh_table,
                &self.target_input,
            ))
        } else {
            form_hx_post_url::<PaymentEditModalKey>(&modal_edit_post_url(
                StudentPaymentsEditPostRouteTag::new(self.id),
                &self.form_name,
            ))
        };
        let choices = payment_method_choice_pairs();
        let student_id = if self.student_id > 0 {
            self.student_id.to_string()
        } else {
            String::new()
        };
        let body = form(FormOpts {
            attrs: form_attrs,
            title: if is_create { "Record payment" } else { "Edit payment" },
            subtitle: if is_create {
                "Log a payment for a student."
            } else {
                "Update amount, method, or references."
            },
            form_error: Some(self.error.as_str()).filter(|e| !e.is_empty()),
            inputs: PaymentForm::render_inputs(
                &FormCtx::form::<PaymentForm>()
                    .value(PaymentFormField::StudentId, &student_id)
                    .display(PaymentFormField::StudentId, &self.student_display)
                    .value(PaymentFormField::Amount, &self.amount)
                    .value(PaymentFormField::PaymentMethod, &self.payment_method)
                    .choices(PaymentFormField::PaymentMethod, &choices)
                    .value(PaymentFormField::TransactionId, &self.transaction_id)
                    .value(PaymentFormField::PaidAt, &self.paid_at)
                    .value(PaymentFormField::Remarks, &self.remarks),
            ),
            actions: html! {
                (button_submit(ButtonSubmit { label: "Save payment", ..Default::default() }))
                @if !is_create {
                    (button_modal_form_route(
                        StudentPaymentsDeleteGetRouteTag::new(self.id),
                        StudentPaymentsDeletePostRouteTag::new(self.id),
                        "Delete",
                        PaymentDeleteModalKey::ID,
                        "btn-error",
                    ))
                }
            },
            ..Default::default()
        });
        if is_create {
            modal_keyed::<PaymentCreateModalKey>("", body)
        } else {
            modal_keyed::<PaymentEditModalKey>("", body)
        }
    }
}

#[derive(Generic)]
pub struct PaymentSelectPage {
    pub payments: ObjectList<PaymentRow>,
    pub filter_transaction_id: String,
    pub path_and_query: String,
    pub target_input: String,
}

impl RenderPickerSelect<PaymentSelectTableKey, PaymentSelectModalKey> for PaymentSelectPage {
    fn render_table(&self) -> Markup {
        let headers = [
            TableColumnHeader { key: "Student", label: "Student", sort_url: None, push_url: false },
            TableColumnHeader { key: "Amount", label: "Amount", sort_url: None, push_url: false },
        ];
        let target = if self.target_input.is_empty() { "PaymentID" } else { self.target_input.as_str() };
        let rows: Vec<TableRow> = self
            .payments
            .items
            .iter()
            .map(|p| TableRow {
                attrs: row_attr_select(target, &p.id.to_string(), &p.student_display),
                cells: vec![
                    field_text(FieldText { value: &p.student_display, classes: "" }),
                    field_text(FieldText { value: &p.amount, classes: "" }),
                ],
            })
            .collect();
        data_table_list_refresh::<PaymentSelectTableKey>(
            "Select Payment",
            table_button_filter(TableButtonFilter {
                panel: form(FormOpts {
                    attrs: form_hx_get_picker_route::<
                        PaymentSelectTableKey,
                        PaymentSelectModalKey,
                        StudentPaymentsSelectRouteTag,
                    >(StudentPaymentsSelectRouteTag)
                        .set("hx-push-url", "false"),
                    inputs: html! {
                        (PaymentFilterForm::render_inputs(
                            &FormCtx::form::<PaymentFilterForm>()
                                .value(PaymentFilterFormField::TransactionId, &self.filter_transaction_id),
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
            render_picker_pagination::<PaymentSelectModalKey>(
                &self.path_and_query,
                self.payments.number,
                self.payments.num_pages,
            ),
            &self.path_and_query,
        )
    }
}

impl RenderTemplate for PaymentSelectPage {
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
                attrs: form_hx_post_selector(&StudentPaymentsDeletePostRouteTag::new(self.id).url(), &target),
                form_error: Some(self.error.as_str()).filter(|e| !e.is_empty()),
                ..Default::default()
            }),
            ..Default::default()
        })
    }
}
