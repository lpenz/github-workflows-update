// Copyright (C) 2022 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use async_trait::async_trait;
use reqwest::header::USER_AGENT;
use tracing::instrument;

use crate::entity::Entity;
use crate::error::Error;
use crate::error::Result;
use crate::updater;
use crate::version::Version;

#[derive(Debug)]
pub struct Github {
    re_ref: regex::Regex,
}

impl Default for Github {
    fn default() -> Github {
        Github {
            re_ref: regex::Regex::new(r"^refs/tags/(?P<version>.+)$").unwrap(),
        }
    }
}

#[async_trait]
impl updater::Updater for Github {
    fn url(&self, resource: &str) -> Option<String> {
        url(resource)
    }

    async fn get_versions(&self, url: &str) -> Result<Vec<Version>> {
        get_versions(self, url).await
    }

    fn updated_line(&self, entity: &Entity) -> Option<String> {
        let rstr = entity.resource.to_string();
        let path = rstr.strip_prefix("github://")?;
        entity.latest.as_ref().map(|v| format!("{}@{}", path, v))
    }
}

#[instrument(level = "debug")]
pub fn url(resource: &str) -> Option<String> {
    resource.strip_prefix("github://").map(|path| {
        format!(
            "https://api.github.com/repos/{}/git/matching-refs/tags",
            path
        )
    })
}

#[instrument(level = "debug")]
async fn get_json(url: &str) -> Result<serde_json::Value> {
    let client = reqwest::Client::new();
    let mut builder = client.get(url);
    builder = builder.header(USER_AGENT, "reqwest");
    builder = builder.header("Accept", "application/vnd.github.v3+json");
    if let Ok(token) = std::env::var("PERSONAL_TOKEN") {
        builder = builder.header("Authorization", format!("token {}", token));
    }
    let response = builder.send().await?;
    if !response.status().is_success() {
        return Err(Error::HttpError(url.into(), response.status()));
    }
    Ok(response.json::<serde_json::Value>().await?)
}

#[instrument(level = "debug")]
fn parse_versions(github: &Github, data: serde_json::Value) -> Result<Vec<Version>> {
    data.as_array()
        .ok_or_else(|| Error::JsonParsing("invalid type for layer object list".into()))?
        .iter()
        .map(|tag_obj| {
            tag_obj
                .as_object()
                .ok_or_else(|| Error::JsonParsing("invalid type for tag object".into()))?
                .get("ref")
                .ok_or_else(|| Error::JsonParsing("ref field not found in tag object".into()))
                .map(|ref_value| {
                    let version_str = ref_value.as_str().ok_or_else(|| {
                        Error::JsonParsing("invalid type for ref field in tag object".into())
                    })?;
                    let m = github.re_ref.captures(version_str).ok_or_else(|| {
                        Error::JsonParsing(format!(
                            "could not match github ref {} to tag regex",
                            version_str
                        ))
                    })?;
                    let version_str = m.name("version").unwrap().as_str();
                    Version::new(version_str)
                        .ok_or_else(|| Error::VersionParsing(version_str.into()))
                })?
        })
        .collect::<Result<Vec<Version>>>()
}

#[instrument(level = "debug")]
pub async fn get_versions(github: &Github, url: &str) -> Result<Vec<Version>> {
    let data = get_json(url).await?;
    let versions = parse_versions(github, data)?;
    Ok(versions)
}

#[test]
fn test_docker_parse_versions() -> Result<()> {
    let json_str = r#"
[
  {
    "ref": "refs/tags/v0.1",
    "node_id": "REF_kwDOHcsoLq5yZWZzL3RhZ3MvdjAuMQ",
    "url": "https://api.github.com/repos/lpenz/ghworkflow-rust/git/refs/tags/v0.1",
    "object": {
      "sha": "ca550057e88e5885030e756b90bd040ad7840cee",
      "type": "commit",
      "url": "https://api.github.com/repos/lpenz/ghworkflow-rust/git/commits/ca550057e88e5885030e756b90bd040ad7840cee"
    }
  },
  {
    "ref": "refs/tags/0.2",
    "node_id": "REF_kwDOHcsoLq5yZWZzL3RhZ3MvdjAuMg",
    "url": "https://api.github.com/repos/lpenz/ghworkflow-rust/git/refs/tags/v0.2",
    "object": {
      "sha": "2b80e7d13e4b1738a17887b4d66143433267cea6",
      "type": "commit",
      "url": "https://api.github.com/repos/lpenz/ghworkflow-rust/git/commits/2b80e7d13e4b1738a17887b4d66143433267cea6"
    }
  },
  {
    "ref": "refs/tags/latest",
    "node_id": "REF_kwDOHcsoLq5yZWZzL3RhZ3MvdjAuMw",
    "url": "https://api.github.com/repos/lpenz/ghworkflow-rust/git/refs/tags/v0.3",
    "object": {
      "sha": "c7d367f5f10a2605aa43a540f9f88177d5fa12ac",
      "type": "commit",
      "url": "https://api.github.com/repos/lpenz/ghworkflow-rust/git/commits/c7d367f5f10a2605aa43a540f9f88177d5fa12ac"
    }
  },
  {
    "ref": "refs/tags/v0.4",
    "node_id": "REF_kwDOHcsoLq5yZWZzL3RhZ3MvdjAuNA",
    "url": "https://api.github.com/repos/lpenz/ghworkflow-rust/git/refs/tags/v0.4",
    "object": {
      "sha": "04bb04c23563d3302fe6ca0c2b832e9e67c47d58",
      "type": "commit",
      "url": "https://api.github.com/repos/lpenz/ghworkflow-rust/git/commits/04bb04c23563d3302fe6ca0c2b832e9e67c47d58"
    }
  }
]
"#;
    let json_value: serde_json::Value = serde_json::from_str(json_str)?;
    let gh = Github::default();
    let mut versions = parse_versions(&gh, json_value)?
        .into_iter()
        .collect::<Vec<_>>();
    versions.sort();
    let versions = versions
        .into_iter()
        .map(|v| format!("{}", v))
        .collect::<Vec<_>>();
    assert_eq!(versions, ["latest", "v0.1", "0.2", "v0.4"]);
    Ok(())
}
