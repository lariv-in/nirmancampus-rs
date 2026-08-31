use lariv_rs::html_form::{
    html_form,
    widgets::{Checkbox, Date, Email, Phone, Select, Text, Textarea},
};
use lariv_rs::plugins::filesystem::routes::VNodeFileSelectRouteTag;

#[html_form]
pub struct StudentForm {
    #[form(label = "Name", required, widget = Text)]
    pub name: String,

    #[form(label = "Enrollment No / Control ID", required, widget = Text)]
    pub student_no: String,

    #[form(label = "Aadhar Card", widget = Text)]
    pub aadhar_card: String,

    #[form(label = "ABC ID", name = "ABCId", widget = Text)]
    pub abc_id: String,

    #[form(label = "Email", widget = Email)]
    pub email: String,

    #[form(label = "Phone", widget = Phone)]
    pub phone: String,

    #[form(label = "Date of Birth", required, widget = Date)]
    pub dob: String,

    #[form(label = "Mother's Name", widget = Text)]
    pub mother_name: String,

    #[form(label = "Father's Name", name = "FatherName", widget = Text)]
    pub fathers_name: String,

    #[form(label = "Category", widget = Select, choices = "category")]
    pub category: String,

    #[form(label = "Address", widget = Textarea, rows = 4)]
    pub address: String,

    #[form(label = "Remarks", widget = Textarea, rows = 4)]
    pub remarks: String,

    #[form(label = "Handicapped", widget = Checkbox)]
    pub handicapped: bool,

    #[form(
        label = "Photo",
        widget = ForeignKey,
        route = VNodeFileSelectRouteTag,
        swap_key = "student-photo",
        display = "photo",
        placeholder = "Select photo…"
    )]
    pub photo_id: String,

    #[form(
        label = "Documents",
        widget = ManyToMany,
        route = VNodeFileSelectRouteTag,
        swap_key = "student-documents",
        placeholder = "Select documents…"
    )]
    pub documents: Vec<i64>,
}

#[html_form]
pub struct StudentFilterForm {
    #[form(label = "Enrollment No / Control ID", widget = Text)]
    pub student_no: String,

    #[form(label = "Aadhar Card", widget = Text)]
    pub aadhar_card: String,

    #[form(label = "ABC ID", name = "ABCId", widget = Text)]
    pub abc_id: String,

    #[form(label = "Name", widget = Text)]
    pub name: String,

    #[form(label = "Email", widget = Text)]
    pub email: String,

    #[form(label = "Phone", widget = Text)]
    pub phone: String,

    #[form(label = "Mother's Name", widget = Text)]
    pub mother_name: String,

    #[form(label = "Father's Name", name = "FatherName", widget = Text)]
    pub fathers_name: String,

    #[form(label = "Category", widget = Text)]
    pub category: String,
}

#[html_form]
pub struct StudentSelectFilterForm {
    #[form(label = "Name", widget = Text)]
    pub name: String,

    #[form(label = "Enrollment No / Control ID", widget = Text)]
    pub student_no: String,

    #[form(label = "Phone", widget = Text)]
    pub phone: String,
}
