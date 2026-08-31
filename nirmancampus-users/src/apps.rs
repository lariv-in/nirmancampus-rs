//! Re-register Users and OTP dashboard tiles for Nirmancampus admin visibility.

use lariv_rs::apps::{AppTile, AppsCapability, AppsRegistrar, PluginType};
use nirmancampus_common::ROLE_ADMIN;

#[derive(Clone, Copy, Default)]
pub struct Hook;

impl AppsRegistrar for Hook {
    fn register_apps(self, apps: AppsCapability) -> AppsCapability {
        apps.register(AppTile {
            key: "p_users".into(),
            verbose_name: "Users".into(),
            href: "/users".into(),
            icon: "users".into(),
            plugin_type: PluginType::App,
            roles: vec!["superuser".into(), ROLE_ADMIN.into()],
        })
        .register(AppTile {
            key: "p_otp".into(),
            verbose_name: "OTP Preferences".into(),
            href: "/otp/preferences".into(),
            icon: "key".into(),
            plugin_type: PluginType::App,
            roles: vec!["superuser".into(), ROLE_ADMIN.into()],
        })
        .register(AppTile {
            key: "p_filesystem".into(),
            verbose_name: "Files".into(),
            href: "/filesystem".into(),
            icon: "folder".into(),
            plugin_type: PluginType::App,
            roles: vec!["superuser".into(), ROLE_ADMIN.into()],
        })
    }
}
