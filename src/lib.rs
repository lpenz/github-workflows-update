// Copyright (C) 2022 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use anyhow::Context;
use anyhow::Result;
use futures::future::join_all;
use std::path;
use tokio::io::AsyncReadExt;
use tokio_stream::wrappers::ReadDirStream;
use tokio_stream::StreamExt;

pub mod entity;
use entity::Entity;

pub mod workflow;

pub async fn workflow_process(filename: impl AsRef<path::Path>) -> Result<Vec<Entity>> {
    let mut file = tokio::fs::File::open(filename).await?;
    let mut contents = vec![];
    file.read_to_end(&mut contents).await?;
    workflow::buf_parse(&*contents).context("parse error")
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
    for (filename, result) in file_entities.iter() {
        match result {
            Ok(entities) => {
                println!("{}: {:?}", filename.display(), entities);
            }
            Err(error) => {
                println!("{}: {:#}", filename.display(), error);
            }
        }
    }
    Ok(())
}
