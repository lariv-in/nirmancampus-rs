use super::{
    handlers,
    keys::{
        ImportantLinkDeleteModalKey, ImportantLinksTableKey, StudentZoneItemDeleteModalKey,
        StudentZoneItemTableKey, StudentZoneSectionDeleteModalKey,
        StudentZoneSectionSelectModalKey, StudentZoneSectionSelectTableKey,
        StudentZoneSectionTableKey, TblfeeTableKey,
    },
};

lariv_rs::define_plugin_routes! {
    plugin: NirmancampusWebsiteTag;
    proof: NirmancampusWebsiteRoutesProof;
    routes: [
        get WebsiteHomeRouteTag, "/", bare handlers::public::home, raw;
        get WebsiteProgramsRouteTag, "/programs-offered", bare handlers::public::programs, raw;
        get WebsiteContactRouteTag, "/contact-us", bare handlers::public::contact, raw;
        get WebsitePrivacyRouteTag, "/privacy-policy", bare handlers::public::privacy, raw;
        get WebsiteStudentZonePublicRouteTag, "/students-zone", bare handlers::public::student_zone, raw;
        post WebsiteStudentZoneLoginRouteTag, "/students-zone/login", bare handlers::public::student_zone_login, raw;
        post WebsiteStudentZoneLogoutRouteTag, "/students-zone/logout", bare handlers::public::student_zone_logout, raw;
        get WebsiteStudentZoneItemPublicRouteTag, "/students-zone/item/{id}", bare handlers::public::student_zone_item, raw;
        get WebsiteImportantLinkItemPublicRouteTag, "/important-links/item/{id}", bare handlers::public::important_link_item, raw;
        get WebsiteEssentialCommitteesRouteTag, "/contact-us/essential-committees-list", bare handlers::public::essential_committees, file;
        get WebsiteStaticRouteTag, "/nirman/static/{*path}", bare handlers::static_files::serve, file;
        get WebsitePublicMediaRouteTag, "/media/{id}", bare handlers::public::media, file;

        get WebsiteAppLandingRouteTag, "/website", handlers::admin::landing;

        get WebsiteImportantLinksListRouteTag, "/website/important-links", handlers::important_links::list, fragment(ImportantLinksTableKey);
        get WebsiteImportantLinksCreateGetRouteTag, "/website/important-links/create", handlers::important_links::create_get, modal;
        post WebsiteImportantLinksCreatePostRouteTag, "/website/important-links/create", handlers::important_links::create_post;
        get WebsiteImportantLinksDetailRouteTag, "/website/important-links/{id}", handlers::important_links::detail;
        get WebsiteImportantLinksEditGetRouteTag, "/website/important-links/{id}/edit", handlers::important_links::edit_get, modal;
        post WebsiteImportantLinksEditPostRouteTag, "/website/important-links/{id}/edit", handlers::important_links::edit_post;
        get WebsiteImportantLinksDeleteGetRouteTag, "/website/important-links/{id}/delete", handlers::important_links::delete_get, modal;
        post WebsiteImportantLinksDeletePostRouteTag, "/website/important-links/{id}/delete", bare handlers::important_links::delete_post, fragment(ImportantLinkDeleteModalKey);

        get WebsiteStudentZoneSectionsListRouteTag, "/website/student-zone", handlers::student_zone::section_list, fragment(StudentZoneSectionTableKey);
        get WebsiteStudentZoneSectionsCreateGetRouteTag, "/website/student-zone/sections/create", handlers::student_zone::section_create_get, modal;
        post WebsiteStudentZoneSectionsCreatePostRouteTag, "/website/student-zone/sections/create", handlers::student_zone::section_create_post;
        get WebsiteStudentZoneSectionsSelectRouteTag, "/website/student-zone/sections/select", handlers::student_zone::section_select, fk_select(StudentZoneSectionSelectTableKey, StudentZoneSectionSelectModalKey);
        get WebsiteStudentZoneItemsListRouteTag, "/website/student-zone/items", handlers::student_zone::item_list, fragment(StudentZoneItemTableKey);
        get WebsiteStudentZoneItemsCreateGetRouteTag, "/website/student-zone/items/create", handlers::student_zone::item_create_get, modal;
        post WebsiteStudentZoneItemsCreatePostRouteTag, "/website/student-zone/items/create", handlers::student_zone::item_create_post;
        get WebsiteStudentZoneItemsDetailRouteTag, "/website/student-zone/items/{id}", handlers::student_zone::item_detail;
        get WebsiteStudentZoneItemsEditGetRouteTag, "/website/student-zone/items/{id}/edit", handlers::student_zone::item_edit_get, modal;
        post WebsiteStudentZoneItemsEditPostRouteTag, "/website/student-zone/items/{id}/edit", handlers::student_zone::item_edit_post;
        get WebsiteStudentZoneItemsDeleteGetRouteTag, "/website/student-zone/items/{id}/delete", handlers::student_zone::item_delete_get, modal;
        post WebsiteStudentZoneItemsDeletePostRouteTag, "/website/student-zone/items/{id}/delete", bare handlers::student_zone::item_delete_post, fragment(StudentZoneItemDeleteModalKey);
        get WebsiteStudentZoneSectionsDetailRouteTag, "/website/student-zone/sections/{id}", handlers::student_zone::section_detail;
        get WebsiteStudentZoneSectionsEditGetRouteTag, "/website/student-zone/sections/{id}/edit", handlers::student_zone::section_edit_get, modal;
        post WebsiteStudentZoneSectionsEditPostRouteTag, "/website/student-zone/sections/{id}/edit", handlers::student_zone::section_edit_post;
        get WebsiteStudentZoneSectionsDeleteGetRouteTag, "/website/student-zone/sections/{id}/delete", handlers::student_zone::section_delete_get, modal;
        post WebsiteStudentZoneSectionsDeletePostRouteTag, "/website/student-zone/sections/{id}/delete", bare handlers::student_zone::section_delete_post, fragment(StudentZoneSectionDeleteModalKey);

        get WebsiteContactPageSettingsDetailRouteTag, "/website/contact-page/settings/{id}", handlers::contact_page::detail;
        get WebsiteContactPageSettingsEditGetRouteTag, "/website/contact-page/settings/{id}/edit", handlers::contact_page::edit_get;
        post WebsiteContactPageSettingsEditPostRouteTag, "/website/contact-page/settings/{id}/edit", handlers::contact_page::edit_post;

        get WebsiteTblfeeListRouteTag, "/website/tblfee", handlers::tblfee::list, fragment(TblfeeTableKey);
        post WebsiteTblfeeSyncRouteTag, "/website/tblfee/sync", handlers::tblfee::sync;
        get WebsiteTblfeeDetailRouteTag, "/website/tblfee/{id}", handlers::tblfee::detail;
    ]
}
