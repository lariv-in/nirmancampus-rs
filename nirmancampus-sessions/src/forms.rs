use lariv_rs::html_form::{
    html_form,
    widgets::{Checkbox, Date, Text},
};

#[html_form]
pub struct SessionForm {
    #[form(label = "Name", required, widget = Text)]
    pub name: String,

    #[form(label = "Code", widget = Text)]
    pub code: String,

    #[form(label = "Start", required, widget = Date)]
    pub start: String,

    #[form(label = "End", required, widget = Date)]
    pub end: String,

    #[form(label = "Active", widget = Checkbox)]
    pub is_active: bool,
}

#[html_form]
pub struct SessionFilterForm {
    #[form(label = "Name", widget = Text)]
    pub name: String,

    #[form(label = "Code", widget = Text)]
    pub code: String,
}
