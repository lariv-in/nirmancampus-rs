use lariv_rs::config::ConfigSection;

pub struct WebsiteConfigTag;

impl ConfigSection for WebsiteConfigTag {
    const KEY: Option<&'static str> = Some("p_nirmancampus_website");
}

#[derive(Clone, Debug, Default, serde::Deserialize)]
pub struct WebsiteConfig {
    /// Optional directory to import into the filesystem plugin under `website/static/`.
    /// `nirman_campus/` next to the process is also imported when present.
    #[serde(default, rename = "staticDir")]
    pub static_dir: String,
}
