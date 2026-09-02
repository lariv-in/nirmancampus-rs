use super::{
    handlers,
    keys::{FeeDeleteModalKey, FeeTableKey},
};

lariv_rs::define_plugin_routes! {
    plugin: NirmancampusStudentFeesTag;
    proof: NirmancampusStudentFeesRoutesProof;
    routes: [
        get StudentFeesListRouteTag, "/student-fees", handlers::fees::list, fragment(FeeTableKey);
        get StudentFeesPrefsGetRouteTag, "/student-fees/preferences", handlers::preferences::get;
        post StudentFeesPrefsPostRouteTag, "/student-fees/preferences", handlers::preferences::post;
        post StudentFeesSyncRouteTag, "/student-fees/sync", handlers::fees::sync;
        get StudentFeesCreateGetRouteTag, "/student-fees/create", handlers::fees::create_get, modal;
        post StudentFeesCreatePostRouteTag, "/student-fees/create", handlers::fees::create_post;
        get StudentFeesDetailRouteTag, "/student-fees/{id}", handlers::fees::detail;
        get StudentFeesEditGetRouteTag, "/student-fees/{id}/edit", handlers::fees::edit_get, modal;
        post StudentFeesEditPostRouteTag, "/student-fees/{id}/edit", handlers::fees::edit_post;
        get StudentFeesDeleteGetRouteTag, "/student-fees/{id}/delete", handlers::fees::delete_get, modal;
        post StudentFeesDeletePostRouteTag, "/student-fees/{id}/delete", bare handlers::fees::delete_post, fragment(FeeDeleteModalKey);
    ]
}
