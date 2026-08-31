use lariv_rs::html_form::{
    html_form,
    widgets::{Number, Select, Text},
};
use lariv_rs::plugins::filesystem::routes::VNodeFileSelectRouteTag;
use nirmancampus_academicrecords::routes::AcademicRecordsSelectRouteTag;
use nirmancampus_courses::routes::CoursesSelectRouteTag;

#[html_form]
pub struct AssignmentForm {
    #[form(label = "Assignment title", required, widget = Text)]
    pub assignment_title: String,

    #[form(label = "Submission status", widget = Select, choices = "status", required)]
    pub submission_status: String,

    #[form(label = "Max marks", required, widget = Number)]
    pub max_marks: i64,

    #[form(label = "Marks", required, widget = Number)]
    pub marks: i64,

    #[form(
        label = "Course",
        required,
        widget = ForeignKey,
        route = CoursesSelectRouteTag,
        swap_key = "fk-assignment-course",
        display = "course",
        placeholder = "Select a course..."
    )]
    pub course_id: i64,

    #[form(
        label = "Academic record",
        required,
        widget = ForeignKey,
        route = AcademicRecordsSelectRouteTag,
        swap_key = "fk-assignment-academic-record",
        display = "academic_record",
        placeholder = "Select an academic record..."
    )]
    pub academic_record_id: i64,

    #[form(
        label = "Assets",
        widget = ManyToMany,
        route = VNodeFileSelectRouteTag,
        swap_key = "fk-assignment-assets",
        placeholder = "Select files…"
    )]
    pub assets: Vec<i64>,
}

#[html_form]
pub struct AssignmentFilterForm {
    #[form(label = "Assignment title", widget = Text)]
    pub assignment_title: String,

    #[form(label = "Submission status", widget = Select, choices = "status")]
    pub submission_status: String,

    #[form(
        label = "Academic record",
        widget = ForeignKey,
        route = AcademicRecordsSelectRouteTag,
        swap_key = "fk-assignment-filter-academic-record",
        display = "academic_record",
        placeholder = "Filter by academic record..."
    )]
    pub academic_record_id: i64,
}
