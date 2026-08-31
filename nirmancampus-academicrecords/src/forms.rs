use lariv_rs::html_form::{
    html_form,
    widgets::{Date, Select, Text},
};
use nirmancampus_courses::routes::CoursesMultiSelectRouteTag;
use nirmancampus_programs::routes::ProgramsSelectRouteTag;
use nirmancampus_sessions::routes::SessionsSelectRouteTag;
use nirmancampus_students::routes::StudentsSelectRouteTag;

use super::routes::AcademicRecordsPsuSelectRouteTag;

#[html_form]
pub struct AcademicRecordForm {
    #[form(
        label = "Admission session",
        required,
        widget = ForeignKey,
        route = SessionsSelectRouteTag,
        swap_key = "fk-academic-record-session",
        display = "session",
        placeholder = "Select an admission session…"
    )]
    pub session_id: i64,

    #[form(
        label = "Student",
        required,
        widget = ForeignKey,
        route = StudentsSelectRouteTag,
        swap_key = "fk-academic-record-student",
        display = "student",
        placeholder = "Select a student..."
    )]
    pub student_id: i64,

    #[form(
        label = "Program",
        required,
        widget = ForeignKey,
        route = ProgramsSelectRouteTag,
        swap_key = "fk-academic-record-program",
        display = "program",
        placeholder = "Select a program..."
    )]
    pub program_id: i64,

    #[form(label = "Status", widget = Select, choices = "status", required)]
    pub status: String,

    #[form(label = "Admission date", required, widget = Date)]
    pub date: String,

    #[form(
        label = "Term",
        required,
        widget = ForeignKey,
        route = AcademicRecordsPsuSelectRouteTag,
        swap_key = "fk-academic-record-psu",
        display = "term",
        placeholder = "Select a term..."
    )]
    pub program_structure_unit_id: i64,

    #[form(
        label = "Optional courses",
        widget = ManyToMany,
        route = CoursesMultiSelectRouteTag,
        swap_key = "fk-academic-record-optional-courses",
        placeholder = "Select optional courses from the program pool…"
    )]
    pub optional_courses: Vec<i64>,
}

#[html_form]
pub struct AcademicRecordFilterForm {
    #[form(label = "Status", widget = Select, choices = "status")]
    pub status: String,

    #[form(label = "Term", widget = Text)]
    pub term: String,

    #[form(
        label = "Program",
        widget = ForeignKey,
        route = ProgramsSelectRouteTag,
        swap_key = "fk-academic-record-filter-program",
        display = "program",
        placeholder = "Filter by program..."
    )]
    pub program_id: i64,
}
