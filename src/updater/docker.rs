// Copyright (C) 2022 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use tracing::instrument;

use crate::error::Error;
use crate::error::Result;
use crate::version::Version;

#[instrument(level = "debug")]
async fn get_json(url: &str) -> Result<serde_json::Value> {
    let response = reqwest::get(url).await?;
    if !response.status().is_success() {
        return Err(Error::HttpError(url.into(), response.status()));
    }
    Ok(response.json::<serde_json::Value>().await?)
}

#[instrument(level = "debug")]
fn parse_versions(data: serde_json::Value) -> Result<Vec<Version>> {
    data.as_array()
        .ok_or_else(|| Error::JsonParsing("invalid type for layer object list".into()))?
        .iter()
        .map(|layer| {
            layer
                .as_object()
                .ok_or_else(|| Error::JsonParsing("invalid type for layer object".into()))?
                .get("name")
                .ok_or_else(|| {
                    Error::JsonParsing("\"name\" field not found in layer object".into())
                })
                .map(|version_value| {
                    let version_str = version_value.as_str().ok_or_else(|| {
                        Error::JsonParsing("invalid type for \"name\" field in layer object".into())
                    })?;
                    Version::new(version_str)
                        .ok_or_else(|| Error::VersionParsing(version_str.into()))
                })?
        })
        .collect::<Result<Vec<Version>>>()
}

#[instrument(level = "debug")]
pub async fn get_versions(url: &str) -> Result<Vec<Version>> {
    let data = get_json(url).await?;
    let versions = parse_versions(data)?;
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
