// Copyright (C) 2022 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! Workflow file parsing, into [`Workflow`] type.
//!
//! A workflow can have one or more [`Entity`]s that represent
//! resource with a version.

use anyhow::{anyhow, Result};
use futures::future::join_all;
use regex::Regex;
use serde_yaml::Value;
use std::collections::HashSet;
use std::io;
use std::path;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tracing::event;
use tracing::instrument;
use tracing::Level;

use crate::entity::Entity;
use crate::resolver;
use crate::version::Version;

#[derive(Debug)]
pub struct Workflow {
    pub filename: path::PathBuf,
    pub contents: String,
    pub entities: HashSet<Entity>,
}

impl Workflow {
    #[instrument(level="debug", fields(filename = ?filename.as_ref().display()))]
    pub async fn new(filename: impl AsRef<path::Path>) -> Result<Workflow> {
        let filename = filename.as_ref();
        let mut file = tokio::fs::File::open(filename).await?;
        let mut contents = String::new();
        file.read_to_string(&mut contents).await?;
        let entities = buf_parse(contents.as_bytes())?;
        Ok(Workflow {
            filename: filename.to_owned(),
            contents,
            entities,
        })
    }

    #[instrument(level = "debug")]
    pub async fn resolve_entities(&mut self, resolver: &resolver::Server) {
        let entities = std::mem::take(&mut self.entities);
        let resolve_entity_tasks = entities
            .into_iter()
            .map(|e| (e, resolver.new_client()))
            .map(|(e, resolver_client)| async move { resolver_client.resolve_entity(e).await });
        self.entities = join_all(resolve_entity_tasks)
            .await
            .into_iter()
            .collect::<HashSet<_>>();
    }

    #[instrument(level = "debug")]
    pub async fn update_file(&self) -> Result<bool> {
        let mut contents = self.contents.clone();
        for entity in &self.entities {
            if let Some(updated_line) = &entity.updated_line {
                contents = contents.replace(&entity.line, updated_line);
            }
        }
        let updated = contents != self.contents;
        if updated {
            let mut file = tokio::fs::File::create(&self.filename).await?;
            file.write_all(contents.as_bytes()).await?;
        }
        Ok(updated)
    }
}

#[instrument(level = "debug")]
fn reference_parse_version(
    re_docker: &Regex,
    re_github: &Regex,
    reference: &str,
) -> Option<(String, Version)> {
    if let Some(m) = re_docker.captures(reference) {
        return Some((
            m.name("resource").unwrap().as_str().into(),
            Version::new(m.name("version").unwrap().as_str())?,
        ));
    }
    if let Some(m) = re_github.captures(reference) {
        return Some((
            format!("github://{}", m.name("userrepo").unwrap().as_str()),
            Version::new(m.name("version").unwrap().as_str())?,
        ));
    }
    None
}

#[instrument(level = "debug", skip(r))]
fn buf_parse(r: impl io::BufRead) -> Result<HashSet<Entity>> {
    let data: serde_yaml::Mapping = serde_yaml::from_reader(r)?;
    let jobs = data
        .get(&Value::String("jobs".into()))
        .ok_or_else(|| anyhow!("jobs entry not found"))?
        .as_mapping()
        .ok_or_else(|| anyhow!("invalid type for jobs entry"))?;
    let mut ret = HashSet::default();
    let re_docker = Regex::new(r"^(?P<resource>docker://[^:]+):(?P<version>[^:]+)$").unwrap();
    let re_github = Regex::new(r"^(?P<userrepo>[^/]+/[^@]+)@(?P<version>[^@]+)$").unwrap();
    for (_, job) in jobs {
        if let Some(steps) = job.get(&Value::String("steps".into())) {
            let steps = steps
                .as_sequence()
                .ok_or_else(|| anyhow!("invalid type for steps entry"))?;
            for step in steps {
                if let Some(uses) = step.get(&Value::String("uses".into())) {
                    let reference = uses
                        .as_str()
                        .ok_or_else(|| anyhow!("invalid type for uses entry"))?;
                    if let Some((resource, version)) =
                        reference_parse_version(&re_docker, &re_github, reference)
                    {
                        let entity = Entity {
                            line: reference.into(),
                            resource,
                            version,
                            ..Default::default()
                        };
                        event!(Level::INFO, reference = reference, "parsed entity");
                        ret.insert(entity);
                    } else {
                        event!(Level::WARN, reference = reference, "entity not parsed");
                    }
                }
            }
        }
    }
    Ok(ret)
}

#[test]
fn test_parse() -> Result<()> {
    let s = r"
---
name: test
jobs:
  omnilint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: docker://lpenz/omnilint:0.4
      - run: ls
";
    buf_parse(s.as_bytes())?;
    Ok(())
}
