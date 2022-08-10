// Copyright (C) 2022 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! Workflow file parsing, into [`Workflow`] type.

use anyhow::{anyhow, Result};
use futures::future::join_all;
use serde_yaml::Value;
use std::collections::HashMap;
use std::collections::HashSet;
use std::io;
use std::path;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tracing::event;
use tracing::instrument;
use tracing::Level;

use crate::proxy;
use crate::resource::Resource;
use crate::version::Version;

#[derive(Debug)]
pub struct Workflow {
    /// The name of the workflow file.
    pub filename: path::PathBuf,
    /// Contents of the workflow file as a `String`.
    pub contents: String,
    /// Set with all [`Resource`]s that the workflow `uses` along with the
    /// current versions.
    pub uses: HashSet<(Resource, Version)>,
    /// The latest version of each [`Resource`] as fetched from the
    /// upstream docker or github repository.
    pub latest: HashMap<Resource, Version>,
}

impl Workflow {
    #[instrument(level="debug", fields(filename = ?filename.as_ref().display()))]
    pub async fn new(filename: impl AsRef<path::Path>) -> Result<Workflow> {
        let filename = filename.as_ref();
        let mut file = tokio::fs::File::open(filename).await?;
        let mut contents = String::new();
        file.read_to_string(&mut contents).await?;
        let uses = buf_parse(contents.as_bytes())?;
        Ok(Workflow {
            filename: filename.to_owned(),
            contents,
            uses,
            latest: Default::default(),
        })
    }

    #[instrument(level = "debug")]
    pub async fn fetch_latest_versions(&mut self, proxy_server: &proxy::Server) {
        let tasks = self
            .uses
            .iter()
            .map(|rv| (rv, proxy_server.new_client()))
            .map(|((resource, current_version), proxy_client)| async move {
                proxy_client
                    .fetch_latest_version(resource, current_version)
                    .await
            });
        self.latest = join_all(tasks)
            .await
            .into_iter()
            .flatten()
            .collect::<HashMap<_, _>>();
    }

    #[instrument(level = "debug")]
    pub async fn update_file(&self) -> Result<bool> {
        let mut contents = self.contents.clone();
        for (resource, current_version) in &self.uses {
            if let Some(latest_version) = self.latest.get(resource) {
                let current_line = resource.versioned_string(current_version);
                let latest_line = resource.versioned_string(latest_version);
                contents = contents.replace(&current_line, &latest_line);
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

#[instrument(level = "debug", skip(r))]
fn buf_parse(r: impl io::BufRead) -> Result<HashSet<(Resource, Version)>> {
    let data: serde_yaml::Mapping = serde_yaml::from_reader(r)?;
    let jobs = data
        .get(&Value::String("jobs".into()))
        .ok_or_else(|| anyhow!("jobs entry not found"))?
        .as_mapping()
        .ok_or_else(|| anyhow!("invalid type for jobs entry"))?;
    let mut ret = HashSet::default();
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
                    if let Ok((resource, version)) = Resource::parse(reference) {
                        event!(
                            Level::INFO,
                            resource = %resource,
                            version = %version,
                            "parsed entity"
                        );
                        ret.insert((resource, version));
                    } else {
                        event!(
                            Level::WARN,
                            reference = reference,
                            "unable to parse resource"
                        );
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
