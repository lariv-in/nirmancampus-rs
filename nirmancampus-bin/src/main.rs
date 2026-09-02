#![recursion_limit = "512"]

use lariv_rs::app::App;
use lariv_rs::plugins::{dashboard, filesystem, otp, pwa, users};
use tracing_subscriber::EnvFilter;

#[lariv_rs::main(
    stack_size = 64 * 1024 * 1024,
    thread_name = "nirmancampus-server"
)]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env().add_directive("warn".parse().expect("directive")),
        )
        .init();

    let app = App::new_web_app();
    let app = users::install(app);
    let app = otp::install(app);
    let app = pwa::install(app);
    let app = filesystem::install(app);
    let app = nirmancampus_users::install(app);
    let app = nirmancampus_courses::install(app);
    let app = nirmancampus_programs::install(app);
    let app = nirmancampus_sessions::install(app);
    let app = nirmancampus_students::install(app);
    let app = nirmancampus_academicrecords::install(app);
    let app = nirmancampus_announcements::install(app);
    let app = nirmancampus_studentapplications::install(app);
    let app = nirmancampus_studentpayments::install(app);
    let app = nirmancampus_studentfees::install(app);
    let app = nirmancampus_examregistrations::install(app);
    let app = nirmancampus_assignmentsubmissions::install(app);
    let app = dashboard::install(app);
    let app = nirmancampus_dashboard::install(app);
    let app = nirmancampus_website::install(app);

    let app = app.load_config("config.toml").await?;
    let app = app.mount();
    app.run().await?;
    Ok(())
}
