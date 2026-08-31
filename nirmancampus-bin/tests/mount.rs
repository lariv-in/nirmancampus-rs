//! Compile smoke test for the Nirmancampus deployment plugin stack.

#![recursion_limit = "512"]

use std::path::PathBuf;

use lariv_rs::app::App;
use lariv_rs::plugins::{dashboard, filesystem, otp, pwa, users};

const STACK_SIZE: usize = 64 * 1024 * 1024;

const MINIMAL_DB_TOML: &str = r#"database_url = "sqlite::memory:"
[users]
adminEmail = "admin@test.local"
adminPassword = "adminadmin"
signingKey = "dGVzdC1zaWduaW5nLWtleS1wYWRkZWQtdG8tNjQtYnl0ZXMhISEhISEhISEhISE="
jwtIssuer = "bGlybWFuY2FtcHVzLXRlc3QtaXNzdWVyLXBhZGRlZC10by02NC1ieXRlcyE="
staffRoles = ["admin"]
"#;

fn temp_config(body: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "nirmancampus-mount-{}-{}.toml",
        std::process::id(),
        uuid::Uuid::new_v4()
    ));
    std::fs::write(&path, body).expect("write temp config");
    path
}

fn run_on_large_stack<F, Fut>(name: &str, f: F)
where
    F: FnOnce() -> Fut + Send + 'static,
    Fut: std::future::Future<Output = ()>,
{
    std::thread::Builder::new()
        .name(name.into())
        .stack_size(STACK_SIZE)
        .spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("tokio runtime");
            rt.block_on(f());
        })
        .expect("spawn mount thread")
        .join()
        .expect("mount thread");
}

macro_rules! install_nirmancampus {
    () => {{
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
        let app = nirmancampus_examregistrations::install(app);
        let app = nirmancampus_assignmentsubmissions::install(app);
        let app = dashboard::install(app);
        let app = nirmancampus_dashboard::install(app);
        nirmancampus_website::install(app)
    }};
}

#[test]
fn nirmancampus_stack_mounts() {
    run_on_large_stack("nirmancampus-mount", || async {
        let app = install_nirmancampus!();
        let path = temp_config(MINIMAL_DB_TOML);
        let app = app.load_config(&path).await.expect("load_config");
        std::fs::remove_file(&path).ok();
        let _mounted = app.mount();
    });
}

#[test]
fn mark_migrations_records_versions_without_ddl() {
    run_on_large_stack("nirmancampus-mark-migrations", || async {
        let app = install_nirmancampus!();
        let path = temp_config(MINIMAL_DB_TOML);
        let app = app.load_config(&path).await.expect("load_config");
        std::fs::remove_file(&path).ok();
        let mounted = app.mount();

        let inserted = mounted.mark_migrations().await.expect("mark_migrations");
        assert!(
            inserted >= 9,
            "expected Lariv + Nirmancampus migration versions"
        );

        mounted
            .run_migrations()
            .await
            .expect("migrate should be a no-op after mark");
    });
}
