// Copyright (C) 2022 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use anyhow::Result;
use futures::future::join_all;
use std::path;
use tokio_stream::wrappers::ReadDirStream;
use tokio_stream::StreamExt;
use tracing::event;
use tracing::instrument;
use tracing::Level;

pub mod entity;

pub mod workflow;
use workflow::Workflow;

pub mod updater;

pub mod resolver;

pub mod prettyvers;

#[instrument(level="info", fields(filename = ?filename.as_ref().display()))]
pub async fn process_file(
    inplace: bool,
    resolver: &resolver::Server,
    filename: impl AsRef<path::Path>,
) {
    let filename = filename.as_ref();
    let mut workflow = match Workflow::new(filename).await {
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
    workflow.resolve_entities(resolver).await;
    for entity in &workflow.entities {
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
    if inplace {
        match workflow.update_file().await {
            Ok(true) => {
                event!(
                    Level::INFO,
                    filename = ?filename,
                    "updated"
                );
            }
            Ok(false) => {
                event!(
                    Level::INFO,
                    filename = ?filename,
                    "unchanged"
                );
            }
            Err(e) => {
                event!(
                    Level::ERROR,
                    error = ?e,
                    filename = ?filename,
                    "error writing updated file"
                );
            }
        }
    }
}

pub async fn main(inplace: bool) -> Result<()> {
    env_logger::init();
    let resolver = resolver::Server::new();
    let futures = ReadDirStream::new(tokio::fs::read_dir(".github/workflows").await?)
        .filter_map(|filename| match filename {
            Ok(filename) => Some(filename.path()),
            Err(ref e) => {
                event!(
                    Level::ERROR,
                    error = ?e,
                    filename = ?filename,
                    "error getting filename from .github/workflows"
                );
                None
            }
        })
        .map(|f| process_file(inplace, &resolver, f))
        .collect::<Vec<_>>()
        .await;
    join_all(futures).await;
    Ok(())
}
