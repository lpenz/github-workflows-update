// Copyright (C) 2022 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! Command line arguments parsing and main function.

use anyhow::Result;
use futures::future::join_all;
use tokio_stream::wrappers::ReadDirStream;
use tokio_stream::StreamExt;
use tracing::event;
use tracing::Level;

use clap::ArgEnum;
use clap::Parser;

use crate::resolver;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    /// Don't update the workflows, just print the outdated actions
    #[clap(short = 'n', long = "dry-run")]
    pub dryrun: bool,
    /// Output format for the outdated action messages
    #[clap(short = 'f', long, arg_enum, value_parser)]
    pub output_format: Option<OutputFormat>,
    /// Return error if any outdated actions are found
    #[clap(long)]
    pub error_on_outdated: bool,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ArgEnum, Default, Debug)]
pub enum OutputFormat {
    #[default]
    Standard,
    /// Generate messages as github action warnings
    GithubWarning,
}

#[tokio::main]
pub async fn main() -> Result<()> {
    let args = Args::parse();
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
        .map(|f| {
            crate::processor::process_file(
                args.dryrun,
                args.output_format.unwrap_or_default(),
                &resolver,
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
