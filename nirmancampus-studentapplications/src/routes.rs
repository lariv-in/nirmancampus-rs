use super::{
    handlers,
    keys::{
        ApplicationDeleteModalKey, ApplicationSelectModalKey, ApplicationSelectTableKey,
        ApplicationTableKey,
    },
};

lariv_rs::define_plugin_routes! {
    plugin: NirmancampusStudentApplicationsTag;
    proof: NirmancampusStudentApplicationsRoutesProof;
    routes: [
        get StudentApplicationsListRouteTag, "/student-applications", handlers::applications::list, fragment(ApplicationTableKey);
        get StudentApplicationsCreateGetRouteTag, "/student-applications/create", handlers::applications::create_get, modal;
        post StudentApplicationsCreatePostRouteTag, "/student-applications/create", handlers::applications::create_post;
        get StudentApplicationsDetailRouteTag, "/student-applications/{id}", handlers::applications::detail;
        get StudentApplicationsEditGetRouteTag, "/student-applications/{id}/edit", handlers::applications::edit_get, modal;
        post StudentApplicationsEditPostRouteTag, "/student-applications/{id}/edit", handlers::applications::edit_post;
        get StudentApplicationsDeleteGetRouteTag, "/student-applications/{id}/delete", handlers::applications::delete_get, modal;
        post StudentApplicationsDeletePostRouteTag, "/student-applications/{id}/delete", bare handlers::applications::delete_post, fragment(ApplicationDeleteModalKey);
        get StudentApplicationsSelectRouteTag, "/student-applications/select", handlers::applications::select, fk_select(ApplicationSelectTableKey, ApplicationSelectModalKey);
    ]
}
