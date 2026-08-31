use lariv_rs::html_form::{
    html_form,
    widgets::{Date, Number, Select, Text, Textarea},
};
use nirmancampus_students::routes::StudentsSelectRouteTag;

#[html_form]
pub struct PaymentForm {
    #[form(
        label = "Student",
        required,
        widget = ForeignKey,
        route = StudentsSelectRouteTag,
        swap_key = "fk-payment-student",
        display = "student",
        placeholder = "Select a student…"
    )]
    pub student_id: i64,

    #[form(label = "Amount", required, widget = Number)]
    pub amount: String,

    #[form(label = "Payment method", widget = Select, choices = "payment_method", required)]
    pub payment_method: String,

    #[form(label = "Transaction ID", widget = Text)]
    pub transaction_id: String,

    #[form(label = "Paid on", widget = Date)]
    pub paid_at: String,

    #[form(label = "Remarks", widget = Textarea, rows = 3)]
    pub remarks: String,
}

#[html_form]
pub struct PaymentFilterForm {
    #[form(label = "Method", widget = Select, choices = "payment_method")]
    pub payment_method: String,

    #[form(
        label = "Student",
        widget = ForeignKey,
        route = StudentsSelectRouteTag,
        swap_key = "fk-payment-filter-student",
        display = "student",
        placeholder = "Filter by student…"
    )]
    pub student_id: i64,

    #[form(label = "Transaction ID", widget = Text)]
    pub transaction_id: String,
}
