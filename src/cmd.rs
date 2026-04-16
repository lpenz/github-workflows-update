// Copyright (C) 2022 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! Command line arguments parsing and main function.

use color_eyre::Result;
use futures::future::join_all;
use tokio_stream::StreamExt;
use tokio_stream::wrappers::ReadDirStream;
use tracing::Level;
use tracing::event;

use crate::cli::Cli;
use crate::cli::OutputFormat;
use crate::proxy;
use clap::Parser;

#[tokio::main]
pub async fn main() -> Result<()> {
    let args = Cli::parse();
    env_logger::init();
    let proxy_server = proxy::Server::new();
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
        .map(|f| {
            crate::processor::process_file(
                args.dryrun,
                args.output_format.unwrap_or_default(),
                &proxy_server,
                f,
            )
        })
        .collect::<Vec<_>>()
        .await;
    let mut any_outdated = false;
    for result in join_all(futures).await {
        match result {
            Ok(true) => {
                any_outdated = true;
            }
            Err(_) => {
                // Errors are traced by the underlying functions, we
                // just need to report the failure to the shell
                std::process::exit(1);
            }
            _ => {}
        }
    }
    if any_outdated && args.error_on_outdated {
        match args.output_format.unwrap_or_default() {
            OutputFormat::Standard => {
                eprintln!("Found oudated entities");
            }
            OutputFormat::GithubWarning => {
                println!("::error ::outdated entities found");
            }
        }
        std::process::exit(2);
    }
    Ok(())
}
