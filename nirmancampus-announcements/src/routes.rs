use super::{
    handlers,
    keys::{
        AnnouncementDeleteModalKey, AnnouncementSelectModalKey, AnnouncementSelectTableKey,
        AnnouncementTableKey,
    },
};

lariv_rs::define_plugin_routes! {
    plugin: NirmancampusAnnouncementsTag;
    proof: NirmancampusAnnouncementsRoutesProof;
    routes: [
        get AnnouncementsListRouteTag, "/announcements", handlers::announcements::list, fragment(AnnouncementTableKey);
        get AnnouncementsCreateGetRouteTag, "/announcements/create", handlers::announcements::create_get, modal;
        post AnnouncementsCreatePostRouteTag, "/announcements/create", handlers::announcements::create_post;
        get AnnouncementsDetailRouteTag, "/announcements/{id}", handlers::announcements::detail;
        get AnnouncementsEditGetRouteTag, "/announcements/{id}/edit", handlers::announcements::edit_get, modal;
        post AnnouncementsEditPostRouteTag, "/announcements/{id}/edit", handlers::announcements::edit_post;
        get AnnouncementsDeleteGetRouteTag, "/announcements/{id}/delete", handlers::announcements::delete_get, modal;
        post AnnouncementsDeletePostRouteTag, "/announcements/{id}/delete", bare handlers::announcements::delete_post, fragment(AnnouncementDeleteModalKey);
        get AnnouncementsSelectRouteTag, "/announcements/select", handlers::announcements::select, fk_select(AnnouncementSelectTableKey, AnnouncementSelectModalKey);
    ]
}
