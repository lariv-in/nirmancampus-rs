use super::keys::AcademicRecordCreateModalKey;
use super::routes::{AcademicRecordsCreateGetRouteTag, AcademicRecordsCreatePostRouteTag};

lariv_rs::impl_create_modal!(
    AcademicRecordCreateModalKey,
    AcademicRecordsCreateGetRouteTag,
    AcademicRecordsCreatePostRouteTag,
    "academicrecords.AcademicRecordCreateForm"
);
