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
use tracing::instrument;

pub mod entity;
use entity::Entity;

pub mod workflow;

pub mod vers;

#[instrument(fields(filename = ?filename.as_ref().display()))]
pub async fn workflow_process(filename: impl AsRef<path::Path>) -> Result<Vec<Entity>> {
    let mut file = tokio::fs::File::open(filename).await?;
    let mut contents = vec![];
    file.read_to_end(&mut contents).await?;
    workflow::buf_parse(&*contents).context("parse error")
}

#[instrument(fields(filename = ?filename.as_ref().display()))]
pub async fn do_process_file(filename: impl AsRef<path::Path>) -> Result<()> {
    let filename = filename.as_ref();
    let entities = workflow_process(filename).await?;
    let resolver = vers::resolver::Server::new();
    let resources = entities
        .iter()
        .map(|e| e.resource.as_str())
        .collect::<HashSet<_>>();
    let version_tasks = resources
        .into_iter()
        .map(|r| (r, resolver.new_client()))
        .map(
            |(r, resolver_client)| async move { (r, resolver_client.get_latest_version(r).await) },
        );
    let versions = join_all(version_tasks)
        .await
        .into_iter()
        .collect::<HashMap<_, _>>();
    for entity in &entities {
        match versions.get(entity.resource.as_str()) {
            Some(Ok(new_version)) => {
                if new_version > &entity.version {
                    println!(
                        "{} {} {} -> {}",
                        filename.display(),
                        entity.resource,
                        entity.version,
                        new_version
                    );
                }
            }
            Some(Err(ref err)) => {
                println!("{} {} -> {:#}", filename.display(), entity.resource, err);
            }
            None => {
                println!(
                    "{} {} -> latest version not found",
                    filename.display(),
                    entity.resource
                );
            }
        }
    }
    Ok(())
}

#[instrument]
pub async fn process_file(filename: path::PathBuf) {
    if let Err(err) = do_process_file(&filename).await {
        eprintln!("{}: {:#}", filename.display(), err)
    }
}

pub async fn main() -> Result<()> {
    env_logger::init();
    let futures = ReadDirStream::new(tokio::fs::read_dir(".github/workflows").await?)
        .filter_map(|filename| match filename {
            Ok(filename) => Some(filename.path()),
            Err(e) => {
                eprintln!("{}", e);
                None
            }
        })
        .map(process_file)
        .collect::<Vec<_>>()
        .await;
    join_all(futures).await;
    Ok(())
}
