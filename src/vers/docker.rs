// Copyright (C) 2022 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use anyhow::anyhow;
use anyhow::ensure;
use anyhow::Context;
use anyhow::Result;
use async_trait::async_trait;
use tracing::event;
use tracing::instrument;
use tracing::Level;
use versions::Version;

use crate::entity::Entity;
use crate::vers::updater;
use crate::vers::Versions;

#[derive(Debug, Default)]
pub struct Docker {}

#[async_trait]
impl updater::Updater for Docker {
    fn url(&self, resource: &str) -> Option<String> {
        url(resource)
    }

    async fn get_versions(&self, url: &str) -> Result<Vec<Version>> {
        get_versions(url).await
    }

    fn updated_line(&self, entity: &Entity) -> Option<String> {
        entity
            .latest
            .as_ref()
            .map(|v| format!("{}:{}", entity.resource, v))
    }
}

#[instrument(level = "debug")]
pub fn url(resource: &str) -> Option<String> {
    resource.strip_prefix("docker://").map(|path| {
        format!(
            "https://registry.hub.docker.com/v1/repositories/{}/tags",
            path
        )
    })
}

#[instrument(level = "debug")]
async fn get_json(url: &str) -> Result<serde_json::Value> {
    let response = reqwest::get(url).await?;
    ensure!(
        response.status().is_success(),
        anyhow!(format!("{} while getting {}", response.status(), url))
    );
    response
        .json::<serde_json::Value>()
        .await
        .with_context(|| format!("error parsing json in {}", url))
}

#[instrument(level = "debug")]
fn parse_versions(data: serde_json::Value) -> Result<Vec<Version>> {
    data.as_array()
        .ok_or_else(|| anyhow!("invalid type for layer object list"))?
        .iter()
        .map(|layer| {
            layer
                .as_object()
                .ok_or_else(|| anyhow!("invalid type for layer object"))?
                .get("name")
                .ok_or_else(|| anyhow!("\"name\" field not found in layer object"))
                .map(|version_value| {
                    let version_str = version_value.as_str().ok_or_else(|| {
                        anyhow!("invalid type for \"name\" field in layer object")
                    })?;
                    Version::new(version_str).ok_or_else(|| anyhow!("unable to parse version"))
                })?
        })
        .collect::<Result<Vec<Version>>>()
}

#[instrument(level = "debug")]
pub async fn get_versions(url: &str) -> Result<Vec<Version>> {
    let data = get_json(url).await?;
    let versions =
        parse_versions(data).with_context(|| format!("error processing json from {}", url))?;
    event!(
        Level::INFO,
        versions = ?Versions::new(&versions),
        "parsed versions"
    );
    Ok(versions)
}

#[test]
fn test_docker_parse_versions() -> Result<()> {
    let json_str = r#"[{"layer": "", "name": "latest"}, {"layer": "", "name": "0.2"}, {"layer": "", "name": "0.3"}, {"layer": "", "name": "0.4"}, {"layer": "", "name": "0.6"}, {"layer": "", "name": "0.7"}, {"layer": "", "name": "0.8.0"}, {"layer": "", "name": "0.9.0"}]"#;
    let json_value: serde_json::Value = serde_json::from_str(json_str)?;
    let versions = parse_versions(json_value)?
        .into_iter()
        .map(|v| format!("{}", v))
        .collect::<Vec<_>>();
    assert_eq!(
        versions,
        ["latest", "0.2", "0.3", "0.4", "0.6", "0.7", "0.8.0", "0.9.0"]
    );
    Ok(())
}
