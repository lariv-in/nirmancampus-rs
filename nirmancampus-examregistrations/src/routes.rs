use super::{
    handlers,
    keys::{
        ExamDeleteModalKey, ExamSelectModalKey, ExamSelectTableKey, ExamTableKey,
    },
};

lariv_rs::define_plugin_routes! {
    plugin: NirmancampusExamRegistrationsTag;
    proof: NirmancampusExamRegistrationsRoutesProof;
    routes: [
        get ExamRegistrationsListRouteTag, "/exam-registrations", handlers::exams::list, fragment(ExamTableKey);
        get ExamRegistrationsCreateGetRouteTag, "/exam-registrations/create", handlers::exams::create_get, modal;
        post ExamRegistrationsCreatePostRouteTag, "/exam-registrations/create", handlers::exams::create_post;
        get ExamRegistrationsBulkGetRouteTag, "/exam-registrations/bulk-create-academic-record", handlers::bulk::bulk_get, modal;
        post ExamRegistrationsBulkPostRouteTag, "/exam-registrations/bulk-create-academic-record", handlers::bulk::bulk_post;
        get ExamRegistrationsDetailRouteTag, "/exam-registrations/{id}", handlers::exams::detail;
        get ExamRegistrationsEditGetRouteTag, "/exam-registrations/{id}/edit", handlers::exams::edit_get, modal;
        post ExamRegistrationsEditPostRouteTag, "/exam-registrations/{id}/edit", handlers::exams::edit_post;
        get ExamRegistrationsDeleteGetRouteTag, "/exam-registrations/{id}/delete", handlers::exams::delete_get, modal;
        post ExamRegistrationsDeletePostRouteTag, "/exam-registrations/{id}/delete", bare handlers::exams::delete_post, fragment(ExamDeleteModalKey);
        get ExamRegistrationsSelectRouteTag, "/exam-registrations/select", handlers::exams::select, fk_select(ExamSelectTableKey, ExamSelectModalKey);
        get ExamRegistrationsReceiptRouteTag, "/exam-registrations/academic-record/{id}/download-receipt", bare handlers::receipt::download, file;
    ]
}
