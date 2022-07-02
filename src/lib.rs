// Copyright (C) 2022 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use anyhow::Context;
use anyhow::Result;
use futures::future::join_all;
use std::collections::HashMap;
use std::collections::HashSet;
use std::path;
use tokio::io::AsyncReadExt;
use tokio_stream::wrappers::ReadDirStream;
use tokio_stream::StreamExt;

pub mod entity;
use entity::Entity;

pub mod workflow;

pub mod vers;

pub async fn workflow_process(filename: impl AsRef<path::Path>) -> Result<Vec<Entity>> {
    let mut file = tokio::fs::File::open(filename).await?;
    let mut contents = vec![];
    file.read_to_end(&mut contents).await?;
    workflow::buf_parse(&*contents).context("parse error")
}

pub async fn resources_get_latest_versions(
    file_entities: &[(path::PathBuf, Result<Vec<Entity>>)],
) -> HashMap<&str, Result<versions::Version, anyhow::Error>> {
    let resources = file_entities
        .iter()
        .filter_map(|(_, result)| result.as_ref().ok())
        .flat_map(|entities| entities.iter().map(|e| e.resource.as_str()))
        .collect::<HashSet<_>>();
    join_all(
        resources
            .into_iter()
            .map(|r| async move { (r, vers::discover_latest_version(r).await) }),
    )
    .await
    .into_iter()
    .collect::<HashMap<_, _>>()
}

pub async fn main() -> Result<()> {
    let futures = ReadDirStream::new(tokio::fs::read_dir(".github/workflows").await?)
        .filter_map(|filename| match filename {
            Ok(filename) => Some(filename.path()),
            Err(e) => {
                eprintln!("{}", e);
                None
            }
        })
        .map(move |filename| async {
            let entities = workflow_process(&filename).await;
            (filename, entities)
        })
        .collect::<Vec<_>>()
        .await;
    let file_entities = join_all(futures).await;
    let versions = resources_get_latest_versions(&file_entities).await;
    for (filename, result) in file_entities.iter() {
        match result {
            Ok(ref entities) => {
                for e in entities {
                    if let Some(Ok(new_version)) = versions.get(e.resource.as_str()) {
                        if new_version > &e.version {
                            println!(
                                "{} {} {} -> {}",
                                filename.display(),
                                e.resource,
                                e.version,
                                new_version
                            );
                        }
                    }
                }
            }
            Err(error) => {
                println!("{}: {:#}", filename.display(), error);
            }
        }
    }
    Ok(())
}
