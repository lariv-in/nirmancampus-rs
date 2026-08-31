use lariv_rs::html_form::{
    Upload, html_form,
    widgets::{Checkbox, File, Number, Text},
};
use lariv_rs::plugins::filesystem::routes::VNodeFileSelectRouteTag;

use crate::routes::WebsiteStudentZoneSectionsSelectRouteTag;

#[html_form]
pub struct ImportantLinkForm {
    #[form(label = "Title", required, widget = Text)]
    pub title: String,

    #[form(label = "Order", widget = Number)]
    pub order: i64,

    #[form(label = "Is link", widget = Checkbox)]
    pub is_link: bool,

    #[form(label = "Link URL", widget = Text)]
    pub link: String,

    #[form(
        label = "File",
        widget = ForeignKey,
        route = VNodeFileSelectRouteTag,
        swap_key = "fk-website-important-link-file",
        display = "file",
        placeholder = "Select a file…"
    )]
    pub file_id: i64,
}

#[html_form]
pub struct ImportantLinkFilterForm {
    #[form(label = "Title", widget = Text)]
    pub title: String,
}

#[html_form]
pub struct StudentZoneSectionForm {
    #[form(label = "Title", required, widget = Text)]
    pub title: String,

    #[form(label = "Order", widget = Number)]
    pub order: i64,
}

#[html_form]
pub struct StudentZoneSectionFilterForm {
    #[form(label = "Title", widget = Text)]
    pub title: String,
}

#[html_form]
pub struct StudentZoneItemForm {
    #[form(label = "Title", required, widget = Text)]
    pub title: String,

    #[form(label = "Is link", widget = Checkbox)]
    pub is_link: bool,

    #[form(label = "Link URL", widget = Text)]
    pub link: String,

    #[form(
        label = "File",
        widget = ForeignKey,
        route = VNodeFileSelectRouteTag,
        swap_key = "fk-website-student-zone-item-file",
        display = "file",
        placeholder = "Select a file…"
    )]
    pub file_id: i64,

    #[form(
        label = "Section",
        required,
        widget = ForeignKey,
        route = WebsiteStudentZoneSectionsSelectRouteTag,
        swap_key = "fk-website-student-zone-section",
        display = "section",
        placeholder = "Select a section…"
    )]
    pub student_zone_section_id: i64,
}

#[html_form]
pub struct StudentZoneItemFilterForm {
    #[form(label = "Title", widget = Text)]
    pub title: String,
}

#[html_form]
pub struct ContactPageSettingsForm {
    #[form(
        label = "Essential committees list (PDF)",
        widget = ForeignKey,
        route = VNodeFileSelectRouteTag,
        swap_key = "fk-website-committees-file",
        display = "file",
        placeholder = "Select a PDF…"
    )]
    pub essential_committees_list_file_id: i64,
}

#[html_form]
pub struct TblfeeFilterForm {
    #[form(label = "Search", widget = Text)]
    pub search: String,
}

#[html_form(default)]
pub struct TblfeeUploadForm {
    #[form(label = "Excel file (.xlsx)", widget = File, accept = ".xlsx", required)]
    pub file: Upload,
}
