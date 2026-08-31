use super::keys::PaymentCreateModalKey;
use super::routes::{StudentPaymentsCreateGetRouteTag, StudentPaymentsCreatePostRouteTag};

lariv_rs::impl_create_modal!(
    PaymentCreateModalKey,
    StudentPaymentsCreateGetRouteTag,
    StudentPaymentsCreatePostRouteTag,
    "studentpayments.PaymentCreateForm"
);
