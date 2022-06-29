// Copyright (C) 2022 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use futures::future::try_join_all;
use tokio::io::AsyncReadExt;
use tokio_stream::wrappers::ReadDirStream;
use tokio_stream::StreamExt;

pub mod entity;
use entity::Entity;

pub mod workflow;

pub async fn workflow_process(filename: tokio::fs::DirEntry) -> Result<Vec<Entity>> {
    let filename = filename.path();
    let mut file = tokio::fs::File::open(&filename).await?;
    let mut contents = vec![];
    file.read_to_end(&mut contents).await?;
    workflow::buf_parse(&*contents)
        .with_context(|| format!("error parsing {}", &filename.display()))
}

pub async fn main() -> Result<()> {
    let futures = ReadDirStream::new(tokio::fs::read_dir(".github/workflows").await?)
        .map(|filename| async {
            if let Ok(filename) = filename {
                workflow_process(filename).await
            } else {
                Err(anyhow!("error getting filename"))
            }
        })
        .collect::<Vec<_>>()
        .await;
    let entities = try_join_all(futures).await?;
    for entity in entities.iter().flatten() {
        println!(
            "job {} uses {} version {}",
            entity.job, entity.resource, entity.version
        );
    }
    Ok(())
}
