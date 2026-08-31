use lariv_rs::html_form::{
    html_form,
    widgets::{Number, Select, Text, Textarea},
};
use nirmancampus_courses::routes::CoursesMultiSelectRouteTag;

use super::routes::ProgramMediaMultiSelectRouteTag;

#[html_form]
pub struct ProgramForm {
    #[form(label = "Name", required, widget = Text)]
    pub name: String,

    #[form(label = "Code", required, widget = Text)]
    pub code: String,

    #[form(label = "Description", widget = Textarea, rows = 3)]
    pub description: String,

    #[form(label = "University", widget = Select, choices = "university")]
    pub university: String,

    #[form(label = "Program type", widget = Select, choices = "program_type")]
    pub program_type: String,

    #[form(
        label = "Admission sessions",
        widget = Select,
        choices = "admission_sessions"
    )]
    pub admission_sessions: String,

    #[form(label = "Term type", widget = Select, choices = "term_type")]
    pub term_type: String,

    #[form(label = "Fee (₹)", widget = Number)]
    pub fee: i64,

    #[form(
        label = "Media languages",
        widget = ManyToMany,
        route = ProgramMediaMultiSelectRouteTag,
        swap_key = "fk-program-media",
        placeholder = "Select instruction languages…"
    )]
    pub program_media: Vec<i64>,
}

#[html_form]
pub struct ProgramFilterForm {
    #[form(label = "Name", widget = Text)]
    pub name: String,

    #[form(label = "Code", widget = Text)]
    pub code: String,

    #[form(label = "University", widget = Select, choices = "university")]
    pub university: String,

    #[form(label = "Program type", widget = Select, choices = "program_type")]
    pub program_type: String,
}

#[html_form]
pub struct StructureUnitForm {
    #[form(label = "Term number", required, widget = Number)]
    pub term_number: i64,

    #[form(
        label = "Compulsory courses",
        widget = ManyToMany,
        route = CoursesMultiSelectRouteTag,
        swap_key = "fk-structure-unit-compulsory",
        placeholder = "Select compulsory courses…"
    )]
    pub compulsory_courses: Vec<i64>,

    #[form(label = "Optional course count", widget = Number)]
    pub optional_course_count: i64,

    #[form(
        label = "Optional course pool",
        widget = ManyToMany,
        route = CoursesMultiSelectRouteTag,
        swap_key = "fk-structure-unit-optional",
        placeholder = "Select optional pool courses…"
    )]
    pub optional_course_selection_pool: Vec<i64>,
}
