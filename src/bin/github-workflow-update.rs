// Copyright (C) 2022 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use anyhow::Result;

use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    /// Update the workflow file in-place
    #[clap(short, long = "in-place")]
    inplace: bool,
}

#[tokio::main]
pub async fn main() -> Result<()> {
    let args = Args::parse();
    github_workflow_update::main(args.inplace).await
}
