use lariv_rs::html_form::{
    Upload, html_form,
    widgets::{Checkbox, Date, File, Number, Password, Text, Textarea},
};

#[derive(Clone)]
#[html_form(default)]
pub struct FeeForm {
    #[form(label = "Receipt ID", required, widget = Number)]
    pub id: i64,

    #[form(label = "Admission session", widget = Text)]
    pub adm_session: String,

    #[form(label = "Admission year", widget = Text)]
    pub adm_year: String,

    #[form(label = "Date of deposit", widget = Date)]
    pub dod: String,

    #[form(label = "Submit type", widget = Text)]
    pub submit: String,

    #[form(label = "Program", widget = Text)]
    pub prog: String,

    #[form(label = "Enrollment", widget = Text)]
    pub enroll: String,

    #[form(label = "Student name", widget = Text)]
    pub student: String,

    #[form(label = "Year/Sem", widget = Text)]
    pub year_sem: String,

    #[form(label = "Category", widget = Text)]
    pub category: String,

    #[form(label = "Date of birth", widget = Text)]
    pub dob: String,

    #[form(label = "Mobile", widget = Text)]
    pub contact: String,

    #[form(label = "Deposit", widget = Text)]
    pub deposit: String,

    #[form(label = "NSD", widget = Text)]
    pub nsd: String,

    #[form(label = "Fee", widget = Text)]
    pub fee: String,

    #[form(label = "Courses", widget = Text)]
    pub courses: String,

    #[form(label = "Remarks", widget = Textarea, rows = 3)]
    pub remarks: String,

    #[form(label = "Deposit by", widget = Text)]
    pub deposit_by: String,

    #[form(label = "TS", widget = Text)]
    pub ts: String,

    #[form(label = "Medium", widget = Text)]
    pub medium: String,

    #[form(label = "Mother", widget = Text)]
    pub mother_name: String,

    #[form(label = "Father", widget = Text)]
    pub father_name: String,

    #[form(label = "Username", widget = Text)]
    pub username: String,

    #[form(label = "Control ID", widget = Text)]
    pub control_id: String,

    #[form(label = "Discrepancy", widget = Text)]
    pub descrepency: String,

    #[form(label = "University", widget = Text)]
    pub university: String,

    #[form(label = "Payment mode", widget = Text)]
    pub payment_mode: String,

    #[form(label = "Trans ID", widget = Text)]
    pub trans_id: String,

    #[form(label = "Bank", widget = Text)]
    pub bank: String,

    #[form(label = "RM", widget = Text)]
    pub rm: String,

    #[form(label = "Reconciled", widget = Checkbox)]
    pub is_reconciled: bool,

    #[form(label = "Online exported", widget = Checkbox)]
    pub online_exported: bool,
}

#[html_form]
pub struct FeeFilterForm {
    #[form(label = "Search", widget = Text)]
    pub search: String,
}

#[html_form(default)]
pub struct PreferencesForm {
    #[form(label = "Host", required, widget = Text)]
    pub host: String,

    #[form(label = "Port", required, widget = Number)]
    pub port: i64,

    #[form(label = "Username", required, widget = Text)]
    pub username: String,

    #[form(label = "Password", widget = Password)]
    pub password: String,

    #[form(label = "Database", required, widget = Text)]
    pub database: String,
}

#[html_form(default)]
pub struct FeeUploadForm {
    #[form(label = "Excel file (.xlsx)", widget = File, accept = ".xlsx", required)]
    pub file: Upload,
}
