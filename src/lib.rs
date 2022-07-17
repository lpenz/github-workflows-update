// Copyright (C) 2022 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use anyhow::Result;
use futures::future::join_all;
use std::path;
use tokio::io::AsyncReadExt;
use tokio_stream::wrappers::ReadDirStream;
use tokio_stream::StreamExt;
use tracing::event;
use tracing::instrument;
use tracing::Level;

pub mod entity;
use entity::Entity;

pub mod workflow;

pub mod vers;

#[instrument(fields(filename = ?filename.as_ref().display()))]
pub async fn workflow_process(filename: impl AsRef<path::Path>) -> Result<Vec<Entity>> {
    let mut file = tokio::fs::File::open(filename).await?;
    let mut contents = vec![];
    file.read_to_end(&mut contents).await?;
    workflow::buf_parse(&*contents)
}

#[instrument(fields(filename = ?filename.as_ref().display()))]
pub async fn process_file(filename: impl AsRef<path::Path>) {
    let filename = filename.as_ref();
    let entities = match workflow_process(filename).await {
        Ok(entities) => entities,
        Err(e) => {
            event!(
                Level::ERROR,
                error = ?e,
                filename = ?filename,
            );
            return;
        }
    };
    let resolver = vers::resolver::Server::new();
    let resolve_entity_tasks = entities
        .into_iter()
        .map(|e| (e, resolver.new_client()))
        .map(|(e, resolver_client)| async move { resolver_client.resolve_entity(e).await });
    let entities = join_all(resolve_entity_tasks)
        .await
        .into_iter()
        .collect::<Vec<_>>();
    for entity in &entities {
        if let Some(ref latest) = entity.latest {
            if &entity.version != latest {
                println!(
                    "{} {} {} -> {}",
                    filename.display(),
                    entity.resource,
                    entity.version,
                    latest
                );
            }
        }
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
