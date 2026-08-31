use super::{
    handlers,
    keys::{
        CourseDeleteModalKey, CourseMultiSelectModalKey, CourseMultiSelectTableKey,
        CourseSelectModalKey, CourseSelectTableKey, CourseTableKey,
    },
};

lariv_rs::define_plugin_routes! {
    plugin: NirmancampusCoursesTag;
    proof: NirmancampusCoursesRoutesProof;
    routes: [
        get CoursesListRouteTag, "/courses", handlers::courses::list, fragment(CourseTableKey);
        get CoursesCreateGetRouteTag, "/courses/create", handlers::courses::create_get, modal;
        post CoursesCreatePostRouteTag, "/courses/create", handlers::courses::create_post;
        get CoursesDetailRouteTag, "/courses/{id}", handlers::courses::detail;
        get CoursesEditGetRouteTag, "/courses/{id}/edit", handlers::courses::edit_get, modal;
        post CoursesEditPostRouteTag, "/courses/{id}/edit", handlers::courses::edit_post;
        get CoursesDeleteGetRouteTag, "/courses/{id}/delete", handlers::courses::delete_get, modal;
        post CoursesDeletePostRouteTag, "/courses/{id}/delete", bare handlers::courses::delete_post, fragment(CourseDeleteModalKey);
        get CoursesSelectRouteTag, "/courses/select", handlers::courses::select, fk_select(CourseSelectTableKey, CourseSelectModalKey);
        get CoursesMultiSelectRouteTag, "/courses/multi-select", handlers::courses::multi_select, fk_select(CourseMultiSelectTableKey, CourseMultiSelectModalKey);
    ]
}
