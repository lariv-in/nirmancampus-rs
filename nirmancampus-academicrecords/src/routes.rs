use super::{
    handlers,
    keys::{
        AcademicRecordDeleteModalKey, AcademicRecordSelectModalKey, AcademicRecordSelectTableKey,
        AcademicRecordTableKey, PsuSelectModalKey, PsuSelectTableKey,
    },
};

lariv_rs::define_plugin_routes! {
    plugin: NirmancampusAcademicRecordsTag;
    proof: NirmancampusAcademicRecordsRoutesProof;
    routes: [
        get AcademicRecordsListRouteTag, "/academic-records", handlers::records::list, fragment(AcademicRecordTableKey);
        get AcademicRecordsCreateGetRouteTag, "/academic-records/create", handlers::records::create_get, modal;
        post AcademicRecordsCreatePostRouteTag, "/academic-records/create", handlers::records::create_post;
        get AcademicRecordsPsuSelectRouteTag, "/academic-records/program-structure-units/select", handlers::records::psu_select, fk_select(PsuSelectTableKey, PsuSelectModalKey);
        get AcademicRecordsSelectRouteTag, "/academic-records/select", handlers::records::select, fk_select(AcademicRecordSelectTableKey, AcademicRecordSelectModalKey);
        get AcademicRecordsDownloadPdfRouteTag, "/academic-records/{id}/download-pdf", bare handlers::records::download_pdf, file;
        get AcademicRecordsDetailRouteTag, "/academic-records/{id}", handlers::records::detail;
        get AcademicRecordsEditGetRouteTag, "/academic-records/{id}/edit", handlers::records::edit_get, modal;
        post AcademicRecordsEditPostRouteTag, "/academic-records/{id}/edit", handlers::records::edit_post;
        get AcademicRecordsDeleteGetRouteTag, "/academic-records/{id}/delete", handlers::records::delete_get, modal;
        post AcademicRecordsDeletePostRouteTag, "/academic-records/{id}/delete", bare handlers::records::delete_post, fragment(AcademicRecordDeleteModalKey);
    ]
}
