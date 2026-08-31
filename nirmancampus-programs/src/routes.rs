use super::{
    handlers,
    keys::{
        ProgramDeleteModalKey, ProgramMediaMultiSelectModalKey, ProgramMediaMultiSelectTableKey,
        ProgramSelectModalKey, ProgramSelectTableKey, ProgramTableKey, StructureUnitDeleteModalKey,
    },
};

lariv_rs::define_plugin_routes! {
    plugin: NirmancampusProgramsTag;
    proof: NirmancampusProgramsRoutesProof;
    routes: [
        get ProgramsListRouteTag, "/programs", handlers::programs::list, fragment(ProgramTableKey);
        get ProgramsCreateGetRouteTag, "/programs/create", handlers::programs::create_get, modal;
        post ProgramsCreatePostRouteTag, "/programs/create", handlers::programs::create_post;
        get ProgramsDetailRouteTag, "/programs/{id}", handlers::programs::detail;
        get ProgramsEditGetRouteTag, "/programs/{id}/edit", handlers::programs::edit_get, modal;
        post ProgramsEditPostRouteTag, "/programs/{id}/edit", handlers::programs::edit_post;
        get ProgramsDeleteGetRouteTag, "/programs/{id}/delete", handlers::programs::delete_get, modal;
        post ProgramsDeletePostRouteTag, "/programs/{id}/delete", bare handlers::programs::delete_post, fragment(ProgramDeleteModalKey);
        get ProgramsSelectRouteTag, "/programs/select", handlers::programs::select, fk_select(ProgramSelectTableKey, ProgramSelectModalKey);
        get ProgramMediaMultiSelectRouteTag, "/programs/program-media/multiselect", handlers::programs::media_multi_select, fk_select(ProgramMediaMultiSelectTableKey, ProgramMediaMultiSelectModalKey);

        get ProgramsStructureEditRouteTag, "/programs/{id}/structure/edit", handlers::structure::edit;
        get ProgramsStructureUnitCreateGetRouteTag, "/programs/{id}/structure/units/new", handlers::structure::unit_create_get, modal;
        post ProgramsStructureUnitCreatePostRouteTag, "/programs/{id}/structure/units", handlers::structure::unit_create_post;
        get ProgramsStructureUnitEditGetRouteTag, "/programs/{id}/structure/units/{unit_id}/edit", handlers::structure::unit_edit_get, modal;
        post ProgramsStructureUnitUpdatePostRouteTag, "/programs/{id}/structure/units/{unit_id}", handlers::structure::unit_update_post;
        get ProgramsStructureUnitDeleteGetRouteTag, "/programs/{id}/structure/units/{unit_id}/delete", handlers::structure::unit_delete_get, modal;
        post ProgramsStructureUnitDeletePostRouteTag, "/programs/{id}/structure/units/{unit_id}/delete", bare handlers::structure::unit_delete_post, fragment(StructureUnitDeleteModalKey);
    ]
}
