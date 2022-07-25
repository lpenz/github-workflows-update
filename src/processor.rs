// Copyright (C) 2022 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use std::path;
use tracing::event;
use tracing::instrument;
use tracing::Level;

use crate::cmd::OutputFormat;
use crate::resolver;
use crate::workflow::Workflow;

#[instrument(level="info", fields(filename = ?filename.as_ref().display()))]
pub async fn process_file(
    dryrun: bool,
    output: OutputFormat,
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
    let dryrunmsg = if dryrun { " (dryrun)" } else { "" };
    for entity in &workflow.entities {
        if let Some(ref latest) = entity.latest {
            if &entity.version != latest {
                match output {
                    OutputFormat::Standard => {
                        println!(
                            "{}: update {} from {} to {}{}",
                            filename.display(),
                            entity.resource,
                            entity.version,
                            latest,
                            dryrunmsg
                        );
                    }
                    OutputFormat::GithubWarning => {
                        println!(
                            "::warning file={}::update {} from {} to {}",
                            filename.display(),
                            entity.resource,
                            entity.version,
                            latest,
                        );
                    }
                }
            }
        }
    }
    if !dryrun {
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
