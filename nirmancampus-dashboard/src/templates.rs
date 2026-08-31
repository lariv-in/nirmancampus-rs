//! Nirmancampus dashboard — unassigned actions or core vs admin apps grid.

use frunk::Generic;
use maud::{Markup, PreEscaped, html};

use lariv_rs::{
    apps::AppTile,
    components::{
        ButtonLink, ShellChrome, ShellTopbar as ShellTopbarScaffold, button_link,
        dashboard_app_href, hx_nav_app_layout_for_url, icon, shell_topbar,
    },
    http::ProvideRequestCaps,
    plugins::dashboard::templates::DashboardAppsPageTag,
    template::{RenderAppPane, RenderTemplate, TemplateCapability, TemplateOf, TemplateRegistrar},
    traits::{get::IndexOfTemplateTag, replace::MapByTag},
};

#[derive(Generic)]
pub struct NirmancampusAppsPage {
    pub name: String,
    pub role: String,
    pub avatar: String,
    pub is_superuser: bool,
    pub is_unassigned: bool,
    pub apps: Vec<AppTile>,
}

impl NirmancampusAppsPage {
    fn split_apps(&self) -> (Vec<&AppTile>, Vec<&AppTile>) {
        // Empty roles = core (students + admins). Non-empty roles = Admin only.
        let mut core = Vec::new();
        let mut admin = Vec::new();
        for app in &self.apps {
            if app.roles.is_empty() {
                core.push(app);
            } else {
                admin.push(app);
            }
        }
        (core, admin)
    }

    fn app_tile_grid(apps: &[&AppTile]) -> Markup {
        html! {
            (PreEscaped(r##"<div class="grid grid-cols-2 @md:grid-cols-4 @2xl:grid-cols-6 gap-2">"##))
            @for app in apps {
                (PreEscaped({
                    let href = dashboard_app_href(&app.href);
                    format!(
                        r##"<a href="{href}" class="btn btn-md h-auto flex-col space-y-1 py-4" x-show="'{name}'.toLowerCase().includes(search.toLowerCase())" x-cloak{hx}>"##,
                        href = html_escape_attr(&href),
                        name = html_escape_js(&app.verbose_name),
                        hx = hx_nav_app_layout_for_url(&href).as_string(),
                    )
                }))
                (icon(&app.icon, "w-8 h-8"))
                div class="text-sm truncate min-w-0 w-full" { (app.verbose_name.as_str()) }
                (PreEscaped("</a>"))
            }
            (PreEscaped("</div>"))
        }
    }

    fn unassigned_body(&self) -> Markup {
        html! {
            div class="container max-w-5xl mx-auto mt-4" {
                div class="flex flex-col gap-4" {
                    h1 class="text-3xl font-bold mb-8" { "Hello " (self.name) }
                    div class="flex flex-wrap gap-3" {
                        (button_link(ButtonLink {
                            label: "Create application",
                            href: "/student-applications/create/",
                            icon_name: Some("plus"),
                            classes: "btn-primary",
                            ..Default::default()
                        }))
                        (button_link(ButtonLink {
                            label: "View your applications",
                            href: "/student-applications/",
                            icon_name: Some("document-text"),
                            classes: "btn-outline",
                            ..Default::default()
                        }))
                    }
                }
            }
        }
    }

    fn apps_grid_body(&self) -> Markup {
        let (core, admin) = self.split_apps();
        html! {
            (PreEscaped(
                r##"<div class="container max-w-5xl mx-auto mt-4 @container" x-data="{ search: '' }">"##,
            ))
            div class="mb-4" {
                (PreEscaped(
                    r##"<input type="text" x-model="search" placeholder="Search apps..." class="input input-bordered w-full">"##,
                ))
            }
            (Self::app_tile_grid(&core))
            @if !admin.is_empty() {
                div class="mt-8 mb-4" {
                    h2 class="text-lg font-semibold" { "Admin only" }
                }
                (Self::app_tile_grid(&admin))
            }
            (PreEscaped("</div>"))
        }
    }

    fn pane_body(&self) -> Markup {
        if self.is_unassigned {
            self.unassigned_body()
        } else {
            self.apps_grid_body()
        }
    }
}

fn html_escape_attr(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
}

fn html_escape_js(value: &str) -> String {
    value.replace('\\', "\\\\").replace('\'', "\\'")
}

impl RenderAppPane for NirmancampusAppsPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        use lariv_rs::components::app_layout_pane;
        app_layout_pane(self.pane_body())
    }

    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        use lariv_rs::components::{LayoutMain, layout_main};
        layout_main(LayoutMain {
            breadcrumbs: Markup::default(),
            content: self.pane_body(),
        })
    }
}

impl RenderTemplate for NirmancampusAppsPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        shell_topbar(ShellTopbarScaffold {
            title: "Nirman Campus",
            registry_head: chrome.head.clone(),
            topbar_items: chrome.topbar_items.clone(),
            body: self.pane_body(),
            ..Default::default()
        })
    }
}

#[derive(Copy, Clone)]
pub struct Hook<AppsIdx>(std::marker::PhantomData<AppsIdx>);

impl<AppsIdx> Default for Hook<AppsIdx> {
    fn default() -> Self {
        Hook(std::marker::PhantomData)
    }
}

type AppsReplaced<T, AppsIdx> =
    <T as MapByTag<DashboardAppsPageTag, TemplateOf<NirmancampusAppsPage>, AppsIdx>>::Output;

impl<T, AppsIdx> TemplateRegistrar<T> for Hook<AppsIdx>
where
    T: frunk::hlist::HList + Clone + ProvideRequestCaps + Send + Sync,
    T: IndexOfTemplateTag<DashboardAppsPageTag, AppsIdx>,
    T: MapByTag<DashboardAppsPageTag, TemplateOf<NirmancampusAppsPage>, AppsIdx>,
{
    type Output = AppsReplaced<T, AppsIdx>;

    fn register_templates(self, cap: TemplateCapability<T>) -> TemplateCapability<Self::Output> {
        cap.replace_template_tag::<DashboardAppsPageTag, TemplateOf<NirmancampusAppsPage>, AppsIdx>(
            |_| TemplateOf::new(),
        )
    }
}
