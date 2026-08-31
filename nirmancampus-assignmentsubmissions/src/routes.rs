use super::{
    handlers,
    keys::{AssignmentDeleteModalKey, AssignmentTableKey},
};

lariv_rs::define_plugin_routes! {
    plugin: NirmancampusAssignmentSubmissionsTag;
    proof: NirmancampusAssignmentSubmissionsRoutesProof;
    routes: [
        get AssignmentSubmissionsListRouteTag, "/assignment-submissions", handlers::assignments::list, fragment(AssignmentTableKey);
        get AssignmentSubmissionsCreateGetRouteTag, "/assignment-submissions/create", handlers::assignments::create_get, modal;
        post AssignmentSubmissionsCreatePostRouteTag, "/assignment-submissions/create", handlers::assignments::create_post;
        get AssignmentSubmissionsBulkCreateGetRouteTag, "/assignment-submissions/bulk-create-academic-record", handlers::bulk::bulk_create_get, modal;
        post AssignmentSubmissionsBulkCreatePostRouteTag, "/assignment-submissions/bulk-create-academic-record", handlers::bulk::bulk_create_post;
        get AssignmentSubmissionsBulkMarksGetRouteTag, "/assignment-submissions/bulk-add-marks-academic-record", handlers::bulk::bulk_marks_get, modal;
        post AssignmentSubmissionsBulkMarksPostRouteTag, "/assignment-submissions/bulk-add-marks-academic-record", handlers::bulk::bulk_marks_post;
        get AssignmentSubmissionsReceiptRouteTag, "/assignment-submissions/academic-record/{id}/download-receipt", bare handlers::receipt::download, file;
        get AssignmentSubmissionsDetailRouteTag, "/assignment-submissions/{id}", handlers::assignments::detail;
        get AssignmentSubmissionsEditGetRouteTag, "/assignment-submissions/{id}/edit", handlers::assignments::edit_get, modal;
        post AssignmentSubmissionsEditPostRouteTag, "/assignment-submissions/{id}/edit", handlers::assignments::edit_post;
        get AssignmentSubmissionsDeleteGetRouteTag, "/assignment-submissions/{id}/delete", handlers::assignments::delete_get, modal;
        post AssignmentSubmissionsDeletePostRouteTag, "/assignment-submissions/{id}/delete", bare handlers::assignments::delete_post, fragment(AssignmentDeleteModalKey);
    ]
}
