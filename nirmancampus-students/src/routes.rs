use super::{
    handlers,
    keys::{StudentDeleteModalKey, StudentSelectModalKey, StudentSelectTableKey, StudentTableKey},
};

lariv_rs::define_plugin_routes! {
    plugin: NirmancampusStudentsTag;
    proof: NirmancampusStudentsRoutesProof;
    routes: [
        get StudentsListRouteTag, "/students", handlers::students::list, fragment(StudentTableKey);
        get StudentsCreateGetRouteTag, "/students/create", handlers::students::create_get, modal;
        post StudentsCreatePostRouteTag, "/students/create", handlers::students::create_post;
        get StudentsDetailRouteTag, "/students/{id}", handlers::students::detail;
        get StudentsEditGetRouteTag, "/students/{id}/edit", handlers::students::edit_get, modal;
        post StudentsEditPostRouteTag, "/students/{id}/edit", handlers::students::edit_post;
        get StudentsDeleteGetRouteTag, "/students/{id}/delete", handlers::students::delete_get, modal;
        post StudentsDeletePostRouteTag, "/students/{id}/delete", bare handlers::students::delete_post, fragment(StudentDeleteModalKey);
        get StudentsSelectRouteTag, "/students/select", handlers::students::select, fk_select(StudentSelectTableKey, StudentSelectModalKey);
    ]
}
