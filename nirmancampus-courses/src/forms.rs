use lariv_rs::html_form::{
    html_form,
    widgets::{Checkbox, Number, Text, Textarea},
};

#[html_form]
pub struct CourseForm {
    #[form(label = "Course Name", required, widget = Text)]
    pub name: String,

    #[form(label = "Code", required, widget = Text)]
    pub code: String,

    #[form(label = "Type", widget = Text)]
    pub course_type: String,

    #[form(label = "Fee (₹)", widget = Number)]
    pub fee: i64,

    #[form(label = "Active", widget = Checkbox)]
    pub is_active: bool,

    #[form(label = "Description", widget = Textarea, rows = 3)]
    pub description: String,
}

#[html_form]
pub struct CourseFilterForm {
    #[form(label = "Name", widget = Text)]
    pub name: String,

    #[form(label = "Code", widget = Text)]
    pub code: String,

    #[form(label = "Type", widget = Text)]
    pub course_type: String,
}
