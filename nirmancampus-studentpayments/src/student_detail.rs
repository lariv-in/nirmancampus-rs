//! Embeds a payments table on student detail via `StudentDetailRelatedCap`.

use maud::html;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder};

use lariv_rs::{
    components::{
        ButtonModalForm, FieldText, TableColumnHeader, TableRow, button_modal_form,
        data_table_list_grid, field_text, row_attr_navigate_route, SwapKey,
    },
    http::RouteQueryBuilder,
    plugins::users::state::AuthContext,
    web::CreateModal,
};
use nirmancampus_common::is_admin;
use nirmancampus_students::student_detail_related::{
    self, StudentDetailRelatedRegistrar, StudentDetailRelatedRegistry,
};

use crate::entities::payment::{self, Entity as PaymentEntity};
use crate::handlers::payments::{format_amount, payment_method_label};
use crate::keys::{PaymentCreateModalKey, StudentDetailPaymentsKey};
use crate::routes::{StudentPaymentsCreateGetRouteTag, StudentPaymentsDetailRouteTag};

#[derive(Clone, Copy, Default)]
pub struct StudentDetailHook;

impl StudentDetailRelatedRegistrar for StudentDetailHook {
    fn register_student_detail_related(
        self,
        cap: StudentDetailRelatedRegistry,
    ) -> StudentDetailRelatedRegistry {
        cap.push(student_detail_related::section(10, |db, student_id, auth| async move {
            payments_section(&db, student_id, &auth).await
        }))
    }
}

fn create_url_with_student(student_id: i64) -> String {
    RouteQueryBuilder::new(StudentPaymentsCreateGetRouteTag)
        .query("StudentID", student_id)
        .build()
}

struct PaymentCard {
    id: i64,
    amount: String,
    method: String,
    paid_at: String,
    transaction_id: String,
}

async fn payments_section(db: &DatabaseConnection, student_id: i64, auth: &AuthContext) -> String {
    let Ok(rows) = PaymentEntity::find()
        .filter(payment::Column::DeletedAt.is_null())
        .filter(payment::Column::StudentId.eq(student_id))
        .order_by_desc(payment::Column::Id)
        .all(db)
        .await
    else {
        return String::new();
    };

    let items: Vec<PaymentCard> = rows
        .into_iter()
        .map(|p| PaymentCard {
            id: p.id,
            amount: format_amount(p.amount),
            method: payment_method_label(&p.payment_method),
            paid_at: p
                .paid_at
                .map(lariv_rs::datetime::format_date)
                .unwrap_or_default(),
            transaction_id: p.transaction_id().to_string(),
        })
        .collect();

    let headers = [
        TableColumnHeader {
            key: "Amount",
            label: "Amount",
            sort_url: None,
            push_url: false,
        },
        TableColumnHeader {
            key: "Method",
            label: "Method",
            sort_url: None,
            push_url: false,
        },
        TableColumnHeader {
            key: "PaidAt",
            label: "Paid on",
            sort_url: None,
            push_url: false,
        },
        TableColumnHeader {
            key: "TransactionID",
            label: "Transaction ID",
            sort_url: None,
            push_url: false,
        },
    ];
    let table_rows: Vec<TableRow> = items
        .iter()
        .map(|p| TableRow {
            attrs: row_attr_navigate_route(StudentPaymentsDetailRouteTag::new(p.id)),
            cells: vec![
                field_text(FieldText {
                    value: &p.amount,
                    classes: "",
                }),
                field_text(FieldText {
                    value: &p.method,
                    classes: "",
                }),
                field_text(FieldText {
                    value: &p.paid_at,
                    classes: "",
                }),
                field_text(FieldText {
                    value: &p.transaction_id,
                    classes: "",
                }),
            ],
        })
        .collect();

    let create_url = create_url_with_student(student_id);
    let admin = is_admin(auth);
    html! {
        div class="mt-4" {
            (data_table_list_grid::<StudentDetailPaymentsKey>(
                "Payments",
                html! {
                    @if admin {
                        (button_modal_form(ButtonModalForm {
                            href: &create_url,
                            name: PaymentCreateModalKey::FORM_NAME,
                            modal_uid: PaymentCreateModalKey::ID,
                            label: "",
                            icon_name: Some("plus"),
                            classes: "btn-square btn-outline btn-sm",
                            ..Default::default()
                        }))
                    }
                },
                &headers,
                &table_rows,
                html! {},
            ))
        }
    }
    .into_string()
}
