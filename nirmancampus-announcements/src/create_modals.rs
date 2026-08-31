use super::keys::AnnouncementCreateModalKey;
use super::routes::{AnnouncementsCreateGetRouteTag, AnnouncementsCreatePostRouteTag};

lariv_rs::impl_create_modal!(
    AnnouncementCreateModalKey,
    AnnouncementsCreateGetRouteTag,
    AnnouncementsCreatePostRouteTag,
    "announcements.AnnouncementCreateForm"
);
