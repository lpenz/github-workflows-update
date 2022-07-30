// Copyright (C) 2022 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use anyhow::anyhow;
use anyhow::ensure;
use anyhow::Context;
use anyhow::Result;
use async_trait::async_trait;
use reqwest::header::USER_AGENT;
use tracing::instrument;

use crate::entity::Entity;
use crate::updater;
use crate::version::Version;

macro_rules! regex {
    ($re:literal $(,)?) => {{
        static RE: once_cell::sync::OnceCell<regex::Regex> = once_cell::sync::OnceCell::new();
        RE.get_or_init(|| regex::Regex::new($re).unwrap())
    }};
}

#[derive(Debug, Default)]
pub struct Github {}

#[async_trait]
impl updater::Updater for Github {
    fn url(&self, resource: &str) -> Option<String> {
        url(resource)
    }

    async fn get_versions(&self, url: &str) -> Result<Vec<Version>> {
        get_versions(url).await
    }

    fn updated_line(&self, entity: &Entity) -> Option<String> {
        let path = entity.resource.strip_prefix("github://").unwrap();
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
        .map(|tag_obj| {
            tag_obj
                .as_object()
                .ok_or_else(|| anyhow!("invalid type for tag object"))?
                .get("ref")
                .ok_or_else(|| anyhow!("ref field not found in tag object"))
                .map(|ref_value| {
                    let re_ref = regex!(r"^refs/tags/(?P<version>.+)$");
                    let m =
                        re_ref
                            .captures(ref_value.as_str().ok_or_else(|| {
                                anyhow!("invalid format for ref field in tag object")
                            })?)
                            .ok_or_else(|| anyhow!("could not find ref field in tag object"))?;
                    Version::new(
                        m.name("version")
                            .ok_or_else(|| anyhow!("unable to parse version"))?
                            .as_str(),
                    )
                    .ok_or_else(|| anyhow!("unable to parse version"))
                })?
        })
        .collect::<Result<Vec<Version>>>()
}

#[instrument(level = "debug")]
pub async fn get_versions(url: &str) -> Result<Vec<Version>> {
    let data = get_json(url).await?;
    let versions =
        parse_versions(data).with_context(|| format!("error processing json from {}", url))?;
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
    "ref": "refs/tags/v0.2",
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
    let versions = parse_versions(json_value)?
        .into_iter()
        .map(|v| format!("{}", v))
        .collect::<Vec<_>>();
    assert_eq!(versions, ["v0.1", "v0.2", "latest", "v0.4"]);
    Ok(())
}
