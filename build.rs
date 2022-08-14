// Copyright (C) 2020 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use anyhow::Result;
use man::prelude::*;
use std::env;
use std::error::Error;
use std::fs::{self, File};
use std::io::Write;
use std::path;

fn generate_man_page<P: AsRef<path::Path>>(outdir: P) -> anyhow::Result<()> {
    let outdir = outdir.as_ref();
    let man_path = outdir.join("github-workflows-update.1");
    let manpage = Manual::new("github-workflows-update")
        .about("Check github workflows for actions that can be updated")
        .author(Author::new("Leandro Lisboa Penz").email("lpenz@lpenz.org"))
        .flag(
            Flag::new()
                .short("-n")
                .long("--dry-run")
                .help("Don't update the workflows, just print what would be done"),
        )
        .option(
            Opt::new("output-format")
                .short("-f")
                .long("--output-format")
                .default_value("standard")
                .help(
                    "Output format for the outdated action messages; \
                       one of \"standard\" or \"github-warning\"",
                ),
        )
        .flag(
            Flag::new()
                .long("--error-on-outdated")
                .help("Return error if any outdated actions are found"),
        )
        .flag(
            Flag::new()
                .short("-h")
                .long("--help")
                .help("Prints help information"),
        )
        .flag(
            Flag::new()
                .short("-V")
                .long("--version")
                .help("Prints version information"),
        )
        .arg(Arg::new("COMMAND"))
        .arg(Arg::new("[ ARGS ]"))
        .description(
            "github-workflows-update reads all github workflow and checks the latest
available versions of all github actions and workflow dispatches used, showing
which ones can be updated and optionally updating them automatically.",
        )
        .example(
            Example::new()
                .text(
                    "Update all actions used in all github workflows \
                       under the current repository",
                )
                .command("github-workflows-update"),
        )
        .example(
            Example::new()
                .text("Show outdated actions without updating them")
                .command("github-workflows-update -n"),
        )
        .render();
    File::create(&man_path)?.write_all(manpage.as_bytes())?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut outdir = path::PathBuf::from(
        env::var_os("OUT_DIR").ok_or_else(|| anyhow::anyhow!("error getting OUT_DIR"))?,
    );
    fs::create_dir_all(&outdir)?;
    generate_man_page(&outdir)?;
    // build/github-workflows-update-*/out
    outdir.pop();
    // build/github-workflows-update-*
    outdir.pop();
    // build
    outdir.pop();
    // .
    // (either target/release or target/build)
    generate_man_page(&outdir)?;
    Ok(())
}
