use super::{
    handlers,
    keys::{
        SessionDeleteModalKey, SessionSelectModalKey, SessionSelectTableKey, SessionTableKey,
    },
};

lariv_rs::define_plugin_routes! {
    plugin: NirmancampusSessionsTag;
    proof: NirmancampusSessionsRoutesProof;
    routes: [
        get SessionsListRouteTag, "/sessions", handlers::sessions::list, fragment(SessionTableKey);
        get SessionsCreateGetRouteTag, "/sessions/create", handlers::sessions::create_get, modal;
        post SessionsCreatePostRouteTag, "/sessions/create", handlers::sessions::create_post;
        get SessionsDetailRouteTag, "/sessions/{id}", handlers::sessions::detail;
        get SessionsEditGetRouteTag, "/sessions/{id}/edit", handlers::sessions::edit_get, modal;
        post SessionsEditPostRouteTag, "/sessions/{id}/edit", handlers::sessions::edit_post;
        get SessionsDeleteGetRouteTag, "/sessions/{id}/delete", handlers::sessions::delete_get, modal;
        post SessionsDeletePostRouteTag, "/sessions/{id}/delete", bare handlers::sessions::delete_post, fragment(SessionDeleteModalKey);
        get SessionsSelectRouteTag, "/sessions/select", handlers::sessions::select, fk_select(SessionSelectTableKey, SessionSelectModalKey);
    ]
}
