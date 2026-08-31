//! Nirmancampus dashboard handlers — unassigned shortcuts or apps launchpad.

use lariv_rs::{
    apps::{AppTile, AppsCapability, PluginType},
    components::{SharedChromeFolder, SlotCtx},
    http::Cap,
    plugins::users::middleware::RequireAuth,
    web::{Htmx, html_built_page_or_app_layout},
};
use nirmancampus_common::is_unassigned;

use crate::templates::NirmancampusAppsPage;

/// Nirmancampus dashboard visibility: empty `roles` means every authenticated
/// user. Non-empty `roles` are admin-only tiles.
///
/// Superusers see every app tile. This differs from Lariv's default
/// [`AppsCapability::visible_apps`], which treats empty roles as staff-only.
fn nirmancampus_visible_apps(
    catalog: &AppsCapability,
    role: &str,
    is_superuser: bool,
) -> Vec<AppTile> {
    let mut apps: Vec<_> = catalog
        .apps()
        .iter()
        .filter(|app| app.plugin_type == PluginType::App)
        .filter(|app| is_superuser || app.roles.is_empty() || app.roles.iter().any(|r| r == role))
        .cloned()
        .collect();
    apps.sort_by(|a, b| a.verbose_name.cmp(&b.verbose_name));
    apps
}

/// Apps launchpad, or unassigned-applicant actions.
pub async fn apps(
    Cap(catalog): Cap<AppsCapability>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
) -> maud::Markup {
    let apps = nirmancampus_visible_apps(&catalog, &ctx.role, ctx.user.is_superuser);
    let avatar = ctx
        .user
        .name
        .chars()
        .next()
        .map(|c| c.to_string())
        .unwrap_or_else(|| "?".into());
    let slot_ctx = SlotCtx::from_auth(&ctx);
    let page = NirmancampusAppsPage {
        name: ctx.user.name.clone(),
        role: ctx.role.clone(),
        avatar,
        is_superuser: ctx.user.is_superuser,
        is_unassigned: is_unassigned(&ctx),
        apps,
    };
    html_built_page_or_app_layout(&page, &htmx, &chrome, &slot_ctx)
}

#[cfg(test)]
mod tests {
    use super::*;
    use nirmancampus_common::{ROLE_ADMIN, ROLE_STUDENT};

    fn tile(key: &str, name: &str, plugin_type: PluginType, roles: &[&str]) -> AppTile {
        AppTile {
            key: key.into(),
            verbose_name: name.into(),
            href: format!("/{key}"),
            icon: "app".into(),
            plugin_type,
            roles: roles.iter().map(|r| (*r).into()).collect(),
        }
    }

    fn catalog() -> AppsCapability {
        AppsCapability::new()
            .register(tile(
                "p_nirmancampus_programs",
                "Programs",
                PluginType::App,
                &[],
            ))
            .register(tile(
                "p_nirmancampus_students",
                "Students",
                PluginType::App,
                &[],
            ))
            .register(tile(
                "p_nirmancampus_courses",
                "Courses",
                PluginType::Addon,
                &[],
            ))
            .register(tile(
                "p_nirmancampus_studentapplications",
                "Student Applications",
                PluginType::Addon,
                &["admin", "unassigned"],
            ))
            .register(tile(
                "p_users",
                "Users",
                PluginType::App,
                &["superuser", ROLE_ADMIN],
            ))
            .register(tile(
                "p_nirmancampus_website",
                "Website",
                PluginType::App,
                &["superuser", ROLE_ADMIN],
            ))
    }

    fn keys(role: &str, is_superuser: bool) -> Vec<String> {
        nirmancampus_visible_apps(&catalog(), role, is_superuser)
            .into_iter()
            .map(|a| a.verbose_name)
            .collect()
    }

    #[test]
    fn student_sees_core_apps_not_admin_tiles() {
        assert_eq!(
            keys(ROLE_STUDENT, false),
            vec!["Programs".to_string(), "Students".to_string()]
        );
    }

    #[test]
    fn admin_sees_core_and_admin_tiles() {
        assert_eq!(
            keys(ROLE_ADMIN, false),
            vec![
                "Programs".to_string(),
                "Students".to_string(),
                "Users".to_string(),
                "Website".to_string(),
            ]
        );
    }

    #[test]
    fn superuser_sees_all_app_tiles_but_not_addons() {
        assert_eq!(
            keys(ROLE_STUDENT, true),
            vec![
                "Programs".to_string(),
                "Students".to_string(),
                "Users".to_string(),
                "Website".to_string(),
            ]
        );
    }
}
