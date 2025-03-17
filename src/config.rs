use config::{ Config, ConfigError };
use serde::Deserialize;

use crate::widgets::pull_requests;

#[derive(Debug, Deserialize, Clone, Default)]
pub struct Settings {
    pub github: GitHubSettings,
}

#[derive(Debug, Deserialize, Clone)]
pub enum PRFilter {
    ReviewRequested,
    Mentions,
    Labels,
    Assigned,
    Created,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct GitHubSettings {
    pub github_token: Option<String>,
    pub repos: Vec<pull_requests::Repo>,
    pub labels: Vec<String>,
    pub filters: Vec<PRFilter>,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let s = Config::builder()
            .add_source(config::File::with_name("config"))
            // .add_source(config::File::with_name(xdg::BaseDirectories::with_prefix("productui")))
            .add_source(config::Environment::with_prefix("PRODUCTUI"))
            .build()
            .unwrap();

        s.try_deserialize()
    }
}
