use lariv_rs::html_form::{
    html_form,
    widgets::{Number, Select, Text},
};
use lariv_rs::plugins::filesystem::routes::VNodeFileSelectRouteTag;
use nirmancampus_academicrecords::routes::AcademicRecordsSelectRouteTag;
use nirmancampus_courses::routes::CoursesSelectRouteTag;

#[html_form]
pub struct ExamForm {
    #[form(label = "Exam title", required, widget = Text)]
    pub exam_title: String,

    #[form(label = "Registration status", widget = Select, choices = "status", required)]
    pub registration_status: String,

    #[form(label = "Max marks", required, widget = Number)]
    pub max_marks: i64,

    #[form(label = "Marks", required, widget = Number)]
    pub marks: i64,

    #[form(label = "Fee (₹)", widget = Number)]
    pub fee: i64,

    #[form(
        label = "Course",
        required,
        widget = ForeignKey,
        route = CoursesSelectRouteTag,
        swap_key = "fk-exam-course",
        display = "course",
        placeholder = "Select a course..."
    )]
    pub course_id: i64,

    #[form(
        label = "Academic record",
        required,
        widget = ForeignKey,
        route = AcademicRecordsSelectRouteTag,
        swap_key = "fk-exam-academic-record",
        display = "academic_record",
        placeholder = "Select an academic record..."
    )]
    pub academic_record_id: i64,

    #[form(
        label = "Assets",
        widget = ManyToMany,
        route = VNodeFileSelectRouteTag,
        swap_key = "fk-exam-assets",
        placeholder = "Select files…"
    )]
    pub assets: Vec<i64>,
}

#[html_form]
pub struct ExamFilterForm {
    #[form(label = "Exam title", widget = Text)]
    pub exam_title: String,

    #[form(label = "Registration status", widget = Select, choices = "status")]
    pub registration_status: String,

    #[form(
        label = "Academic record",
        widget = ForeignKey,
        route = AcademicRecordsSelectRouteTag,
        swap_key = "fk-exam-filter-academic-record",
        display = "academic_record",
        placeholder = "Filter by academic record..."
    )]
    pub academic_record_id: i64,
}
