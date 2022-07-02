// Copyright (C) 2022 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use anyhow::anyhow;
use anyhow::ensure;
use anyhow::Context;
use anyhow::Result;
use versions::Version;

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

pub async fn docker_latest_version(path: &str) -> Result<Version> {
    let url = format!(
        "https://registry.hub.docker.com/v1/repositories/{}/tags",
        path
    );
    let data = get_json(&url).await?;
    let versions =
        parse_versions(data).with_context(|| format!("error processing json from {}", url))?;
    let latest = versions
        .into_iter()
        .max()
        .ok_or_else(|| anyhow!("no versions found"))?;
    Ok(latest)
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
