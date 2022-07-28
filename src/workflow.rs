// Copyright (C) 2022 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use anyhow::{anyhow, Result};
use futures::future::join_all;
use serde_yaml::Value;
use std::io;
use std::path;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tracing::event;
use tracing::instrument;
use tracing::Level;
use versions::Version;

use crate::entity::Entity;
use crate::resolver;

#[derive(Debug)]
pub struct Workflow {
    pub filename: path::PathBuf,
    pub contents: String,
    pub entities: Vec<Entity>,
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
            .collect::<Vec<_>>();
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

macro_rules! regex {
    ($re:literal $(,)?) => {{
        static RE: once_cell::sync::OnceCell<regex::Regex> = once_cell::sync::OnceCell::new();
        RE.get_or_init(|| regex::Regex::new($re).unwrap())
    }};
}

#[instrument(level = "debug")]
fn reference_parse_version(reference: &str) -> Option<(String, Version)> {
    let re_docker = regex!(r"^(?P<resource>docker://[^:]+):(?P<version>[^:]+)$");
    if let Some(m) = re_docker.captures(reference) {
        return Some((
            m.name("resource").unwrap().as_str().into(),
            Version::new(m.name("version").unwrap().as_str())?,
        ));
    }
    let re_github = regex!(r"^(?P<userrepo>[^/]+/[^@]+)@(?P<version>[^@]+)$");
    if let Some(m) = re_github.captures(reference) {
        return Some((
            format!("github://{}", m.name("userrepo").unwrap().as_str()),
            Version::new(m.name("version").unwrap().as_str())?,
        ));
    }
    None
}

#[instrument(level = "debug", skip(r))]
fn buf_parse(r: impl io::BufRead) -> Result<Vec<Entity>> {
    let data: serde_yaml::Mapping = serde_yaml::from_reader(r)?;
    let jobs = data
        .get(&Value::String("jobs".into()))
        .ok_or_else(|| anyhow!("jobs entry not found"))?
        .as_mapping()
        .ok_or_else(|| anyhow!("invalid type for jobs entry"))?;
    let mut ret = vec![];
    for (jobname_, job) in jobs {
        if let Some(steps) = job.get(&Value::String("steps".into())) {
            let steps = steps
                .as_sequence()
                .ok_or_else(|| anyhow!("invalid type for steps entry"))?;
            let jobname = jobname_
                .as_str()
                .ok_or_else(|| anyhow!("invalid type for job key"))?;
            for step in steps {
                if let Some(uses) = step.get(&Value::String("uses".into())) {
                    let reference = uses
                        .as_str()
                        .ok_or_else(|| anyhow!("invalid type for uses entry"))?;
                    if let Some((resource, version)) = reference_parse_version(reference) {
                        let entity = Entity {
                            job: String::from(jobname),
                            line: reference.into(),
                            resource,
                            version,
                            ..Default::default()
                        };
                        event!(Level::INFO, reference = reference, "parsed entity");
                        ret.push(entity);
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
