// Copyright (C) 2020 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use clap::CommandFactory;
use clap_complete::generate_to;
use clap_complete::shells::Bash;
use clap_complete::shells::Fish;
use clap_complete::shells::Zsh;
use color_eyre::{Result, eyre::Context, eyre::eyre};
use std::env;
use std::fs::{self, File};
use std::io::Write;
use std::path;

include!("src/cli.rs");

fn generate_man_page<P: AsRef<path::Path>>(outdir: P) -> Result<()> {
    let outdir = outdir.as_ref();
    let man_path = outdir.join("github-workflows-update.1");
    let cmd = Cli::command();
    let manual: man::Manual = clap2man::Manual::try_from(&cmd)
        .wrap_err("error converting clap command to manual")?
        .into();
    let manpage = manual
        .example(
            man::prelude::Example::new()
                .text(
                    "Update all actions used in all github workflows \
                       under the current repository",
                )
                .command("github-workflows-update"),
        )
        .example(
            man::prelude::Example::new()
                .text("Show outdated actions without updating them")
                .command("github-workflows-update -n"),
        )
        .render();
    File::create(man_path)?.write_all(manpage.as_bytes())?;
    Ok(())
}

fn main() -> Result<()> {
    let mut outdir =
        path::PathBuf::from(env::var_os("OUT_DIR").ok_or_else(|| eyre!("error getting OUT_DIR"))?);
    // build/github-workflows-update-*/out
    outdir.pop();
    // build/github-workflows-update-*
    outdir.pop();
    // build
    outdir.pop();
    // .
    // (either target/release or target/build)

    fs::create_dir_all(&outdir)?;
    generate_man_page(&outdir)?;

    let mut cmd = Cli::command();
    generate_to(Bash, &mut cmd, "github-workflows-update", &outdir)?;
    generate_to(Fish, &mut cmd, "github-workflows-update", &outdir)?;
    generate_to(Zsh, &mut cmd, "github-workflows-update", &outdir)?;

    Ok(())
}
