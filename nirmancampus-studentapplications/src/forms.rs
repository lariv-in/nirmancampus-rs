use lariv_rs::html_form::{
    html_form,
    widgets::{Date, ForeignKey, Phone, Select, Text, Textarea},
};
use lariv_rs::plugins::filesystem::routes::VNodeFileSelectRouteTag;

#[html_form]
pub struct ApplicationForm {
    #[form(
        label = "Program",
        required,
        widget = ForeignKey,
        url = "/programs/select/",
        swap_key = "fk-application-program",
        display = "program",
        placeholder = "Select a program..."
    )]
    pub program_id: i64,

    #[form(label = "Student name", required, widget = Text)]
    pub student_name: String,

    #[form(label = "Date of birth", widget = Date)]
    pub dob: String,

    #[form(label = "Mother name", widget = Text)]
    pub mother_name: String,

    #[form(label = "Father name", widget = Text)]
    pub father_name: String,

    #[form(label = "Category", widget = Select, choices = "category")]
    pub category: String,

    #[form(label = "Mobile", widget = Phone)]
    pub mobile: String,

    #[form(label = "Email", widget = Text)]
    pub email: String,

    #[form(label = "Address", widget = Textarea, rows = 4)]
    pub address: String,

    #[form(
        label = "Photo",
        widget = ForeignKey,
        route = VNodeFileSelectRouteTag,
        swap_key = "fk-application-photo",
        display = "photo",
        placeholder = "Select a photo…"
    )]
    pub photo_id: i64,

    #[form(
        label = "Documents",
        widget = ManyToMany,
        route = VNodeFileSelectRouteTag,
        swap_key = "fk-application-documents",
        placeholder = "Select documents…"
    )]
    pub documents: Vec<i64>,
}

#[html_form]
pub struct ApplicationFilterForm {
    #[form(label = "Email", widget = Text)]
    pub email: String,

    #[form(label = "Student name", widget = Text)]
    pub student_name: String,

    #[form(label = "Mobile", widget = Text)]
    pub mobile: String,
}
