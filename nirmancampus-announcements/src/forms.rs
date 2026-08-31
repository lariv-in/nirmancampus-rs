use lariv_rs::html_form::{
    html_form,
    widgets::{Datetime, Text, Textarea},
};
use lariv_rs::plugins::filesystem::routes::VNodeFileSelectRouteTag;

#[html_form]
pub struct AnnouncementForm {
    #[form(label = "Title", required, widget = Text)]
    pub title: String,

    #[form(label = "Description", widget = Textarea, rows = 4)]
    pub description: String,

    #[form(label = "URL", widget = Text)]
    pub url: String,

    #[form(label = "Release At", widget = Datetime)]
    pub release_at: String,

    #[form(label = "Expiry At", widget = Datetime)]
    pub expiry_at: String,

    #[form(
        label = "Assets",
        widget = ManyToMany,
        route = VNodeFileSelectRouteTag,
        swap_key = "fk-announcement-assets",
        placeholder = "Select files…"
    )]
    pub assets: Vec<i64>,
}

#[html_form]
pub struct AnnouncementFilterForm {
    #[form(label = "Title", widget = Text)]
    pub title: String,

    #[form(label = "Description", widget = Text)]
    pub description: String,
}
