use super::keys::{
    ImportantLinkCreateModalKey, StudentZoneItemCreateModalKey, StudentZoneSectionCreateModalKey,
};
use super::routes::{
    WebsiteImportantLinksCreateGetRouteTag, WebsiteImportantLinksCreatePostRouteTag,
    WebsiteStudentZoneItemsCreateGetRouteTag, WebsiteStudentZoneItemsCreatePostRouteTag,
    WebsiteStudentZoneSectionsCreateGetRouteTag, WebsiteStudentZoneSectionsCreatePostRouteTag,
};

lariv_rs::impl_create_modal!(
    ImportantLinkCreateModalKey,
    WebsiteImportantLinksCreateGetRouteTag,
    WebsiteImportantLinksCreatePostRouteTag,
    "nirmancampus_website.ImportantLinksCreateForm"
);

lariv_rs::impl_create_modal!(
    StudentZoneSectionCreateModalKey,
    WebsiteStudentZoneSectionsCreateGetRouteTag,
    WebsiteStudentZoneSectionsCreatePostRouteTag,
    "nirmancampus_website.StudentZoneSectionCreateForm"
);

lariv_rs::impl_create_modal!(
    StudentZoneItemCreateModalKey,
    WebsiteStudentZoneItemsCreateGetRouteTag,
    WebsiteStudentZoneItemsCreatePostRouteTag,
    "nirmancampus_website.StudentZoneItemCreateForm"
);
