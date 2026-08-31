use super::{
    handlers,
    keys::{
        PaymentDeleteModalKey, PaymentSelectModalKey, PaymentSelectTableKey, PaymentTableKey,
    },
};

lariv_rs::define_plugin_routes! {
    plugin: NirmancampusStudentPaymentsTag;
    proof: NirmancampusStudentPaymentsRoutesProof;
    routes: [
        get StudentPaymentsListRouteTag, "/student-payments", handlers::payments::list, fragment(PaymentTableKey);
        get StudentPaymentsCreateGetRouteTag, "/student-payments/create", handlers::payments::create_get, modal;
        post StudentPaymentsCreatePostRouteTag, "/student-payments/create", handlers::payments::create_post;
        get StudentPaymentsDetailRouteTag, "/student-payments/{id}", handlers::payments::detail;
        get StudentPaymentsEditGetRouteTag, "/student-payments/{id}/edit", handlers::payments::edit_get, modal;
        post StudentPaymentsEditPostRouteTag, "/student-payments/{id}/edit", handlers::payments::edit_post;
        get StudentPaymentsDeleteGetRouteTag, "/student-payments/{id}/delete", handlers::payments::delete_get, modal;
        post StudentPaymentsDeletePostRouteTag, "/student-payments/{id}/delete", bare handlers::payments::delete_post, fragment(PaymentDeleteModalKey);
        get StudentPaymentsSelectRouteTag, "/student-payments/select", handlers::payments::select, fk_select(PaymentSelectTableKey, PaymentSelectModalKey);
        get StudentPaymentsReceiptRouteTag, "/student-payments/{id}/download-receipt", bare handlers::receipt::download, file;
    ]
}
